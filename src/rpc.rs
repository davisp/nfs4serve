use std::io::{Cursor, Read, Write};
use std::net::SocketAddr;

use anyhow::{Context as _, Result, anyhow};
use num_derive::{FromPrimitive, ToPrimitive};
use tokio::net::TcpStream;

use crate::nfs;
use crate::tcp::TcpConnection;
use crate::xdr::{self, XdrDeserialize, XdrSerialize};

pub struct RpcConnection {
    conn: TcpConnection,
}

impl RpcConnection {
    pub fn new(conn: TcpStream, addr: SocketAddr) -> Self {
        Self {
            conn: TcpConnection::new(conn, addr),
        }
    }

    pub async fn read(&mut self) -> Result<RpcContext> {
        loop {
            let frame = self
                .conn
                .read()
                .await
                .context("Error reading RPC Framed Message.")?;

            let mut reader = Cursor::new(frame);

            let mesg = RpcMessage::deserialize(&mut reader)
                .context("Error decoding base RPC message.")?;

            let call = match mesg.body {
                RpcBody::Call(call) => call,
                RpcBody::Reply(mesg) => {
                    log::error!(
                        "Client sent an RPC Reply instead of a Call: {mesg:#?}",
                    );
                    return Err(anyhow!(
                        "Bad RPC Reply message received from client."
                    ));
                }
            };

            let auth = if matches!(call.credentials.flavor, AuthFlavor::Unix) {
                Some(
                    AuthUnix::deserialize(&mut Cursor::new(
                        &call.credentials.body,
                    ))
                    .context(
                        "Error deserialize unix authentication in call body.",
                    )?,
                )
            } else {
                None
            };

            let mut ctx = RpcContext::new(mesg.xid, call, auth, reader);

            if ctx.call.rpc_version != 2 {
                log::warn!(
                    "Invalid RPC version sent by client: {} != 2",
                    ctx.call.rpc_version
                );

                ctx.write(&RpcMessage::rpc_version_mismatch_reply(ctx.xid))?;
                self.send(ctx).await?;
                continue;
            }

            log::debug!("RPC MESSAGE: {:#?}", ctx.call);

            if ctx.call.program == nfs::PROGRAM {
                return Ok(ctx);
            }

            if ctx.call.program == nfs::PROGRAM_ACL
                || ctx.call.program == nfs::PROGRAM_ID_MAP
                || ctx.call.program == nfs::PROGRAM_METADATA
            {
                log::trace!("Ignoring NFS ACL RPC calls: {:?}", mesg.xid);
            } else {
                log::warn!("Unknown RPC program number: {}", ctx.call.program);
            }

            ctx.write(&RpcMessage::program_unavailable_reply(ctx.xid))?;
            self.send(ctx).await?;
        }
    }

    pub async fn send(&self, ctx: RpcContext) -> Result<()> {
        let data = ctx.writer.into_inner();
        self.conn
            .send(data)
            .await
            .context("Error sending RPC response.")
    }
}

pub struct RpcContext {
    xid: u32,
    call: RpcBodyCall,
    auth: Option<AuthUnix>,
    reader: Cursor<Vec<u8>>,
    writer: Cursor<Vec<u8>>,
    header_pos: u64,
}

impl RpcContext {
    fn new(
        xid: u32,
        call: RpcBodyCall,
        auth: Option<AuthUnix>,
        reader: Cursor<Vec<u8>>,
    ) -> Self {
        Self {
            xid,
            call,
            auth,
            reader,
            writer: Cursor::new(Vec::new()),
            header_pos: 0,
        }
    }

    pub fn xid(&self) -> u32 {
        self.xid
    }

    pub fn call(&self) -> &RpcBodyCall {
        &self.call
    }

    pub fn read<T: XdrDeserialize>(&mut self) -> std::io::Result<T> {
        T::deserialize(&mut self.reader)
    }

    pub fn write<T: XdrSerialize>(&mut self, val: &T) -> std::io::Result<()> {
        val.serialize(&mut self.writer)
    }

    pub fn success(&self) -> RpcMessage {
        RpcMessage::successful_reply(self.xid)
    }

    pub fn mark_header(&mut self) {
        self.header_pos = self.writer.position();
    }

    pub fn header_pos(&self) -> u64 {
        self.header_pos
    }

    /// Access to the underlying writer.
    ///
    /// This is used by the NFS layer to backup and overwrite the header
    /// when an error is encountered.
    pub fn writer(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.writer
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum MessageType {
    Call = 0,
    Reply = 1,
}
xdr::serde_enum!(MessageType);

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum ReplyState {
    Accepted = 0,
    Denied = 1,
}
xdr::serde_enum!(ReplyState);

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum AcceptState {
    Success = 0,
    ProgramUnavailable = 1,
    ProgramMismatch = 2,
    ProcedureUnavailable = 3,
    GarbageArguments = 4,
}
xdr::serde_enum!(AcceptState);

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum RejectState {
    RpcMismatch = 0,
    AuthError = 1,
}
xdr::serde_enum!(RejectState);

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum AuthState {
    BadCredentials = 0,
    RejectedCredentials = 1,
    BadVerifier = 2,
    RejectedVerifier = 4,
    TooWeak = 5,
}
xdr::serde_enum!(AuthState);

#[derive(Clone, Copy, Debug, Default, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum AuthFlavor {
    #[default]
    Null = 0,
    Unix = 1,
    Short = 2,
    Des = 3,
    RpcSecGss = 6,
}
xdr::serde_enum!(AuthFlavor);

#[derive(Clone, Debug)]
pub struct AuthUnix {
    pub stamp: u32,
    pub machinename: Vec<u8>,
    pub uid: u32,
    pub gid: u32,
    pub gids: Vec<u32>,
}

impl XdrSerialize for AuthUnix {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.stamp.serialize(dest)?;
        self.machinename.serialize(dest)?;
        self.uid.serialize(dest)?;
        self.gid.serialize(dest)?;
        self.gids.serialize(dest)?;
        Ok(())
    }
}

impl XdrDeserialize for AuthUnix {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            stamp: u32::deserialize(src)?,
            machinename: Vec::<u8>::deserialize(src)?,
            uid: u32::deserialize(src)?,
            gid: u32::deserialize(src)?,
            gids: Vec::<u32>::deserialize(src)?,
        })
    }
}

#[derive(Debug, Default)]
pub struct OpaqueAuth {
    pub flavor: AuthFlavor,
    pub body: Vec<u8>,
}

impl XdrSerialize for OpaqueAuth {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.flavor.serialize(dest)?;
        self.body.serialize(dest)?;
        Ok(())
    }
}

impl XdrDeserialize for OpaqueAuth {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            flavor: AuthFlavor::deserialize(src)?,
            body: Vec::<u8>::deserialize(src)?,
        })
    }
}

#[derive(Debug)]
pub struct RpcMessage {
    pub xid: u32,
    pub body: RpcBody,
}

impl RpcMessage {
    pub fn successful_reply(xid: u32) -> Self {
        Self::accepted_reply(xid, AcceptedBody::Success)
    }

    pub fn procedure_unavailable_reply(xid: u32) -> Self {
        Self::accepted_reply(xid, AcceptedBody::ProcedureUnavailable)
    }

    pub fn program_unavailable_reply(xid: u32) -> Self {
        Self::accepted_reply(xid, AcceptedBody::ProgramUnavailable)
    }

    pub fn program_mismatch_reply(xid: u32, accepted_version: u32) -> Self {
        Self::accepted_reply(
            xid,
            AcceptedBody::ProgramMismatch(MismatchInfo {
                low: accepted_version,
                high: accepted_version,
            }),
        )
    }

    pub fn garbage_arguments_reply(xid: u32) -> Self {
        Self::accepted_reply(xid, AcceptedBody::GarbageArguments)
    }

    pub fn rpc_version_mismatch_reply(xid: u32) -> Self {
        let reply = RejectedReply::RpcMismatch(MismatchInfo::default());
        Self {
            xid,
            body: RpcBody::Reply(RpcBodyReply::Rejected(reply)),
        }
    }

    fn accepted_reply(xid: u32, reply: AcceptedBody) -> Self {
        let reply = AcceptedReply {
            verifier: OpaqueAuth::default(),
            body: reply,
        };
        let reply = RpcBodyReply::Accepted(reply);
        Self::reply(xid, reply)
    }

    fn reply(xid: u32, msg: RpcBodyReply) -> Self {
        Self {
            xid,
            body: RpcBody::Reply(msg),
        }
    }
}

impl XdrSerialize for RpcMessage {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.xid.serialize(dest)?;
        self.body.serialize(dest)?;
        Ok(())
    }
}

impl XdrDeserialize for RpcMessage {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            xid: u32::deserialize(src)?,
            body: RpcBody::deserialize(src)?,
        })
    }
}

#[derive(Debug)]
pub enum RpcBody {
    Call(RpcBodyCall),
    Reply(RpcBodyReply),
}

impl XdrSerialize for RpcBody {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Call(body) => {
                0u32.serialize(dest)?;
                body.serialize(dest)?;
            }
            Self::Reply(body) => {
                1u32.serialize(dest)?;
                body.serialize(dest)?;
            }
        }
        Ok(())
    }
}

impl XdrDeserialize for RpcBody {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        match u32::deserialize(src)? {
            0 => Ok(Self::Call(RpcBodyCall::deserialize(src)?)),
            1 => Ok(Self::Reply(RpcBodyReply::deserialize(src)?)),
            n => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid tag for RpcBody: '{n}'"),
            )),
        }
    }
}

#[derive(Debug)]
pub struct RpcBodyCall {
    pub rpc_version: u32,
    pub program: u32,
    pub version: u32,
    pub procedure: u32,
    pub credentials: OpaqueAuth,
    pub verifier: OpaqueAuth,
}

impl XdrSerialize for RpcBodyCall {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.rpc_version.serialize(dest)?;
        self.program.serialize(dest)?;
        self.version.serialize(dest)?;
        self.procedure.serialize(dest)?;
        self.credentials.serialize(dest)?;
        self.verifier.serialize(dest)?;
        Ok(())
    }
}

impl XdrDeserialize for RpcBodyCall {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            rpc_version: u32::deserialize(src)?,
            program: u32::deserialize(src)?,
            version: u32::deserialize(src)?,
            procedure: u32::deserialize(src)?,
            credentials: OpaqueAuth::deserialize(src)?,
            verifier: OpaqueAuth::deserialize(src)?,
        })
    }
}

#[derive(Debug)]
pub enum RpcBodyReply {
    Accepted(AcceptedReply),
    Rejected(RejectedReply),
}

impl XdrSerialize for RpcBodyReply {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Accepted(body) => {
                0u32.serialize(dest)?;
                body.serialize(dest)?;
            }
            Self::Rejected(body) => {
                1u32.serialize(dest)?;
                body.serialize(dest)?;
            }
        }
        Ok(())
    }
}

impl XdrDeserialize for RpcBodyReply {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        match u32::deserialize(src)? {
            0 => Ok(Self::Accepted(AcceptedReply::deserialize(src)?)),
            1 => Ok(Self::Rejected(RejectedReply::deserialize(src)?)),
            n => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid tag for RpcBodyReply: '{n}'"),
            )),
        }
    }
}

#[derive(Debug)]
pub struct AcceptedReply {
    pub verifier: OpaqueAuth,
    pub body: AcceptedBody,
}

impl XdrSerialize for AcceptedReply {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.verifier.serialize(dest)?;
        self.body.serialize(dest)?;
        Ok(())
    }
}

impl XdrDeserialize for AcceptedReply {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            verifier: OpaqueAuth::deserialize(src)?,
            body: AcceptedBody::deserialize(src)?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum AcceptedBody {
    Success,
    ProgramUnavailable,
    ProgramMismatch(MismatchInfo),
    ProcedureUnavailable,
    GarbageArguments,
}

impl XdrSerialize for AcceptedBody {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Success => {
                0u32.serialize(dest)?;
            }
            Self::ProgramUnavailable => {
                1u32.serialize(dest)?;
            }
            Self::ProgramMismatch(info) => {
                2u32.serialize(dest)?;
                info.serialize(dest)?;
            }
            Self::ProcedureUnavailable => {
                3u32.serialize(dest)?;
            }
            Self::GarbageArguments => {
                4u32.serialize(dest)?;
            }
        }
        Ok(())
    }
}

impl XdrDeserialize for AcceptedBody {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        match u32::deserialize(src)? {
            0 => Ok(Self::Success),
            1 => Ok(Self::ProgramUnavailable),
            2 => Ok(Self::ProgramMismatch(MismatchInfo::deserialize(src)?)),
            3 => Ok(Self::ProcedureUnavailable),
            4 => Ok(Self::GarbageArguments),
            n => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid tag for AcceptedBody: '{n}'"),
            )),
        }
    }
}

#[derive(Debug)]
pub enum RejectedReply {
    RpcMismatch(MismatchInfo),
    AuthError(AuthState),
}

impl XdrSerialize for RejectedReply {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::RpcMismatch(body) => {
                0u32.serialize(dest)?;
                body.serialize(dest)?;
            }
            Self::AuthError(body) => {
                1u32.serialize(dest)?;
                body.serialize(dest)?;
            }
        }
        Ok(())
    }
}

impl XdrDeserialize for RejectedReply {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        match u32::deserialize(src)? {
            0 => Ok(Self::RpcMismatch(MismatchInfo::deserialize(src)?)),
            1 => Ok(Self::AuthError(AuthState::deserialize(src)?)),
            n => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid tag for RejectedReply: '{n}'"),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MismatchInfo {
    pub low: u32,
    pub high: u32,
}

impl XdrSerialize for MismatchInfo {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.low.serialize(dest)?;
        self.high.serialize(dest)?;
        Ok(())
    }
}

impl XdrDeserialize for MismatchInfo {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            low: u32::deserialize(src)?,
            high: u32::deserialize(src)?,
        })
    }
}
