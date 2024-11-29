/*
 * Copyright (c) 2024 Thomas Preindl
 * MIT License (see LICENSE or https://mit-license.org)
 */

use crate::handlers::VerificationResponse;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::io;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    // #[error("EPD Zero Knowledge Proof missing")]
    // MissingZkProof,

    #[error("No Result available")]
    MissingResult,

    #[error("{0}")]
    ProofGenerationFailed(Arc<ProvingError>),

    #[error("Unknown Task: {0}")]
    UnknownTask(Uuid),

    #[error("Task is not complete")]
    TaskNotCompleted,

    #[error("Unable to decode Zero Knowledge Proof")]
    ProofDecodingFailed,

    #[error("EPD information does not match proof commitments!")]
    NonMatchingEPDInfo,

    #[error("Proof could not be verified!")]
    InvalidProof,

    #[error("Invalid Request data!")]
    InvalidRequestData(Arc<str>),
}

impl<A> From<A> for AppError
where
    A: Into<ProvingError>
{
    fn from(err: A) -> Self {
        Self::ProofGenerationFailed(Arc::new(err.into()))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            // AppError::MissingZkProof => StatusCode::BAD_REQUEST,
            AppError::MissingResult => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ProofGenerationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UnknownTask(_) => StatusCode::NOT_FOUND,
            AppError::TaskNotCompleted => StatusCode::BAD_REQUEST,
            AppError::ProofDecodingFailed => StatusCode::BAD_REQUEST,
            AppError::NonMatchingEPDInfo => StatusCode::OK,
            AppError::InvalidProof => StatusCode::OK,
            AppError::InvalidRequestData(_) => StatusCode::BAD_REQUEST,
        };
        if let StatusCode::OK = status {
            (
                status,
                Json(VerificationResponse {
                    verified: false,
                    error: Some(self.to_string()),
                })
            ).into_response()
        } else {
            (
                status,
                self.to_string()
            ).into_response()
        }
    }
}

#[derive(Error, Debug)]
pub enum ProvingError {
    #[error("Proof generation failed: {0}")]
    ProvingFailed(#[from] anyhow::Error),

    #[error("Proof output could not be deserialized.")]
    JournalDecodingFailed(#[from] risc0_zkvm::serde::Error),

    #[error("Proof could not be serialized.")]
    SerializationFailed(#[from] ciborium::ser::Error<io::Error>)
}