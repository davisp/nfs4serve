use std::sync::Arc;

use anyhow::{Context as _, Result};
use nfs41server::{
    NFSv41Server, NfsAttribute, NfsAttributeValue, NfsExpirationPolicy, NfsFh,
    NfsFileSystemId, NfsHandleType, NfsHandler,
};

// TODO: Make this change each time the process start.
const GENERATION: u64 = 1;

#[derive(Clone, Copy, Debug)]
pub struct Inode(u64, u64);

impl From<Inode> for NfsFh {
    fn from(val: Inode) -> Self {
        let mut data = Vec::from(val.0.to_be_bytes());
        data.extend(val.1.to_be_bytes());
        NfsFh::new(data).expect("Somehow 8 bytes is bigger than 128 bytes.")
    }
}

pub struct MyHandler;

#[async_trait::async_trait]
impl NfsHandler for MyHandler {
    fn root_fh(&self) -> NfsFh {
        Inode(GENERATION, 0).into()
    }

    fn public_fh(&self) -> NfsFh {
        self.root_fh()
    }

    async fn get_attributes(
        &self,
        fh: &NfsFh,
        attributes: &[NfsAttribute],
    ) -> Result<Vec<NfsAttributeValue>> {
        let vals = attributes
            .iter()
            .map(|attr| match attr {
                NfsAttribute::SupportedAttributes => {
                    NfsAttributeValue::SupportedAttributes(
                        NfsAttribute::required().collect::<Vec<_>>(),
                    )
                }
                NfsAttribute::HandleType => {
                    // In real life, we'd look this up keyed by the `fh`
                    // argument.
                    NfsAttributeValue::HandleType(NfsHandleType::Regular)
                }
                NfsAttribute::ExpirationPolicy => {
                    NfsAttributeValue::ExpirationPolicy(
                        NfsExpirationPolicy::Persistent,
                    )
                }
                NfsAttribute::Changed => {
                    // For realsies, we'd return the last file or directory
                    // modification time or some other indicator.
                    NfsAttributeValue::Changed(0)
                }
                NfsAttribute::Size => {
                    // Another fake for testing. Normally we'd track this
                    // somewhere.
                    NfsAttributeValue::Size(0)
                }
                NfsAttribute::LinkSupport => {
                    NfsAttributeValue::LinkSupport(false)
                }
                NfsAttribute::SymlinkSupport => {
                    NfsAttributeValue::SymlinkSupport(false)
                }
                NfsAttribute::NamedAttributes => {
                    NfsAttributeValue::NamedAttributes(false)
                }
                NfsAttribute::FileSystemId => {
                    NfsAttributeValue::FileSystemId(NfsFileSystemId {
                        major: 42,
                        minor: 314_159,
                    })
                }
                NfsAttribute::UniqueHandles => {
                    // This will very much depend on the use case.
                    NfsAttributeValue::UniqueHandles(false)
                }
                NfsAttribute::LeaseTime => {
                    // I'm not completely certain if I should expose this or
                    // make it a config value somewhere.
                    NfsAttributeValue::LeaseTime(30)
                }
                NfsAttribute::ReadAttributeError => {
                    unreachable!("Ope, this is no good.")
                }
                NfsAttribute::FileHandle => {
                    NfsAttributeValue::FileHandle(fh.clone())
                }
                NfsAttribute::ExclusiveCreateAttributes => {
                    NfsAttributeValue::ExclusiveCreateAttributes(
                        NfsAttribute::required().collect::<Vec<_>>(),
                    )
                }
            })
            .collect::<Vec<_>>();

        Ok(vals)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    stderrlog::new().verbosity(4).init().unwrap();

    let handler = Arc::new(MyHandler {});

    let server = NFSv41Server::new(handler, "127.0.0.1:9342")
        .context("Error creating server.")?;
    server.serve().await.context("The main server loop died.")
}
