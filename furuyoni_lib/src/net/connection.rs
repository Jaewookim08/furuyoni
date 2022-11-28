use crate::net::frames;
use crate::net::frames::Frame;
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use std::marker::PhantomData;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

pub struct Connection<TOutput, TInput>
where
    TOutput: Frame,
    TInput: Frame,
{
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    phantom_1: PhantomData<TOutput>,
    phantom_2: PhantomData<TInput>,
}

#[derive(Debug)]
pub enum ReadError {
    IOError(std::io::Error),
    ParseError(ParseError),
    ConnectionClosed(ConnectionClosed),
}

#[derive(Debug)]
pub enum WriteError {
    IOError(std::io::Error),
    FrameWriteError(frames::WriteError),
}

#[derive(Debug)]
pub struct ConnectionClosed {
    is_clean_shutdown: bool,
}

#[derive(Debug)]
pub struct ParseError {
    error_str: String,
}

impl From<std::io::Error> for ReadError {
    fn from(io_error: std::io::Error) -> Self {
        Self::IOError(io_error)
    }
}

impl From<std::io::Error> for WriteError {
    fn from(io_error: std::io::Error) -> Self {
        Self::IOError(io_error)
    }
}

impl From<frames::WriteError> for WriteError {
    fn from(err: frames::WriteError) -> Self {
        Self::FrameWriteError(err)
    }
}

impl<TOutput: Frame + Sync, TInput: Frame + Sync> Connection<TOutput, TInput> {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
            phantom_1: Default::default(),
            phantom_2: Default::default(),
        }
    }

    pub async fn read_frame(&mut self) -> Result<TInput, ReadError> {
        loop {
            // Attempt to parse a frame from the buffered data. If
            // enough data has been buffered, the frame is
            // returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(frame);
            }

            // There is not enough buffered data to read a frame.
            // Attempt to read more data from the socket.
            //
            // On success, the number of bytes is returned. `0`
            // indicates "end of stream".
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be
                // a clean shutdown, there should be no data in the
                // read buffer. If there is, this means that the
                // peer closed the socket while sending a frame.
                return Err(ReadError::ConnectionClosed(ConnectionClosed {
                    is_clean_shutdown: self.buffer.is_empty(),
                }));
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<TInput>, ReadError> {
        // Create the `T: Buf` type.
        let mut buf = Cursor::new(&self.buffer[..]);

        match TInput::parse(&mut buf) {
            Ok(frame) => {
                self.buffer.advance(buf.position() as usize);
                Ok(Some(frame))
            }
            Err(frames::ParseError::Incomplete) => Ok(None),
            Err(frames::ParseError::InvalidMessage(invalid)) => {
                Err(ReadError::ParseError(ParseError {
                    error_str: invalid.err_str,
                }))
            }
        }
    }

    async fn write_frame(&mut self, frame: &TOutput) -> Result<(), WriteError> {
        frame.write_to(&mut self.stream).await?;
        self.stream.flush().await?;

        Ok(())
    }
}
