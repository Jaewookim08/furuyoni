use crate::net::frames;
use crate::net::frames::Frame;
use bytes::BytesMut;
use std::marker::PhantomData;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    FrameWriteError(frames::WriteError),
}

pub struct ConnectionWriter<TOutput>
where
    TOutput: Frame,
{
    stream: WriteHalf<TcpStream>,
    phantom: PhantomData<TOutput>,
}

impl From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Self {
        Self::IOError(io_error)
    }
}

impl From<frames::WriteError> for Error {
    fn from(err: frames::WriteError) -> Self {
        Self::FrameWriteError(err)
    }
}

impl<TOutput: Frame> ConnectionWriter<TOutput> {
    pub async fn write_frame(&mut self, frame: &TOutput) -> Result<(), Error> {
        frame.write_to(&mut self.stream).await?;
        self.stream.flush().await?;

        Ok(())
    }
}
