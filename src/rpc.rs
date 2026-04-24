use std::io::{Read, Write};

use num_derive::{FromPrimitive, ToPrimitive};

use crate::xdr::{self, XdrSerde};

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
}
xdr::serde_enum!(AuthFlavor);

#[derive(Clone, Debug)]
pub struct AuthUnix {
    stamp: u32,
    machinename: Vec<u8>,
    uid: u32,
    gid: u32,
    gids: Vec<u32>,
}

impl XdrSerde for AuthUnix {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.stamp.serialize(dest)?;
        self.machinename.serialize(dest)?;
        self.uid.serialize(dest)?;
        self.gid.serialize(dest)?;
        self.gids.serialize(dest)?;
        Ok(())
    }

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
    flavor: AuthFlavor,
    body: Vec<u8>,
}

impl XdrSerde for OpaqueAuth {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.flavor.serialize(dest)?;
        self.body.serialize(dest)?;
        Ok(())
    }

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            flavor: AuthFlavor::deserialize(src)?,
            body: Vec::<u8>::deserialize(src)?,
        })
    }
}

#[derive(Debug)]
pub struct RpcMessage {
    xid: u32,
    body: RpcBody,
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

impl XdrSerde for RpcMessage {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.xid.serialize(dest)?;
        self.body.serialize(dest)?;
        Ok(())
    }

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

impl XdrSerde for RpcBody {
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
    rpc_version: u32,
    program: u32,
    version: u32,
    procedure: u32,
    credentials: OpaqueAuth,
    verifier: OpaqueAuth,
}

impl XdrSerde for RpcBodyCall {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.rpc_version.serialize(dest)?;
        self.program.serialize(dest)?;
        self.version.serialize(dest)?;
        self.procedure.serialize(dest)?;
        self.credentials.serialize(dest)?;
        self.verifier.serialize(dest)?;
        Ok(())
    }

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

impl XdrSerde for RpcBodyReply {
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
    verifier: OpaqueAuth,
    body: AcceptedBody,
}

impl XdrSerde for AcceptedReply {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.verifier.serialize(dest)?;
        self.body.serialize(dest)?;
        Ok(())
    }

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

impl XdrSerde for AcceptedBody {
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

impl XdrSerde for RejectedReply {
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

impl XdrSerde for MismatchInfo {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.low.serialize(dest)?;
        self.high.serialize(dest)?;
        Ok(())
    }

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            low: u32::deserialize(src)?,
            high: u32::deserialize(src)?,
        })
    }
}
