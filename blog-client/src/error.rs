use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),

    #[error("Transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
