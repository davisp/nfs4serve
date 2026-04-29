use crate::nfs::constants::NFS_OPAQUE_LIMIT;
use crate::nfs::types::{ClientId, ClientOwner, SequenceId, Verifier};
use crate::xdr::MaxLenBytes;

#[derive(Debug)]
pub struct Client {
    pub owner_id: MaxLenBytes<NFS_OPAQUE_LIMIT>,
    pub verifier: Verifier,
    pub client_id: ClientId,
    pub sequence: SequenceId,
    pub confirmed: bool,
}

impl Client {
    pub fn new(owner: ClientOwner, client_id: ClientId) -> Self {
        Self {
            owner_id: owner.owner_id,
            verifier: owner.verifier,
            client_id,
            sequence: 1,
            confirmed: false,
        }
    }
}
