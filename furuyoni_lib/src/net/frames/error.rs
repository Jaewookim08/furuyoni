use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum ParseError {
    /// Not enough data is available to parse a message
    Incomplete,
    /// Invalid message encoding
    InvalidMessage(InvalidMessage),
}

#[derive(Debug)]
pub struct InvalidMessage {
    pub err_str: String,
}
impl From<String> for InvalidMessage {
    fn from(str: String) -> Self {
        Self { err_str: str }
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(parse_error: serde_json::Error) -> Self {
        Self::InvalidMessage(InvalidMessage {
            err_str: parse_error.to_string(),
        })
    }
}

impl From<FromUtf8Error> for ParseError {
    fn from(err: FromUtf8Error) -> Self {
        Self::InvalidMessage(InvalidMessage {
            err_str: err.to_string(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Writing a frame failed.")]
pub enum WriteError {
    IOError(std::io::Error),
    SerializationError(serde_json::Error),
}

impl From<serde_json::Error> for WriteError {
    fn from(parse_error: serde_json::Error) -> Self {
        Self::SerializationError(parse_error)
    }
}
impl From<std::io::Error> for WriteError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}