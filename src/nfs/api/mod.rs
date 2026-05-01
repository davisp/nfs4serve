use anyhow::Result;

mod attributes;

pub use attributes::*;

pub use super::NfsStatus;
pub use super::types::{
    NfsAce, NfsAceFlag, NfsAceMask, NfsAceType, NfsAclFlag, NfsChangePolicy,
    NfsExpirationPolicy, NfsFh, NfsFileSystemId, NfsFileType, NfsFsLocation,
    NfsFsLocations, NfsFsLocationsInfo, NfsFsLocationsItem,
    NfsFsLocationsServer, NfsFsStatus, NfsLayoutHint, NfsLayoutType,
    NfsModeMasked, NfsPathName, NfsSpecData, NfsTime,
};

// Rexport to avoid forcing the dependency.
pub use num_traits::cast::{FromPrimitive, ToPrimitive};

/// The core `NFSHandler` trait.
///
/// Implementing this trait should allo you to mount your handler as a file
/// system using `NFSv4.1Server`.
#[async_trait::async_trait]
pub trait NfsHandler: Send + Sync {
    /// Return the root filehandle of the filesystem.
    fn root_fh(&self) -> NfsFh;

    /// Return the public file handle of the filesystem.
    ///
    /// Reading the documentation, I'm not 100% certain on when we'd have a
    /// different root than public filehandle. Something to do with multiple
    /// exported shares is about all I know. For now, I'll just return the
    /// root filehandle since they're allowed to be the same.
    fn public_fh(&self) -> NfsFh;

    /// Get the requested file attributes.
    ///
    /// The `attributes` argument is the list of requested attributes. If some
    /// attributes are expensive to calculate, you can check if the attribute
    /// has been requested or not.
    async fn get_attributes(
        &self,
        fh: &NfsFh,
        attributes: &[NfsAttribute],
    ) -> Result<NfsAttributes, NfsStatus>;
}
