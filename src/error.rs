use std::result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmallestEnclosingH3Error {
    #[error("Invalid lat lng: {0}")]
    InvalidLatLng(String),
    #[error("Invalid resolution: {0}")]
    InvalidResolution(String),
    #[error("Invalid radius: {0}")]
    InvalidRadius(String),
    #[error("Grid distance error: {0}")]
    GridDistanceError(String),
    #[error("Grid Ring error: {0}")]
    GridRingError(String),
}

pub type Result<T> = result::Result<T, SmallestEnclosingH3Error>;
