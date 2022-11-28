use bytes::{Buf, BytesMut};
use furuyoni_lib::messages::{GameMessageFrame, PlayerMessageFrame};
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub struct GameConnection {
    stream: TcpStream,
    buffer: BytesMut,
}

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    ParseError(ParseError),
    ConnectionClosed(ConnectionClosed),
}

#[derive(Debug)]
pub struct ConnectionClosed {
    is_clean_shutdown: bool,
}

#[derive(Debug)]
pub struct ParseError {
    error_str: String,
}

impl From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Self {
        Self::IOError(io_error)
    }
}

impl GameConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_frame(&mut self) -> Result<PlayerMessageFrame, Error> {
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
                return Err(Error::ConnectionClosed(ConnectionClosed {
                    is_clean_shutdown: self.buffer.is_empty(),
                }));
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<PlayerMessageFrame>, Error> {
        // Create the `T: Buf` type.
        let mut buf = Cursor::new(&self.buffer[..]);

        match PlayerMessageFrame::parse(&mut buf) {
            Ok(frame) => {
                self.buffer.advance(buf.position() as usize);
                Ok(Some(frame))
            }
            Err(furuyoni_lib::messages::Error::Incomplete) => Ok(None),
            Err(furuyoni_lib::messages::Error::InvalidMessage(invalid)) => {
                Err(Error::ParseError(ParseError {
                    error_str: invalid.err_str,
                }))
            }
        }
    }
}
