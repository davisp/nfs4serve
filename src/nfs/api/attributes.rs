use super::NfsFh;

pub enum NfsHandleType {
    Regular,
    Directory,
    Block,
    Character,
    Link,
    Socket,
    Fifo,
    AttributeDirectory,
    NamedAttribute,
}

pub enum NfsExpirationPolicy {
    /// The object won't expire until removed from the file system.
    Persistent,

    /// The object may expire at any time.
    Volatile,

    /// The object may expire at any time, except if it is open.
    VolatileExceptWhenOpen,
}

/// An identifier for a particular filesystem.
///
/// Most users will likely just want to have a single id that is returned
/// for all `NfsAttribute::FilesystemId` attribute requests.
pub struct NfsFileSystemId {
    pub major: u64,
    pub minor: u64,
}

/// Attributes of objects on the file system.
#[derive(Clone, Copy, Debug)]
pub enum NfsAttribute {
    /// The list of supported `NfsAttribute`s for the filehandle.
    SupportedAttributes,

    /// The `NfsType` of the object.
    HandleType,

    /// The `NfsExpirationPolicy` for this object.
    ExpirationPolicy,

    /// A `u64` value used to detect if the object contents have changed.
    ///
    /// This can be as simple as a timestamp on the object.
    Changed,

    /// The size of the object in bytes.
    Size,

    /// Whether hard links are supported.
    LinkSupport,

    /// Whether symbolic links are supported.
    SymlinkSupport,

    /// Wether this filehandle has any named attributes defined.
    ///
    /// Note that this means actually defined. Don't confuse this with wether
    /// named attributes are supported.
    NamedAttributes,

    /// An `NfsFilesystemId` identifying the filesystem.
    FileSystemId,

    /// Wether distinct filehandles are guaranteed to be different objects.
    ///
    /// This will depend on how users of this trait implement their filesystem
    /// tree.
    UniqueHandles,

    /// The time that a lease is valid for.
    LeaseTime,

    /// Used to signal errors during `NfsHandler::read_directory`.
    ReadAttributeError,

    /// Used to retrieve a file handle during `NfsHandler::read_directory`.
    FileHandle,

    /// The list of required and recommended `NfsAttribute`s that will be set
    /// during an exclusive create of a filesystem object.
    ExclusiveCreateAttributes,
}

impl NfsAttribute {
    /// List all `NfsAttribute` variants.
    pub fn all() -> impl Iterator<Item = Self> {
        [
            Self::SupportedAttributes,
            Self::HandleType,
            Self::ExpirationPolicy,
            Self::Changed,
            Self::Size,
            Self::LinkSupport,
            Self::SymlinkSupport,
            Self::NamedAttributes,
            Self::FileSystemId,
            Self::UniqueHandles,
            Self::LeaseTime,
            Self::ReadAttributeError,
            Self::FileHandle,
            Self::ExclusiveCreateAttributes,
        ]
        .iter()
        .copied()
    }

    pub fn required() -> impl Iterator<Item = Self> {
        [
            Self::SupportedAttributes,
            Self::HandleType,
            Self::ExpirationPolicy,
            Self::Changed,
            Self::Size,
            Self::LinkSupport,
            Self::SymlinkSupport,
            Self::NamedAttributes,
            Self::FileSystemId,
            Self::UniqueHandles,
            Self::LeaseTime,
            Self::ReadAttributeError,
            Self::FileHandle,
            Self::ExclusiveCreateAttributes,
        ]
        .iter()
        .copied()
    }
}

/// Values for the corresponding `NfsAttribute`s
pub enum NfsAttributeValue {
    SupportedAttributes(Vec<NfsAttribute>),
    HandleType(NfsHandleType),
    ExpirationPolicy(NfsExpirationPolicy),
    Changed(u64),
    Size(u64),
    LinkSupport(bool),
    SymlinkSupport(bool),
    NamedAttributes(bool),
    FileSystemId(NfsFileSystemId),
    UniqueHandles(bool),
    LeaseTime(u32),
    FileHandle(NfsFh),
    ExclusiveCreateAttributes(Vec<NfsAttribute>),
}
