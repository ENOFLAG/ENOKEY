use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;

pub enum EnokeysError {
    IOError(std::io::Error),
    ReqwestError,
    InvalidData(String)
}

impl From<reqwest::Error> for EnokeysError {
    fn from(_: reqwest::Error) -> Self {
        unimplemented!()
    }
}

impl From<std::io::Error> for EnokeysError {
    fn from(_: std::io::Error) -> Self {
        unimplemented!()
    }
}

impl Debug for EnokeysError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        unimplemented!()
    }
}