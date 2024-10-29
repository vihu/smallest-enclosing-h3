use std::result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SmallestEnclosingH3Error {
    #[error("Invalid resolution: {0}")]
    InvalidResolution(String),
    #[error("Invalid radius: {0}")]
    InvalidRadius(String),
}

pub type Result<T> = result::Result<T, SmallestEnclosingH3Error>;
