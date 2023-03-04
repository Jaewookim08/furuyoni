use crate::net::frames;
use crate::net::frames::Frame;

use std::marker::PhantomData;
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};


#[derive(Error, Debug)]
#[error("Write failed.")]
pub enum Error {
    IOError(#[from] std::io::Error),
    FrameWriteError(#[from] frames::WriteError),
}

pub struct ConnectionWriter<TWrite, TOutput>
where
    TOutput: Frame,
    TWrite: AsyncWrite + Unpin + Send,
{
    stream: TWrite,
    phantom: PhantomData<TOutput>,
}

impl<TWrite: AsyncWrite + Unpin + Send, TOutput: Frame> ConnectionWriter<TWrite, TOutput> {
    pub fn new(stream: TWrite) -> Self {
        Self {
            stream,
            phantom: Default::default(),
        }
    }

    pub async fn write_frame(&mut self, frame: &TOutput) -> Result<(), Error> {
        frame.write_to(&mut self.stream).await?;
        self.stream.flush().await?;

        Ok(())
    }
}
