use num_derive::{FromPrimitive, ToPrimitive};

use super::{
    NfsAce, NfsAclFlag, NfsChangePolicy, NfsExpirationPolicy, NfsFh,
    NfsFileSystemId, NfsFileType, NfsFsLocations, NfsFsLocationsInfo,
    NfsFsStatus, NfsLayoutHint, NfsLayoutType, NfsModeMasked, NfsSpecData,
    NfsTime,
};

/// NFSv4.1 Attributes names
#[derive(
    Clone,
    Copy,
    Debug,
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    FromPrimitive,
    ToPrimitive,
)]
#[repr(u32)]
pub enum NfsAttribute {
    /// The list of supported `NfsAttribute`s for the filehandle.
    SupportedAttributes = 0,

    /// The `NfsFileType` of the object.
    FileType = 1,

    /// The `NfsExpirationPolicy` for this object.
    ExpirationPolicy = 2,

    /// A `u64` value used to detect if the object contents have changed.
    ///
    /// This can be as simple as a timestamp on the object.
    Changed = 3,

    /// The size of the object in bytes.
    Size = 4,

    /// Whether hard links are supported.
    LinkSupport = 5,

    /// Whether symbolic links are supported.
    SymlinkSupport = 6,

    /// Wether this filehandle has any named attributes defined.
    ///
    /// Note that this means actually defined. Don't confuse this with wether
    /// named attributes are supported.
    NamedAttributes = 7,

    /// An `NfsFilesystemId` identifying the filesystem.
    FileSystemId = 8,

    /// Wether distinct filehandles are guaranteed to be different objects.
    ///
    /// This will depend on how users of this trait implement their filesystem
    /// tree.
    UniqueHandles = 9,

    /// The time that a lease is valid for.
    LeaseTime = 10,

    /// Used to signal errors during `NfsHandler::read_directory`.
    ReadAttributeError = 11,

    /// Used to retrieve a file handle during `NfsHandler::read_directory`.
    FileHandle = 19,

    /// The list of required and recommended `NfsAttribute`s that will be set
    /// during an exclusive create of a filesystem object.
    ExclusiveCreateAttributes = 75,

    /// The list of Access Control Entities for this object.
    Acl = 12,

    /// The level of Acl support for this object.
    AclSupport = 13,

    /// Whether or not the object has been archived.
    Archive = 14,

    /// Whether the filesystem allows setting a time.
    CanSetTime = 15,

    /// Whether file system objects are case insensitive
    CaseInsensitive = 16,

    /// Whether file system objects preserve their case
    CasePreserving = 17,

    /// A value used to detect if filesystem policy has changed for the object.
    ChangePolicy = 60,

    /// Whether or not a user must be root to issue chown commands
    ChownRestricted = 18,

    /// Similar to Acl, but only supports ALLOW and DENY ACE's.
    DAcl = 58,

    /// The minimum time in seconds the server will delay directory notifications.
    DirectoryNotificationDelay = 56,

    /// The minimum time in seconds the server will delay entry notifications.
    DirectoryEntryNotificationDelay = 57,

    /// A number that uniquely identifies the object.
    FileId = 20,

    /// The number of available files that the current use can create.
    FilesAvailable = 21,

    /// The total number of files that can be created on the filesystem.
    FilesFree = 22,

    /// The total number of files that exist on the filesystem.
    FilesTotal = 23,

    /// The filesystem's charset abilities.
    FileSystemCharsetAbilities = 76,

    /// The type of filesystem layout type being used.
    FileSystemLayoutType = 62,

    /// Locations where this file system maybe found.
    FileSystemLocations = 24,

    /// Full function file system location.
    FileSystemLocationsInfo = 67,

    /// Generic file system type information.
    FileSystemStatus = 61,

    /// Whether this object is hidden by the Windows APIs
    Hidden = 25,

    /// Whether all objects have the same per-file attributes
    Homogeneous = 26,

    /// The preferred alignment for file system operations.
    LayoutAlignment = 66,

    /// The preferred size of I/O operations
    LayoutBlockSize = 65,

    /// Can be set on newly created objects to influecen metadata servers.
    LayoutHint = 63,

    /// The types of layout available for a file.
    LayoutType = 64,

    /// The maximum size of a file.
    MaxFileSize = 27,

    /// The maximum number of links for an object.
    MaxLinks = 28,

    /// The maximum object name length.
    MaxNameLength = 29,

    /// Maximum amount of data returned by a read operation
    MaxReadLength = 30,

    /// Maximum amount of data that can be written at once.
    MaxWriteLength = 31,

    /// Metadata server threshold size, writes smaller than this go to the
    /// metadata server (if one exists).
    MetadataServerSizeThreshold = 68,

    /// The mimetype of the object.
    MimeType = 32,

    /// The mode of the file.
    Mode = 33,

    /// Set bits in a mode without affecting others.
    ModeSetMasked = 74,

    /// The file id that this file system is mounted on.
    MountedOnFileId = 55,

    /// Whether names exceeding `MaxNameLength` are truncated or an error.
    NoTruncation = 34,

    /// The number of hard links to the object
    NumLinks = 35,

    /// The string name of the owner of this object
    Owner = 36,

    /// The string name of the group ownership of this object.
    OwnerGroup = 37,

    /// The hard limit on remaining disk space
    QuotaAvailableHard = 38,

    /// The soft limit on remaining disk space.
    QuotaAvailableSoft = 39,

    /// The value in bytes used for the applicable quota.
    QuotaUsed = 40,

    /// The raw device info for an object.
    ///
    /// This is only used for block and character devices.
    RawDevice = 41,

    /// Get the event based retention information.
    GetEventRetention = 71,

    /// Set the event based retention information.
    SetEventRetention = 72,

    /// Get the begining time of retention.
    GetRetention = 69,

    /// Set the retention duration for an object.
    SetRetention = 70,

    /// Set an administrative retention on an object.
    HoldRetention = 73,

    /// A limited version of Acl that only applies audit and alarm ACE's
    SAcl = 59,

    /// Space available to this user in bytes.
    FileSystemSpaceAvailable = 42,

    /// Space available on this file system in bytes.
    FileSystemSpaceFree = 43,

    /// Total amount of space the file system can use in bytes.
    FileSystemSpaceTotal = 44,

    /// Total amount of space used by the file system.
    FileSystemSpaceUsed = 45,

    /// Whether or not Windows should treat this as a "system" file.
    IsSystemFile = 46,

    /// The last time this object was accessed.
    TimeAccess = 47,

    /// Set the last time this object was accessed.
    SetTimeAccess = 48,

    /// The last time this object was backed up.
    TimeBackup = 49,

    /// The time this object was created.
    TimeCreate = 50,

    /// The smallest time delta on this file system.
    TimeDelta = 51,

    /// The last time the object's metadata was updated.
    TimeMetadata = 52,

    /// The last time an object's data was modified.
    TimeModify = 53,

    /// Set the time of last modificatin to the object's data.
    SetTimeModify = 54,

    /// A sentinel value for unexpected attribute values sent by clients.
    Illegal = 255,
}

/// A concrete object for describing object attributes.
#[expect(clippy::struct_excessive_bools, reason = "Indeed it does")]
#[derive(Clone, Debug, Default)]
pub struct NfsAttributes {
    // The required attributes
    pub supported_attributes: Vec<NfsAttribute>,
    pub file_type: NfsFileType,
    pub expiration_policy: NfsExpirationPolicy,
    pub changed: u64,
    pub size: u64,
    pub link_support: bool,
    pub symlink_support: bool,
    pub named_attributes: bool,
    pub file_system_id: NfsFileSystemId,
    pub unique_handles: bool,
    pub lease_time: u32,
    pub file_handle: NfsFh,
    pub exclusive_create_attributes: Vec<NfsAttribute>,

    // Optional attributes
    pub acl: Option<Vec<NfsAce>>,
    pub acl_support: Option<u32>,
    pub archive: Option<bool>,
    pub can_set_time: Option<bool>,
    pub case_insensitive: Option<bool>,
    pub case_preserving: Option<bool>,
    pub change_policy: Option<NfsChangePolicy>,
    pub chown_restricted: Option<bool>,
    pub dacl: Option<(NfsAclFlag, Vec<NfsAce>)>,
    pub directory_notification_delay: Option<NfsTime>,
    pub directory_entry_notification_delay: Option<NfsTime>,
    pub file_id: Option<u64>,
    pub files_available: Option<u64>,
    pub files_free: Option<u64>,
    pub files_total: Option<u64>,
    pub file_system_charset_abilities: Option<u32>,
    pub file_system_layout_type: Option<Vec<NfsLayoutType>>,
    pub file_system_locations: Option<NfsFsLocations>,
    pub file_system_locations_info: Option<NfsFsLocationsInfo>,
    pub file_system_status: Option<NfsFsStatus>,
    pub hidden: Option<bool>,
    pub homogeneous: Option<bool>,
    pub layout_alignment: Option<u32>,
    pub layout_block_size: Option<u32>,
    pub layout_hint: Option<NfsLayoutHint>,
    pub layout_type: Option<Vec<NfsLayoutType>>,
    pub max_file_size: Option<u64>,
    pub max_links: Option<u32>,
    pub max_name_length: Option<u32>,
    pub max_read_length: Option<u64>,
    pub max_write_length: Option<u64>,
    // metadata_server_size_threshold - Not exposed for now.
    pub mime_type: Option<String>,
    pub mode: Option<u32>,
    pub mode_set_masked: Option<NfsModeMasked>,
    pub mounted_on_file_id: Option<u64>,
    pub no_truncation: Option<bool>,
    pub num_links: Option<u32>,
    pub owner: Option<String>,
    pub owner_group: Option<String>,
    pub quota_available_hard: Option<u64>,
    pub quota_available_soft: Option<u64>,
    pub quota_used: Option<u64>,
    pub raw_device: Option<NfsSpecData>,
    // get_event_retention - Ignoring for now
    // set_event_retention - Ignoring for now
    // get_retention - Ignoring for now
    // set_retention - Ignoring for now
    // hold_retention - Ignoring for now
    pub sacl: Option<(NfsAclFlag, Vec<NfsAce>)>,
    pub file_system_space_available: Option<u64>,
    pub file_system_space_free: Option<u64>,
    pub file_system_space_total: Option<u64>,
    pub file_system_space_used: Option<u64>,
    pub is_system_file: Option<bool>,
    pub time_access: Option<NfsTime>,
    pub set_time_access: Option<NfsTime>,
    pub time_backup: Option<NfsTime>,
    pub time_create: Option<NfsTime>,
    pub time_delta: Option<NfsTime>,
    pub time_metadata: Option<NfsTime>,
    pub time_modify: Option<NfsTime>,
    pub set_time_modify: Option<NfsTime>,
}
