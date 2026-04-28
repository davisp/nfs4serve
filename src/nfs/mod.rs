use anyhow::{Context as _, Result};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::FromPrimitive as _;

use crate::rpc::{AuthUnix, RpcBodyCall, RpcHandler, RpcMessage};

mod constants;
mod handlers;
mod ops;
mod status;
mod types;

pub use constants::*;
pub use ops::NfsOperation;
pub use status::NfsStatus;

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum NfsProgram {
    Null = 0,
    Compound = 1,
    Invalid = 255,
}

pub trait AsNfsStatus {
    fn as_status(&self) -> NfsStatus;
}

pub fn handle(
    rpc: &mut RpcHandler,
    call: &RpcBodyCall,
    _auth: Option<AuthUnix>,
) -> Result<()> {
    if call.version != VERSION {
        log::warn!(
            "Client attempted an unsupported version of NFS: {} != {VERSION}",
            call.version
        );
        rpc.write(&RpcMessage::program_mismatch_reply(rpc.xid(), VERSION))?;
        return Ok(());
    }

    let prog =
        NfsProgram::from_u32(call.procedure).unwrap_or(NfsProgram::Invalid);

    log::info!("NFS program: {prog:?}");

    match prog {
        NfsProgram::Null => {
            rpc.write(&rpc.success())?;
            Ok(())
        }
        NfsProgram::Compound => {
            let tag = rpc
                .read::<Vec<u8>>()
                .context("Error reading compound tag.")?;
            let version = rpc
                .read::<u32>()
                .context("Error reading compound minor version.")?;

            if version != VERSION_MINOR {
                rpc.write(&rpc.success())?;
                rpc.write(&NfsStatus::MinorVersionMismatch)?;
                rpc.write(&tag)?;
                rpc.write(&0u32)?;

                return Ok(());
            }

            todo!();
        }
        NfsProgram::Invalid => {
            rpc.write(&RpcMessage::procedure_unavailable_reply(rpc.xid()))?;
            Ok(())
        }
    }
}
