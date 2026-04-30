use anyhow::Result;

mod attributes;

pub use attributes::*;

pub use super::NfsStatus;
pub use super::types::NfsFh;

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
    /// Note, if for some reason the server is currently incapable of returning
    /// one of the supported attributes in this request (i.e., a network
    /// connection died) then no attribute values should be returned.
    async fn get_attributes(
        &self,
        fh: &NfsFh,
        attributes: &[NfsAttribute],
    ) -> Result<Vec<NfsAttributeValue>>;
}
