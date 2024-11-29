/*
 * Copyright (c) 2024 Thomas Preindl
 * MIT License (see LICENSE or https://mit-license.org)
 */

use crate::proving::{new_config as proof_config, ProofConfig, ProofInput, ProofResponse};
use crate::verify::{new_config as verify_config, VerifyConfig};
use axum::body::Bytes;
use axum::extract::rejection::{BytesRejection, JsonRejection, QueryRejection};
use axum::extract::{FromRef, FromRequest, FromRequestParts, Query, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::identity;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use thiserror::Error;
use zk_epdcalc_core::VerifiedEpd;

#[derive(Error, Debug)]
pub enum RequestRejection {
    #[error("Unknown value for zktype: {0}")]
    UnknownZkType(Box<str>),
    #[error("Query Error")]
    QueryRejection(#[from] QueryRejection),
    #[error("Json Parse Error")]
    JsonRejection(#[from] JsonRejection),
    #[error("Bytes buffering Error")]
    BytesRejection(#[from] BytesRejection),
}

impl IntoResponse for RequestRejection {
    fn into_response(self) -> Response {
        match self {
            RequestRejection::UnknownZkType(_) => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            RequestRejection::QueryRejection(qr) => qr.into_response(),
            RequestRejection::JsonRejection(jr) => jr.into_response(),
            RequestRejection::BytesRejection(br) => br.into_response(),
        }
    }
}

type StoredProofConfig = Box<dyn ProofConfig>;

pub trait ConfigFactory {
    fn build_proof_config(
        &self,
        bytes: &Bytes,
        snark_p: bool,
    ) -> Result<StoredProofConfig, RequestRejection>;
    fn build_verify_config(&self, bytes: &Bytes) -> Result<Box<dyn VerifyConfig>, RequestRejection>;
}

impl<I, Epd, ZkEpd> ConfigFactory for ConfigFactoryImpl<I, Epd, ZkEpd>
where
    I: ProofInput + DeserializeOwned + Debug + Send + Sync + 'static,
    ZkEpd: ProofResponse + VerifiedEpd<Epd> + Serialize + Sync + Send + Debug + DeserializeOwned + 'static,
    Epd: DeserializeOwned + Debug + Sync + Send + Eq + 'static,
{
    fn build_proof_config(
        &self,
        bytes: &Bytes,
        snark_p: bool,
    ) -> Result<StoredProofConfig, RequestRejection> {
        let Json(input): Json<I> = Json::from_bytes(bytes)?;
        let config = proof_config::<I, Epd, ZkEpd>(snark_p, self.guest_elf, input);
        Ok(config)
    }

    fn build_verify_config(
        &self,
        bytes: &Bytes,
    ) -> Result<Box<dyn VerifyConfig>, RequestRejection> {
        let Json(verify_epd): Json<ZkEpd> = Json::from_bytes(bytes)?;
        let config = verify_config(self.guest_id, verify_epd);
        Ok(config)
    }
}

struct ConfigFactoryImpl<I, Epd, ZkEpd, > {
    guest_elf: &'static [u8],
    guest_id: &'static [u32; 8],
    phantom_input: PhantomData<I>,
    phantom_zk_epd: PhantomData<ZkEpd>,
    phantom_epd: PhantomData<Epd>,
}

pub fn new_config_factory<I, Epd, ZkEpd>(
    guest_elf: &'static [u8],
    guest_id: &'static [u32; 8],
) -> Box<dyn ConfigFactory + Send + Sync>
where
    I: ProofInput + DeserializeOwned + Debug + Send + Sync + 'static,
    ZkEpd: ProofResponse + Serialize + VerifiedEpd<Epd> + Sync + Send + Debug + DeserializeOwned + 'static,
    Epd: DeserializeOwned + Debug + Sync + Send + Eq + 'static,
{
    let config_factory: ConfigFactoryImpl<I, Epd, ZkEpd> = ConfigFactoryImpl {
        guest_elf,
        guest_id,
        phantom_input: Default::default(),
        phantom_zk_epd: Default::default(),
        phantom_epd: Default::default(),
    };
    Box::new(config_factory)
}

#[derive(Clone)]
pub(crate) struct ConfigFactoryMap {
    map: Arc<HashMap<&'static str, Box<dyn ConfigFactory + Sync + Send>>>,
}

impl ConfigFactoryMap {
    pub(crate) fn new(mapping: Vec<(&'static str, Box<dyn ConfigFactory + Sync + Send>)>) -> Self {
        let map = Arc::new(mapping.into_iter().collect());
        Self { map }
    }
}

#[derive(Deserialize)]
struct CreateParams {
    snark: Option<bool>,
    #[serde(rename = "zktype")]
    zk_type: Box<str>,
}

pub(crate) struct ExtractConfig(pub Box<dyn ProofConfig>);

#[async_trait]
impl<OuterState> FromRequest<OuterState> for ExtractConfig
where
    ConfigFactoryMap: FromRef<OuterState>,
    OuterState: Send + Sync,
{
    type Rejection = RequestRejection;

    async fn from_request(req: Request, state: &OuterState) -> Result<Self, Self::Rejection> {
        let config_map = ConfigFactoryMap::from_ref(state);
        let (mut parts, body) = req.into_parts();

        let Query(params): Query<CreateParams> =
            Query::from_request_parts(&mut parts, state).await?;
        let snark_p = params.snark.is_some_and(identity);
        let zk_type = params.zk_type.as_ref();

        let req = Request::from_parts(parts, body);
        let bytes = Bytes::from_request(req, state).await?;

        let factory = config_map
            .map
            .get(zk_type)
            .ok_or(RequestRejection::UnknownZkType(params.zk_type))?;
        let config = factory.build_proof_config(&bytes, snark_p)?;
        Ok(Self(config))
    }
}

pub struct Verify(pub Box<dyn VerifyConfig>);

#[async_trait]
impl<OuterState> FromRequest<OuterState> for Verify
where
    ConfigFactoryMap: FromRef<OuterState>,
    OuterState: Send + Sync,
{
    type Rejection = RequestRejection;

    async fn from_request(req: Request, state: &OuterState) -> Result<Self, Self::Rejection> {
        let config_map = ConfigFactoryMap::from_ref(state);
        let (mut parts, body) = req.into_parts();

        let Query(params): Query<CreateParams> =
            Query::from_request_parts(&mut parts, state).await?;
        let zk_type = params.zk_type.as_ref();

        let req = Request::from_parts(parts, body);
        let bytes = Bytes::from_request(req, state).await?;

        let factory = config_map
            .map
            .get(zk_type)
            .ok_or(RequestRejection::UnknownZkType(params.zk_type))?;
        let config = factory.build_verify_config(&bytes)?;
        Ok(Self(config))
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    #[test]
    fn add_build() {
        #[derive(Deserialize)]
        struct TestStruct {}
    }
}
