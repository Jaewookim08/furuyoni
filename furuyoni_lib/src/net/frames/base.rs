use std::io::{Cursor};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::io::AsyncWriteExt;

use crate::net::frames::error::{ParseError, WriteError};

#[async_trait]
pub trait OutputFrame {
    async fn write_to(
        &self,
        writer: &mut (impl AsyncWriteExt + Unpin + Send),
    ) -> Result<(), WriteError>;
}

pub trait InputFrame {
    fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, ParseError>
    where
        Self: Sized;
}

pub trait Frame: OutputFrame + InputFrame {}
impl<T> Frame for T where T: OutputFrame + InputFrame {}

#[derive(Serialize, Deserialize, Debug)]
pub struct WithRequestId<T> {
    pub request_id: u32,
    pub data: T,
}
impl<T> WithRequestId<T> {
    pub fn new(request_id: u32, data: T) -> Self {
        Self { request_id, data }
    }
}

pub fn parse<T: for<'a> Deserialize<'a>>(src: &mut Cursor<&[u8]>) -> Result<T, ParseError> {
    let line = get_line(src)?.to_vec();
    let str = String::from_utf8(line)?;
    let deserialized = serde_json::from_str::<T>(&str)?;
    Ok(deserialized)
}

pub async fn write_serialized(
    writer: &mut (impl AsyncWriteExt + Unpin),
    data: impl Serialize,
) -> Result<(), WriteError> {
    let serialized = serde_json::to_string(&data)?;
    writer.write_all(serialized.as_bytes()).await?;
    writer.write_all(b"\r\n").await?;
    Ok(())
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], ParseError> {
    let len = src.get_ref().len();
    if len <= 1 {
        return Err(ParseError::Incomplete);
    }

    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second to last byte
    let end = len - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // We found a line, update the position to be *after* the \n
            src.set_position((i + 2) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(ParseError::Incomplete)
}
