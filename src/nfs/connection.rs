use std::net::SocketAddr;

use anyhow::{Context as _, Result, anyhow};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::FromPrimitive as _;
use tokio::net::TcpStream;

use crate::nfs::types::{ClientId, NfsFh, SessionId, StateId};
use crate::nfs::{AsNfsStatus, NfsOperation, NfsStatus};
use crate::rpc::{RpcConnection, RpcContext, RpcMessage};
use crate::xdr::{XdrDeserialize, XdrSerialize};

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
enum NfsProgram {
    Null = 0,
    Compound = 1,
    Invalid = 255,
}

pub struct NfsConnection {
    conn: RpcConnection,
}

impl NfsConnection {
    pub fn new(conn: TcpStream, addr: SocketAddr) -> Self {
        Self {
            conn: RpcConnection::new(conn, addr),
        }
    }

    pub async fn read(&mut self) -> Result<NfsRequest> {
        loop {
            let mut rpc = self
                .conn
                .read()
                .await
                .context("Error reading next RPC message.")?;

            if rpc.call().version != super::constants::VERSION {
                log::warn!(
                    "Client attempted an unsupported version of NFS: {} != {}",
                    rpc.call().version,
                    super::constants::VERSION
                );

                rpc.write(&RpcMessage::program_mismatch_reply(
                    rpc.xid(),
                    super::constants::VERSION,
                ))?;

                self.conn.send(rpc).await?;

                continue;
            }

            let prog = NfsProgram::from_u32(rpc.call().procedure)
                .unwrap_or(NfsProgram::Invalid);

            log::trace!("NFS program: {prog:?}");

            if matches!(prog, NfsProgram::Null) {
                rpc.write(&rpc.success())?;
                self.conn.send(rpc).await?;

                continue;
            }

            if matches!(prog, NfsProgram::Invalid) {
                rpc.write(&RpcMessage::procedure_unavailable_reply(rpc.xid()))?;
                self.conn.send(rpc).await?;

                continue;
            }

            assert!(
                matches!(prog, NfsProgram::Compound),
                "Invalid RPC program logic."
            );

            log::trace!("Reading NFS COMPOUND tag");

            let tag = rpc
                .read::<Vec<u8>>()
                .context("Error reading compound tag.")?;

            log::trace!("Reading COMPOUND minor version.");

            let version = rpc
                .read::<u32>()
                .context("Error reading compound minor version.")?;

            log::trace!("Minor version: {version}");

            if version != super::constants::VERSION_MINOR {
                rpc.write(&rpc.success())?;
                rpc.write(&NfsStatus::MinorVersionMismatch)?;
                rpc.write(&tag)?;
                rpc.write(&0u32)?;

                continue;
            }

            let num_ops = rpc
                .read::<u32>()
                .context("Error reading COMPOUND operation count.")?;

            log::trace!("COMPOUND ops: {num_ops}");

            // At this point we have accepted the message for processing so
            // we mark the RPC layer as successful. We're also pre-emptively
            // writing a status and response count. These will be overwritten
            // in the case of an error.
            rpc.write(&rpc.success())?;
            rpc.mark_header();
            rpc.write(&NfsStatus::Ok)?;
            rpc.write(&tag)?;
            rpc.write(&0u32)?;

            return Ok(NfsRequest::new(rpc, tag, num_ops));
        }
    }

    pub async fn send(&self, mut req: NfsRequest) -> Result<()> {
        let pos = req.rpc.writer().position();
        let res = req.rewrite_header();
        req.rpc.writer().set_position(pos);

        res?;

        self.conn
            .send(req.rpc)
            .await
            .context("Error sending NFS response.")
    }
}

pub struct NfsRequest {
    pub rpc: RpcContext,
    pub tag: Vec<u8>,
    pub client_id: Option<ClientId>,
    pub session_id: Option<SessionId>,
    pub current_fh: Option<NfsFh>,
    pub current_state_id: Option<StateId>,
    pub saved_fh: Option<NfsFh>,
    pub num_ops: usize,
    pub curr_op: usize,
    pub status: NfsStatus,
    pub replied: u32,
}

impl NfsRequest {
    fn new(rpc: RpcContext, tag: Vec<u8>, num_ops: u32) -> Self {
        Self {
            rpc,
            tag,
            client_id: None,
            session_id: None,
            current_fh: None,
            current_state_id: None,
            saved_fh: None,
            num_ops: num_ops as usize,
            curr_op: 0,
            status: NfsStatus::Ok,
            replied: 0,
        }
    }

    pub fn num_ops(&self) -> usize {
        self.num_ops
    }

    pub fn set_session(&mut self, client_id: ClientId, session_id: SessionId) {
        self.client_id = Some(client_id);
        self.session_id = Some(session_id);
    }

    pub fn clear_session(&mut self) {
        self.session_id = None;
    }

    pub fn next_op(&mut self) -> Result<NfsOperation> {
        if self.curr_op >= self.num_ops {
            return Err(anyhow!("This NFS message has already been consumed."));
        }
        self.curr_op += 1;
        self.read::<NfsOperation>()
            .context("Error reading COMPOUND operation number.")
    }

    pub fn read<T: XdrDeserialize>(&mut self) -> std::io::Result<T> {
        self.rpc.read::<T>()
    }

    pub fn ack(
        &mut self,
        op: NfsOperation,
        status: NfsStatus,
    ) -> std::io::Result<()> {
        self.status = status;
        self.replied += 1;
        self.rpc.write(&op)?;
        self.rpc.write(&status)
    }

    pub fn reply<T: XdrSerialize + AsNfsStatus>(
        &mut self,
        op: NfsOperation,
        val: &T,
    ) -> std::io::Result<()> {
        self.status = val.as_status();
        self.replied += 1;
        self.rpc.write(&op)?;
        if val.has_body() {
            self.rpc.write(&self.status)?;
            self.rpc.write(val)
        } else {
            self.rpc.write(&self.status)
        }
    }

    pub fn rewrite_header(&mut self) -> std::io::Result<()> {
        let header_pos = self.rpc.header_pos();
        let mut writer = self.rpc.writer();
        writer.set_position(header_pos);

        self.status.serialize(&mut writer)?;
        self.tag.serialize(&mut writer)?;
        self.replied.serialize(&mut writer)?;

        Ok(())
    }
}
