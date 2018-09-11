extern crate openssl;
extern crate serde_json;

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ErrorKind {
    Unknown,
    Security,
    NotFound,
    InvalidJson,
}

#[derive(Debug)]
pub struct Error {
    message: Option<String>,
    kind: ErrorKind,
}

impl Error {
    pub fn empty(kind: ErrorKind) -> Self {
        Error {
            message: None,
            kind,
        }
    }

    pub fn debug(error: impl fmt::Debug, kind: ErrorKind) -> Self {
        Error {
            message: Some(format!("{:?}", error)),
            kind,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::NotFound {
            Error::empty(ErrorKind::NotFound)
        } else {
            Error {
                message: Some(format!("IO Error: {:?}", error)),
                kind: ErrorKind::Unknown,
            }
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::debug(error, ErrorKind::InvalidJson)
    }
}

impl From<openssl::error::ErrorStack> for Error {
    fn from(error: openssl::error::ErrorStack) -> Self {
        Error::debug(error, ErrorKind::Security)
    }
}
