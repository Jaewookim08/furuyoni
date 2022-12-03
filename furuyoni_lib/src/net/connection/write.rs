use crate::net::frames;
use crate::net::frames::Frame;
use bytes::BytesMut;
use std::marker::PhantomData;
use thiserror::Error;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;

#[derive(Error, Debug)]
#[error("Write failed.")]
pub enum Error {
    IOError(#[from] std::io::Error),
    FrameWriteError(#[from] frames::WriteError),
}

pub struct ConnectionWriter<TOutput>
where
    TOutput: Frame,
{
    stream: WriteHalf<TcpStream>,
    phantom: PhantomData<TOutput>,
}

impl<TOutput: Frame> ConnectionWriter<TOutput> {
    pub async fn write_frame(&mut self, frame: &TOutput) -> Result<(), Error> {
        frame.write_to(&mut self.stream).await?;
        self.stream.flush().await?;

        Ok(())
    }
}
