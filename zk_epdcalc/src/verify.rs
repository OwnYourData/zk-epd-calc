use crate::error::AppError;
use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use risc0_zkvm::Receipt;
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use zk_epdcalc_core::VerifiedEpd;

struct VerifyConfigImpl<Epd, ZkEpd> {
    guest_id: &'static [u32; 8],
    zk_epd: ZkEpd,
    phantom_data: PhantomData<Epd>
}

impl<Epd, ZkEpd> Debug for VerifyConfigImpl<Epd, ZkEpd>
where
    ZkEpd: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifyConfig")
            .field("epd", &self.zk_epd)
            .finish()
    }
}

pub fn new_config<Epd, ZkEpd>(guest_id: &'static [u32; 8], zk_epd: ZkEpd) -> Box<dyn VerifyConfig>
where
    ZkEpd: VerifiedEpd<Epd> + Debug + Send + Sync + 'static,
    Epd: Eq + Debug + Send + Sync + DeserializeOwned + 'static
{
    let config: VerifyConfigImpl<Epd, ZkEpd> = VerifyConfigImpl {
        guest_id, zk_epd, phantom_data: Default::default()
    };
    Box::new(config)
}


impl<Epd, ZkEpd> VerifyConfig for VerifyConfigImpl<Epd, ZkEpd>
where
    ZkEpd: VerifiedEpd<Epd> + Debug + Send + Sync,
    Epd: Send + Eq + Debug + DeserializeOwned
{
    fn verify(&self) -> Result<(), AppError> {
        let zkp = self.zk_epd.get_zkp();
        let receipt_cbor = BASE64_STANDARD.decode(zkp)
            .or(Err(AppError::ProofDecodingFailed))?;
        let receipt: Receipt =
            ciborium::de::from_reader(receipt_cbor.as_slice())
                .or(Err(AppError::ProofDecodingFailed))?;
        let epd: Epd = receipt.journal.decode().or(Err(AppError::ProofDecodingFailed))?;
        if epd == *self.zk_epd.get_epd() {
            receipt.verify(*self.guest_id).or(Err(AppError::InvalidProof))
        } else {
            Err(AppError::NonMatchingEPDInfo)
        }
    }
}

pub trait VerifyConfig: Debug + Send{
    fn verify(&self) -> Result<(), AppError>;
}