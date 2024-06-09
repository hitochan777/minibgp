use thiserror::Error;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct ConfigParseError {
    #[from]
    source: anyhow::Error,
}

#[derive(Error, Debug)]
#[error(transparent)] // TODO: check what this does
pub struct CreateConnectionError {
    #[from] // TODO: check what this does
    source: anyhow::Error,
}
