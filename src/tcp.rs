use std::io::Cursor;
use std::net::SocketAddr;

use anyhow::{Context as _, Result, anyhow};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::xdr::{XdrDeserialize as _, XdrSerialize as _};

const RPC_LAST_FRAME: u32 = 0x80_00_00_00;
const RPC_FRAME_LEN: u32 = 0x7F_FF_FF_FF;

#[derive(Debug)]
pub struct TcpConnection {
    address: SocketAddr,
    reader: ConnectionReader,
    writer: ConnectionWriter,
}

impl TcpConnection {
    pub fn new(socket: TcpStream, address: SocketAddr) -> Self {
        let (reader, writer) = socket.into_split();

        Self {
            address,
            reader: ConnectionReader::new(reader),
            writer: ConnectionWriter::new(writer),
        }
    }

    pub async fn read(&mut self) -> Result<Vec<u8>> {
        let Some(frame) = self.reader.recv().await else {
            return Err(anyhow!("Client disconnected: {:?}", self.address));
        };

        Ok(frame)
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<()> {
        self.writer.send(data).await
    }
}

#[derive(Debug)]
pub struct ConnectionReader {
    join: JoinHandle<Result<()>>,
    rx: Receiver<Vec<u8>>,
}

impl ConnectionReader {
    fn new(reader: OwnedReadHalf) -> Self {
        let (tx, rx) = mpsc::channel::<Vec<u8>>(1024);
        Self {
            join: tokio::spawn(Self::run(reader, tx)),
            rx,
        }
    }

    async fn recv(&mut self) -> Option<Vec<u8>> {
        self.rx.recv().await
    }

    async fn run(mut reader: OwnedReadHalf, tx: Sender<Vec<u8>>) -> Result<()> {
        let mut header_buf = [0u8; 4];
        let mut frame = Vec::new();
        loop {
            reader
                .read_exact(&mut header_buf)
                .await
                .context("Error reading from client socket.")?;

            let mut rbuf = Cursor::new(&mut header_buf[..]);
            let header = u32::deserialize(&mut rbuf)
                .context("Error parsing frame length sent by client.")?;

            let is_last = header & RPC_LAST_FRAME != 0;
            let size = (header & RPC_FRAME_LEN) as usize;

            let read_offset = frame.len();
            frame.resize(read_offset + size, 0);

            // TODO: Create some sort of "frame buffer" (heh) where we can
            //       fetch existing allocations to reuse rather than constantly
            //       allocating new vec's for each frame.
            reader
                .read_exact(&mut frame[read_offset..])
                .await
                .context("Error reading frame data from client.")?;

            if is_last {
                let frame = std::mem::take(&mut frame);
                match tx.send(frame).await {
                    Ok(()) => (),
                    Err(_) => {
                        // The connection has died so it's time for us to exit.
                        break Ok(());
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ConnectionWriter {
    join: JoinHandle<Result<()>>,
    tx: Sender<Vec<u8>>,
}

impl ConnectionWriter {
    fn new(writer: OwnedWriteHalf) -> Self {
        let (tx, rx) = mpsc::channel::<Vec<u8>>(1024);
        Self {
            join: tokio::spawn(Self::run(writer, rx)),
            tx,
        }
    }

    async fn send(&self, frame: Vec<u8>) -> Result<()> {
        match self.tx.send(frame).await {
            Ok(()) => Ok(()),
            Err(_) => Err(anyhow!("Client write channel died.")),
        }
    }

    async fn run(
        mut writer: OwnedWriteHalf,
        mut rx: Receiver<Vec<u8>>,
    ) -> Result<()> {
        let mut length = [0u8; 4];
        loop {
            let Some(frame) = rx.recv().await else {
                // Our last frame has been sent, time to exit.
                break;
            };

            assert!(
                frame.len() < RPC_LAST_FRAME as usize,
                "TODO: Update to follow RFC 1057 Section 10."
            );

            #[expect(
                clippy::cast_possible_truncation,
                reason = "See the above assertion."
            )]
            let header = (frame.len() as u32) | RPC_LAST_FRAME;
            let mut w = Cursor::new(&mut length[..]);
            (header)
                .serialize(&mut w)
                .context("Error encoding frame length for client.")?;

            writer
                .write_all(&length[..])
                .await
                .context("Error sending frame size to the client.")?;

            writer
                .write_all(&frame)
                .await
                .context("Error sending frame data to the client.")?;
        }

        Ok(())
    }
}
