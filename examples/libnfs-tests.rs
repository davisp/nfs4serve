use std::sync::Arc;

use anyhow::{Context as _, Result, anyhow};
use nfs41server::{
    NFSv41Server, NfsAttribute, NfsAttributes, NfsExpirationPolicy, NfsFh,
    NfsFileSystemId, NfsFileType, NfsHandler, NfsLayoutType, NfsSpecData,
    NfsStatus, NfsTime,
};

// TODO: Make this change each time the process start.
const GENERATION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Inode(u64, u64);

impl From<Inode> for NfsFh {
    fn from(val: Inode) -> Self {
        let mut data = Vec::from(val.0.to_be_bytes());
        data.extend(val.1.to_be_bytes());
        NfsFh::new(data).expect("Somehow 8 bytes is bigger than 128 bytes.")
    }
}

impl TryFrom<NfsFh> for Inode {
    type Error = anyhow::Error;
    fn try_from(fh: NfsFh) -> Result<Self, Self::Error> {
        let data = Vec::from(fh);
        if data.len() != 16 {
            return Err(anyhow!("Invalid file handle: '{data:#?}'"));
        }

        let generation = u64::from_be_bytes((&data[..8]).try_into().unwrap());
        let inode = u64::from_be_bytes((&data[8..16]).try_into().unwrap());

        Ok(Self(generation, inode))
    }
}

pub struct MyHandler;

impl MyHandler {
    fn default_attributes() -> NfsAttributes {
        NfsAttributes {
            supported_attributes: vec![
                NfsAttribute::SupportedAttributes,
                NfsAttribute::FileType,
                NfsAttribute::ExpirationPolicy,
                NfsAttribute::Changed,
                NfsAttribute::Size,
                NfsAttribute::LinkSupport,
                NfsAttribute::SymlinkSupport,
                NfsAttribute::NamedAttributes,
                NfsAttribute::FileSystemId,
                NfsAttribute::UniqueHandles,
                NfsAttribute::LeaseTime,
                NfsAttribute::ReadAttributeError,
                NfsAttribute::CanSetTime,
                NfsAttribute::FileHandle,
                NfsAttribute::CaseInsensitive,
                NfsAttribute::CasePreserving,
                NfsAttribute::ChownRestricted,
                NfsAttribute::FileHandle,
                NfsAttribute::FileId,
                NfsAttribute::Homogeneous,
                NfsAttribute::MaxFileSize,
                NfsAttribute::MaxLinks,
                NfsAttribute::MaxReadLength,
                NfsAttribute::MaxWriteLength,
                NfsAttribute::Mode,
                NfsAttribute::NoTruncation,
                NfsAttribute::NumLinks,
                NfsAttribute::RawDevice,
                NfsAttribute::TimeAccess,
                NfsAttribute::SetTimeAccess,
                NfsAttribute::TimeCreate,
                NfsAttribute::TimeDelta,
                NfsAttribute::TimeModify,
                NfsAttribute::SetTimeModify,
                NfsAttribute::MountedOnFileId,
                NfsAttribute::FileSystemLayoutType,
                NfsAttribute::LayoutAlignment,
                NfsAttribute::LayoutBlockSize,
                NfsAttribute::ExclusiveCreateAttributes,
            ],
            expiration_policy: NfsExpirationPolicy::Persistent,
            link_support: false,
            symlink_support: false,
            named_attributes: false,
            file_system_id: NfsFileSystemId { major: 0, minor: 0 },
            unique_handles: true,
            lease_time: 90,
            can_set_time: Some(true),
            case_insensitive: Some(false),
            case_preserving: Some(true),
            chown_restricted: Some(true),
            homogeneous: Some(false),
            max_file_size: Some(u64::MAX),
            max_links: Some(1),
            max_read_length: Some(0x80000),
            max_write_length: Some(0x80000),
            no_truncation: Some(true),
            num_links: Some(1),
            raw_device: Some(NfsSpecData {
                specdata1: 0,
                specdata2: 0,
            }),
            time_delta: Some(NfsTime {
                seconds: 0,
                nseconds: 4_000_000,
            }),
            mounted_on_file_id: Some(1),
            file_system_layout_type: Some(vec![NfsLayoutType::NfsV41Files]),
            layout_alignment: Some(4096),
            layout_block_size: Some(4096),
            exclusive_create_attributes: vec![
                NfsAttribute::Size,
                NfsAttribute::Mode,
                NfsAttribute::TimeCreate,
            ],
            ..Default::default()
        }
    }
}

#[async_trait::async_trait]
impl NfsHandler for MyHandler {
    fn root_fh(&self) -> NfsFh {
        Inode(GENERATION, 1).into()
    }

    fn public_fh(&self) -> NfsFh {
        self.root_fh()
    }

    async fn get_attributes(
        &self,
        fh: &NfsFh,
        _attributes: &[NfsAttribute],
    ) -> Result<NfsAttributes, NfsStatus> {
        #[expect(clippy::map_err_ignore, reason = "That's what I'm doing.")]
        let fh = Inode::try_from(fh.clone())
            .context("Invalid file handle.")
            .map_err(|_| NfsStatus::EINVAL)?;

        if fh.0 != GENERATION {
            return Err(NfsStatus::ESTALE);
        }

        let mut vals = Self::default_attributes();

        // Root file handle is special.
        if fh.1 == 1 {
            vals.file_type = NfsFileType::Directory;
            vals.changed = 0;
            vals.size = 4096;
            vals.file_handle = fh.into();
            vals.file_id = Some(fh.1);
            vals.mode = Some(0x0766);
        } else {
            todo!();
        }

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
