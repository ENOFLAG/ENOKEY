#[derive(Debug)]
pub enum EnokeysError {
    IOError(std::io::Error),
    ReqwestError(reqwest::Error),
    InvalidData(String),
    InvalidProviderError(String)
}

impl From<reqwest::Error> for EnokeysError {
    fn from(error: reqwest::Error) -> Self {
        EnokeysError::ReqwestError(error)
    }
}

impl From<std::io::Error> for EnokeysError {
    fn from(error: std::io::Error) -> Self {
        EnokeysError::IOError(error)
    }
}
