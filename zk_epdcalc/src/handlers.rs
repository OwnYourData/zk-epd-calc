use crate::error::AppError;
use crate::proving;
use crate::proving::{ProvingService, TaskStatus};
use crate::requests::{ConfigFactoryMap, ExtractConfig, Verify};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;


pub(crate) async fn create_epd(
    ExtractConfig(config): ExtractConfig
) -> ResponseResult<Response> {
    println!("Creating epd for {config:?}");

    let proof_response = proving::generate_epd(config)?;
    Ok(proof_response.into_response())
}

pub(crate) async fn start_epd_creation(
    State(service): State<ProvingService>,
    ExtractConfig(config): ExtractConfig,
) -> ResponseResult<EPDTaskStatus> {
    println!("{config:?}");

    let id = service.add_task(config);

    Ok(EPDTaskStatus {
        id,
        state: TaskStatus::Submitted,
    })
}

pub(crate) async fn get_epd_creation(
    State(service): State<ProvingService>,
    Path(id): Path<Uuid>,
) -> ResponseResult<EPDTaskStatus> {
    service
        .get_status(id)
        .await
        .map(|state| EPDTaskStatus {
            id,
            state,
        })
}

pub(crate) async fn get_epd_result(
    State(service): State<ProvingService>,
    Path(id): Path<Uuid>,
) -> ResponseResult<Response> {
    let response = service
        .get_status(id)
        .await
        .and_then(|status| status.get_response())?;
    Ok(response.into_response())
}

//#[debug_handler]
pub(crate) async fn post_verify_epd(
    State(_): State<ConfigFactoryMap>,
    Verify(config): Verify,
) -> ResponseResult<Json<VerificationResponse>> {
    println!("Verifying EPD for {config:?}");

    config.verify()?;
    Ok(Json(VerificationResponse {
        verified: true,
        error: None,
    }))
}

type ResponseResult<T> = anyhow::Result<T, AppError>;

#[derive(Serialize)]
pub(crate) struct VerificationResponse {
    pub(crate) verified: bool,
    pub(crate) error: Option<String>,
}

#[derive(Serialize)]
pub struct EPDTaskStatus {
    id: Uuid,
    state: TaskStatus,
}

impl IntoResponse for EPDTaskStatus {
    fn into_response(self) -> Response {
        let status = match self.state {
            TaskStatus::Submitted => StatusCode::ACCEPTED,
            _ => StatusCode::OK,
        };
        (status, Json(self)).into_response()
    }
}