use std::io::{Read, Write};

use num_derive::{FromPrimitive, ToPrimitive};

use crate::nfs::AsNfsStatus;
use crate::nfs::constants::*;
use crate::nfs::status::NfsStatus;
use crate::rpc::{AuthFlavor, AuthUnix};
use crate::xdr::{self, MaxLenBytes, XdrDeserialize, XdrOpaque, XdrSerialize};

// From RFC 5662: https://www.rfc-editor.org/rfc/rfc5662.txt

#[derive(Clone, Copy, Debug, Default, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum NfsFileType {
    #[default]
    Regular = 1,
    Directory = 2,
    Block = 3,
    Character = 4,
    Link = 5,
    Socket = 6,
    Fifo = 7,
    AttributeDirectory = 8,
    NamedAttribute = 9,
}
xdr::serde_enum!(NfsFileType);

#[derive(Clone, Copy, Debug, Default, FromPrimitive, ToPrimitive)]
pub enum NfsExpirationPolicy {
    /// The object won't expire until removed from the file system.
    #[default]
    Persistent = 0,

    /// The object may expire at any time.
    Volatile = 2,

    /// The object may expire at any time, except if it is open.
    VolatileExceptWhenOpen = 3,
}
xdr::serde_enum!(NfsExpirationPolicy);

/// An identifier for a particular filesystem.
///
/// Most users will likely just want to have a single id that is returned
/// for all `NfsAttribute::FilesystemId` attribute requests.
#[derive(Clone, Copy, Debug, Default)]
pub struct NfsFileSystemId {
    pub major: u64,
    pub minor: u64,
}

impl XdrSerialize for NfsFileSystemId {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.major.serialize(dest)?;
        self.minor.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFileSystemId {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let major = u64::deserialize(src)?;
        let minor = u64::deserialize(src)?;

        Ok(Self { major, minor })
    }
}

pub type AttrList = XdrOpaque;
pub type BitMap = Vec<u32>;
pub type ChangeId = u64;
pub type ClientId = u64;
pub type Count = u32;
pub type Length = u64;
pub type Mode = u32;
pub type NfsCookie = u64;
pub type NfsFh = MaxLenBytes<NFS_FHSIZE>;
pub type Offset = u64;
pub type QOp = u32;
pub type SecOid = XdrOpaque;
pub type SequenceId = u32;
pub type SeqId = u32;
pub type SessionId = [u8; NFS_SESSIONID_SIZE];
pub type SlotId = u32;
pub type Utf8String = Vec<u8>;
pub type Utf8StringCaseSensitive = Utf8String;
pub type Utf8StringCaseInsensitive = Utf8String;
pub type Utf8StringMixed = Utf8String;
pub type Component = Utf8StringCaseSensitive;
pub type LinkText = Utf8StringCaseSensitive;
pub type NfsPathName = Vec<Component>;
pub type Verifier = [u8; NFS_VERIFIER_SIZE];

#[derive(Clone, Copy, Debug)]
pub struct NfsTime {
    pub seconds: i64,
    pub nseconds: u32,
}

impl XdrSerialize for NfsTime {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.seconds.serialize(dest)?;
        self.nseconds.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsTime {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let seconds = i64::deserialize(src)?;
        let nseconds = u32::deserialize(src)?;

        Ok(Self { seconds, nseconds })
    }
}

pub enum TimeHow {
    SetToServer = 0,
    SetToClient = 1,
}

pub enum SetTime {
    SetToServer,
    SetToClient(NfsTime),
}

impl XdrSerialize for SetTime {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::SetToServer => Ok(()),
            Self::SetToClient(time) => time.serialize(dest),
        }
    }
}

type NfsLease = u32;

pub struct FsId {
    major: u64,
    minor: u64,
}

impl XdrSerialize for FsId {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.major.serialize(dest)?;
        self.minor.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for FsId {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let major = u64::deserialize(src)?;
        let minor = u64::deserialize(src)?;

        Ok(Self { major, minor })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NfsChangePolicy {
    pub major: u64,
    pub minor: u64,
}

impl XdrSerialize for NfsChangePolicy {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.major.serialize(dest)?;
        self.minor.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsChangePolicy {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let major = u64::deserialize(src)?;
        let minor = u64::deserialize(src)?;

        Ok(Self { major, minor })
    }
}

#[derive(Clone, Debug)]
pub struct NfsFsLocation {
    server: Utf8StringCaseInsensitive,
    pathname: NfsPathName, // rootpath in the rfc, undefined AFAICT
}

impl XdrSerialize for NfsFsLocation {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.server.serialize(dest)?;
        self.pathname.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFsLocation {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let server = Utf8StringCaseInsensitive::deserialize(src)?;
        let pathname = NfsPathName::deserialize(src)?;

        Ok(Self { server, pathname })
    }
}

#[derive(Clone, Debug)]
pub struct NfsFsLocations {
    root: NfsPathName,
    locations: Vec<NfsFsLocation>,
}

impl XdrSerialize for NfsFsLocations {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.root.serialize(dest)?;
        xdr::serialize_vec(dest, &self.locations)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFsLocations {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let root = NfsPathName::deserialize(src)?;
        let locations = xdr::deserialize_vec::<_, NfsFsLocation>(src)?;

        Ok(Self { root, locations })
    }
}

// Ace = Access Control Entity
pub type NfsAceType = u32;
pub type NfsAceFlag = u32;
pub type NfsAceMask = u32;

#[derive(Clone, Debug)]
pub struct NfsAce {
    typ: NfsAceType,
    flag: NfsAceFlag,
    access_mask: NfsAceMask,
    who: Utf8StringMixed,
}

impl XdrSerialize for NfsAce {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.typ.serialize(dest)?;
        self.flag.serialize(dest)?;
        self.access_mask.serialize(dest)?;
        self.who.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsAce {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let typ = NfsAceType::deserialize(src)?;
        let flag = NfsAceFlag::deserialize(src)?;
        let access_mask = NfsAceMask::deserialize(src)?;
        let who = Utf8StringMixed::deserialize(src)?;

        Ok(Self {
            typ,
            flag,
            access_mask,
            who,
        })
    }
}

pub type NfsAclFlag = u32;

pub struct NfsAcl {
    flag: NfsAclFlag,
    aces: Vec<NfsAce>,
}

impl XdrSerialize for NfsAcl {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.flag.serialize(dest)?;
        xdr::serialize_vec(dest, &self.aces)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsAcl {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let flag = NfsAclFlag::deserialize(src)?;
        let aces = xdr::deserialize_vec::<_, NfsAce>(src)?;

        Ok(Self { flag, aces })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NfsModeMasked {
    pub value_to_set: u32,
    pub mask_bits: u32,
}

impl XdrSerialize for NfsModeMasked {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.value_to_set.serialize(dest)?;
        self.mask_bits.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsModeMasked {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let value_to_set = Mode::deserialize(src)?;
        let mask_bits = Mode::deserialize(src)?;

        Ok(Self {
            value_to_set,
            mask_bits,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NfsSpecData {
    pub specdata1: u32,
    pub specdata2: u32,
}

impl XdrSerialize for NfsSpecData {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.specdata1.serialize(dest)?;
        self.specdata1.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsSpecData {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let specdata1 = u32::deserialize(src)?;
        let specdata2 = u32::deserialize(src)?;

        Ok(Self {
            specdata1,
            specdata2,
        })
    }
}

pub struct NetAddress {
    netid: Vec<u8>,
    addr: Vec<u8>,
}

impl XdrSerialize for NetAddress {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.netid.serialize(dest)?;
        self.addr.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NetAddress {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let netid = Vec::<u8>::deserialize(src)?;
        let addr = Vec::<u8>::deserialize(src)?;

        Ok(Self { netid, addr })
    }
}

#[derive(Debug)]
pub struct NfsImplId {
    domain: Utf8StringCaseInsensitive,
    name: Utf8StringCaseSensitive,
    date: NfsTime,
}

impl XdrSerialize for NfsImplId {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.domain.serialize(dest)?;
        self.name.serialize(dest)?;
        self.date.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsImplId {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let domain = Vec::<u8>::deserialize(src)?;
        let name = Vec::<u8>::deserialize(src)?;
        let date = NfsTime::deserialize(src)?;

        Ok(Self { domain, name, date })
    }
}

#[derive(Debug)]
pub struct StateId {
    seqid: SeqId,
    other: [u8; 12],
}

impl XdrSerialize for StateId {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.seqid.serialize(dest)?;
        self.other.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for StateId {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let seqid = SeqId::deserialize(src)?;
        let other = <[u8; 12]>::deserialize(src)?;

        Ok(Self { seqid, other })
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum NfsLayoutType {
    NfsV41Files = 1,
    Osd2Objects = 2,
    BlockVolume = 3,
}
xdr::serde_enum!(NfsLayoutType);

#[derive(Debug)]
pub struct LayoutContent {
    typ: NfsLayoutType,
    body: XdrOpaque,
}

impl XdrSerialize for LayoutContent {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.typ.serialize(dest)?;
        self.body.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LayoutContent {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let typ = NfsLayoutType::deserialize(src)?;
        let body = XdrOpaque::deserialize(src)?;

        Ok(Self { typ, body })
    }
}

#[derive(Clone, Debug)]
pub struct NfsLayoutHint {
    typ: NfsLayoutType,
    body: XdrOpaque,
}

impl XdrSerialize for NfsLayoutHint {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.typ.serialize(dest)?;
        self.body.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsLayoutHint {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let typ = NfsLayoutType::deserialize(src)?;
        let body = XdrOpaque::deserialize(src)?;

        Ok(Self { typ, body })
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum LayoutIoMode {
    Read = 1,
    Rw = 2,
    Any = 3,
}
xdr::serde_enum!(LayoutIoMode);

#[derive(Debug)]
pub struct Layout {
    offset: Offset,
    length: Length,
    io_mode: LayoutIoMode,
    content: LayoutContent,
}

impl XdrSerialize for Layout {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.offset.serialize(dest)?;
        self.length.serialize(dest)?;
        self.io_mode.serialize(dest)?;
        self.content.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for Layout {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let offset = Offset::deserialize(src)?;
        let length = Length::deserialize(src)?;
        let io_mode = LayoutIoMode::deserialize(src)?;
        let content = LayoutContent::deserialize(src)?;

        Ok(Self {
            offset,
            length,
            io_mode,
            content,
        })
    }
}

pub type DeviceId = [u8; NFS_DEVICEID_SIZE];

#[derive(Debug)]
pub struct DeviceAddress {
    layout_type: NfsLayoutType,
    addr_body: XdrOpaque,
}

impl XdrSerialize for DeviceAddress {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.layout_type.serialize(dest)?;
        self.addr_body.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for DeviceAddress {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let layout_type = NfsLayoutType::deserialize(src)?;
        let addr_body = XdrOpaque::deserialize(src)?;

        Ok(Self {
            layout_type,
            addr_body,
        })
    }
}

#[derive(Debug)]
pub struct LayoutUpdate {
    typ: NfsLayoutType,
    body: XdrOpaque,
}

impl XdrSerialize for LayoutUpdate {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.typ.serialize(dest)?;
        self.body.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LayoutUpdate {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let typ = NfsLayoutType::deserialize(src)?;
        let body = XdrOpaque::deserialize(src)?;

        Ok(Self { typ, body })
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum LayoutReturnType {
    File = LAYOUT_RET_REC_FILE,
    FsId = LAYOUT_RET_REC_FSID,
    All = LAYOUT_RET_REC_ALL,
}
xdr::serde_enum!(LayoutReturnType);

#[derive(Debug)]
pub struct LayoutReturnFile {
    offset: Offset,
    length: Length,
    state_id: StateId,
    body: XdrOpaque,
}

impl XdrSerialize for LayoutReturnFile {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.offset.serialize(dest)?;
        self.length.serialize(dest)?;
        self.state_id.serialize(dest)?;
        self.body.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LayoutReturnFile {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let offset = Offset::deserialize(src)?;
        let length = Length::deserialize(src)?;
        let state_id = StateId::deserialize(src)?;
        let body = XdrOpaque::deserialize(src)?;

        Ok(Self {
            offset,
            length,
            state_id,
            body,
        })
    }
}

#[derive(Debug)]
pub enum LayoutReturn {
    File(LayoutReturnFile),
    FsId,
    All,
}

impl XdrSerialize for LayoutReturn {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::File(file) => {
                LayoutReturnType::File.serialize(dest)?;
                file.serialize(dest)
            }
            Self::FsId => LayoutReturnType::FsId.serialize(dest),
            Self::All => LayoutReturnType::All.serialize(dest),
        }
    }
}

impl XdrDeserialize for LayoutReturn {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let ret_type = LayoutReturnType::deserialize(src)?;
        let ret = match ret_type {
            LayoutReturnType::File => {
                let file = LayoutReturnFile::deserialize(src)?;
                Self::File(file)
            }
            LayoutReturnType::FsId => Self::FsId,
            LayoutReturnType::All => Self::All,
        };

        Ok(ret)
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum FsStatusType {
    Fixed = 1,
    Updated = 2,
    Versioned = 3,
    Writable = 4,
    Referral = 5,
}
xdr::serde_enum!(FsStatusType);

#[derive(Clone, Debug)]
pub struct NfsFsStatus {
    absent: bool,
    typ: FsStatusType,
    source: Utf8StringCaseSensitive,
    current: Utf8StringCaseSensitive,
    age: i32,
    version: NfsTime,
}

impl XdrSerialize for NfsFsStatus {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.absent.serialize(dest)?;
        self.typ.serialize(dest)?;
        self.source.serialize(dest)?;
        self.current.serialize(dest)?;
        self.age.serialize(dest)?;
        self.version.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFsStatus {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let absent = bool::deserialize(src)?;
        let typ = FsStatusType::deserialize(src)?;
        let source = Utf8StringCaseSensitive::deserialize(src)?;
        let current = Utf8StringCaseSensitive::deserialize(src)?;
        let age = i32::deserialize(src)?;
        let version = NfsTime::deserialize(src)?;

        Ok(Self {
            absent,
            typ,
            source,
            current,
            age,
            version,
        })
    }
}

pub type ThresholdReadSize = Length;
pub type ThresholdWriteSize = Length;
pub type ThresholdReadIoSize = Length;
pub type ThresholdWriteIoSize = Length;

pub struct ThresholdItem {
    layout_type: NfsLayoutType,
    hintset: BitMap,
    hintlist: XdrOpaque,
}

impl XdrSerialize for ThresholdItem {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.layout_type.serialize(dest)?;
        self.hintset.serialize(dest)?;
        self.hintlist.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ThresholdItem {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let layout_type = NfsLayoutType::deserialize(src)?;
        let hintset = BitMap::deserialize(src)?;
        let hintlist = XdrOpaque::deserialize(src)?;

        Ok(Self {
            layout_type,
            hintset,
            hintlist,
        })
    }
}

pub struct MdsThreshold {
    hints: Vec<ThresholdItem>,
}

impl XdrSerialize for MdsThreshold {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        xdr::serialize_vec(dest, &self.hints)?;

        Ok(())
    }
}

impl XdrDeserialize for MdsThreshold {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let hints = xdr::deserialize_vec::<_, ThresholdItem>(src)?;

        Ok(Self { hints })
    }
}

pub struct RetentionGet {
    duration: u64,
    begin_time: Option<NfsTime>,
}

impl XdrSerialize for RetentionGet {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.duration.serialize(dest)?;
        self.begin_time.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for RetentionGet {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let duration = u64::deserialize(src)?;
        let begin_time = <Option<NfsTime>>::deserialize(src)?;

        Ok(Self {
            duration,
            begin_time,
        })
    }
}

pub struct RetentionSet {
    enable: bool,
    duration: Option<u64>,
}

impl XdrSerialize for RetentionSet {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.enable.serialize(dest)?;
        self.duration.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for RetentionSet {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let enable = bool::deserialize(src)?;
        let duration = <Option<u64>>::deserialize(src)?;

        Ok(Self { enable, duration })
    }
}

pub type FsCharsetCap = u32;

pub mod fattr {
    use super::*;

    pub type SupportedAttrs = BitMap;
    pub type Type = NfsFileType;
    pub type FhExpireType = u32;
    pub type Change = ChangeId;
    pub type Size = u64;
    pub type LinkSupport = bool;
    pub type SymlinkSupport = bool;
    pub type NamedAttr = bool;
    pub type FsId = super::FsId;
    pub type UniqueHandles = bool;
    pub type LeaseTime = NfsLease;
    pub type RdAttrError = NfsStatus;
    pub type Acl = NfsAce;
    pub type AclSupport = u32;
    pub type Archive = bool;
    pub type CanSetTime = bool;
    pub type CaseInsensitive = bool;
    pub type CasePreserving = bool;
    pub type ChownRestricted = bool;
    pub type FileId = u64;
    pub type FilesAvailable = u64;
    pub type FileHandle = NfsFh;
    pub type FilesFree = u64;
    pub type FilesTotal = u64;
    pub type FsLocations = super::NfsFsLocations;
    pub type Hidden = bool;
    pub type Homogenous = bool;
    pub type MaxFileSize = u64;
    pub type MaxLink = u32;
    pub type MaxName = u32;
    pub type MaxRead = u32;
    pub type MaxWrite = u32;
    pub type MimeType = Utf8StringCaseSensitive;
    pub type Mode = super::Mode;
    pub type ModeSetMasked = NfsModeMasked;
    pub type MountedOnFileId = u64;
    pub type NoTrunc = bool;
    pub type NumLinks = u32;
    pub type Owner = Utf8StringMixed;
    pub type OwnerGroup = Utf8StringMixed;
    pub type QuotaAvailableHard = u64;
    pub type QuotaAvailableSoft = u64;
    pub type QuoteUsed = u64;
    pub type RawDevice = NfsSpecData;
    pub type SpaceAvailable = u64;
    pub type SpaceFree = u64;
    pub type SpaceTotal = u64;
    pub type SpaceUsed = u64;
    pub type System = bool;
    pub type TimeAccess = NfsTime;
    pub type TimeAccessSet = NfsTime;
    pub type TimeBackup = NfsTime;
    pub type TimeCreate = NfsTime;
    pub type TimeDelta = NfsTime;
    pub type TimeMetadata = NfsTime;
    pub type TimeModify = NfsTime;
    pub type TimeModifySet = NfsTime;

    // New in 4.1
    pub type SupportAttrExclusiveCreate = BitMap;
    pub type DirectoryNotifyDelay = NfsTime;
    pub type DirectoryEntryNotifyDelay = NfsTime;
    pub type FsLayoutTypes = Vec<NfsLayoutType>;
    pub type FsStatus = super::NfsFsStatus;
    pub type FsCharasetCap = super::FsCharsetCap;
    pub type LayoutAlignment = u32;
    pub type LayoutBlocksize = u32;
    pub type LayoutHint = super::NfsLayoutHint;
    pub type LayoutTypes = Vec<NfsLayoutType>;
    pub type MdsThreshold = super::MdsThreshold;
    pub type RetentionGet = super::RetentionGet;
    pub type RetentionSet = super::RetentionSet;
    pub type RetentionEventGet = super::RetentionGet;
    pub type RetentionEventSet = super::RetentionSet;
    pub type RetentionHold = u64;
    pub type DAcl = NfsAcl;
    pub type SAcl = NfsAcl;
    pub type ChangePolicy = super::NfsChangePolicy;

    // From further down the RFC
    pub type FsLocationsInfo = super::NfsFsLocationsInfo;
}

#[derive(Debug)]
pub struct FileAttrs {
    pub mask: BitMap,
    pub attrs: AttrList,
}

impl XdrSerialize for FileAttrs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.mask.serialize(dest)?;
        self.attrs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for FileAttrs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let mask = BitMap::deserialize(src)?;
        let attrs = AttrList::deserialize(src)?;

        Ok(Self { mask, attrs })
    }
}

#[derive(Debug)]
pub struct ChangeInfo {
    atomic: bool,
    before: ChangeId,
    after: ChangeId,
}

impl XdrSerialize for ChangeInfo {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.atomic.serialize(dest)?;
        self.before.serialize(dest)?;
        self.after.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ChangeInfo {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let atomic = bool::deserialize(src)?;
        let before = ChangeId::deserialize(src)?;
        let after = ChangeId::deserialize(src)?;

        Ok(Self {
            atomic,
            before,
            after,
        })
    }
}

pub type ClientAddress = NetAddress;

pub struct CallbackClient {
    program: u32,
    location: NetAddress,
}

impl XdrSerialize for CallbackClient {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.program.serialize(dest)?;
        self.location.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CallbackClient {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let program = u32::deserialize(src)?;
        let location = NetAddress::deserialize(src)?;

        Ok(Self { program, location })
    }
}

/// NFSv4.0 Long Hand Client Id
pub struct NfsClientId {
    verifier: Verifier,
    id: MaxLenBytes<NFS_OPAQUE_LIMIT>,
}

impl XdrSerialize for NfsClientId {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.verifier.serialize(dest)?;
        self.id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsClientId {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let verifier = Verifier::deserialize(src)?;
        let id = <MaxLenBytes<NFS_OPAQUE_LIMIT>>::deserialize(src)?;

        Ok(Self { verifier, id })
    }
}

/// NFSv4.1 Client Owner (aka long hand client id)
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ClientOwner {
    pub verifier: Verifier,
    pub owner_id: MaxLenBytes<NFS_OPAQUE_LIMIT>,
}

impl XdrSerialize for ClientOwner {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.verifier.serialize(dest)?;
        self.owner_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ClientOwner {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let verifier = Verifier::deserialize(src)?;
        let owner_id = <MaxLenBytes<NFS_OPAQUE_LIMIT>>::deserialize(src)?;

        Ok(Self { verifier, owner_id })
    }
}

/// NFSv4.1 `ServerOwner`
#[derive(Clone, Debug)]
pub struct ServerOwner {
    pub minor_id: u64,
    pub major_id: MaxLenBytes<NFS_OPAQUE_LIMIT>,
}

impl XdrSerialize for ServerOwner {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.minor_id.serialize(dest)?;
        self.major_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ServerOwner {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let minor_id = u64::deserialize(src)?;
        let major_id = <MaxLenBytes<NFS_OPAQUE_LIMIT>>::deserialize(src)?;

        Ok(Self { minor_id, major_id })
    }
}

#[derive(Debug)]
pub struct StateOwner {
    client_id: ClientId,
    owner: MaxLenBytes<NFS_OPAQUE_LIMIT>,
}

impl XdrSerialize for StateOwner {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_id.serialize(dest)?;
        self.owner.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for StateOwner {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_id = ClientId::deserialize(src)?;
        let owner = <MaxLenBytes<NFS_OPAQUE_LIMIT>>::deserialize(src)?;

        Ok(Self { client_id, owner })
    }
}

pub type OpenOwner = StateOwner;
pub type LockOwner = StateOwner;

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum NfsLockType {
    Read = 1,
    Write = 2,
    BlockingRead = 3,
    BlockingWrite = 4,
}
xdr::serde_enum!(NfsLockType);

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum SsvSubkey {
    MicI2T = 1,
    MicT2I = 2,
    SealI2T = 3,
    SealT2I = 4,
}
xdr::serde_enum!(SsvSubkey);

pub struct SsvMicPlainToken {
    ssv_seq: u32,
    orig_plain: XdrOpaque,
}

impl XdrSerialize for SsvMicPlainToken {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.ssv_seq.serialize(dest)?;
        self.orig_plain.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SsvMicPlainToken {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let ssv_seq = u32::deserialize(src)?;
        let orig_plain = XdrOpaque::deserialize(src)?;

        Ok(Self {
            ssv_seq,
            orig_plain,
        })
    }
}

pub struct SsvSealPlainToken {
    confounder: XdrOpaque,
    ssv_seq: u32,
    orig_plain: XdrOpaque,
    pad: XdrOpaque,
}

impl XdrSerialize for SsvSealPlainToken {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.confounder.serialize(dest)?;
        self.ssv_seq.serialize(dest)?;
        self.orig_plain.serialize(dest)?;
        self.pad.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SsvSealPlainToken {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let confounder = XdrOpaque::deserialize(src)?;
        let ssv_seq = u32::deserialize(src)?;
        let orig_plain = XdrOpaque::deserialize(src)?;
        let pad = XdrOpaque::deserialize(src)?;

        Ok(Self {
            confounder,
            ssv_seq,
            orig_plain,
            pad,
        })
    }
}

pub struct SsvSealCipherToken {
    ssv_seq: u32,
    iv: XdrOpaque,
    encr_data: XdrOpaque,
    hmac: XdrOpaque,
}

impl XdrSerialize for SsvSealCipherToken {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.ssv_seq.serialize(dest)?;
        self.iv.serialize(dest)?;
        self.encr_data.serialize(dest)?;
        self.hmac.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SsvSealCipherToken {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let ssv_seq = u32::deserialize(src)?;
        let iv = XdrOpaque::deserialize(src)?;
        let encr_data = XdrOpaque::deserialize(src)?;
        let hmac = XdrOpaque::deserialize(src)?;

        Ok(Self {
            ssv_seq,
            iv,
            encr_data,
            hmac,
        })
    }
}

#[derive(Clone, Debug)]
pub struct NfsFsLocationsServer {
    currency: i32,
    info: XdrOpaque,
    server: Utf8StringCaseInsensitive,
}

impl XdrSerialize for NfsFsLocationsServer {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.currency.serialize(dest)?;
        self.info.serialize(dest)?;
        self.server.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFsLocationsServer {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let currency = i32::deserialize(src)?;
        let info = XdrOpaque::deserialize(src)?;
        let server = Utf8StringCaseInsensitive::deserialize(src)?;

        Ok(Self {
            currency,
            info,
            server,
        })
    }
}

#[derive(Clone, Debug)]
pub struct NfsFsLocationsItem {
    entries: Vec<NfsFsLocationsServer>,
    rootpath: NfsPathName,
}

impl XdrSerialize for NfsFsLocationsItem {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        xdr::serialize_vec(dest, &self.entries)?;
        self.rootpath.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFsLocationsItem {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let entries = xdr::deserialize_vec::<_, NfsFsLocationsServer>(src)?;
        let rootpath = NfsPathName::deserialize(src)?;

        Ok(Self { entries, rootpath })
    }
}

#[derive(Clone, Debug)]
pub struct NfsFsLocationsInfo {
    flags: u32,
    valid_for: i32,
    fs_root: NfsPathName,
    items: Vec<NfsFsLocationsItem>,
}

impl XdrSerialize for NfsFsLocationsInfo {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.flags.serialize(dest)?;
        self.valid_for.serialize(dest)?;
        self.fs_root.serialize(dest)?;
        xdr::serialize_vec(dest, &self.items)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFsLocationsInfo {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let flags = u32::deserialize(src)?;
        let valid_for = i32::deserialize(src)?;
        let fs_root = NfsPathName::deserialize(src)?;
        let items = xdr::deserialize_vec::<_, NfsFsLocationsItem>(src)?;

        Ok(Self {
            flags,
            valid_for,
            fs_root,
            items,
        })
    }
}

pub type NflUtil = u32;

// This is defined in the spec, but not used anywhere.
// pub enum FileLayoutHintCare {
//     Dense = NFL_UFLG_DENSE,
//     CommitThruMds = NFL_UFLG_COMMIT_THRU_MDS,
//     StripeCount = 0x00_00_00_80,
// }

// Encoded in the loh_body field of data type layouthint4:
pub struct NfsFileLayoutHint {
    care: u32,
    util: NflUtil,
    stripe_count: Count,
}

impl XdrSerialize for NfsFileLayoutHint {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.care.serialize(dest)?;
        self.util.serialize(dest)?;
        self.stripe_count.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFileLayoutHint {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let care = u32::deserialize(src)?;
        let util = NflUtil::deserialize(src)?;
        let stripe_count = Count::deserialize(src)?;

        Ok(Self {
            care,
            util,
            stripe_count,
        })
    }
}

pub type MultipathList = Vec<NetAddress>;

// Encoded in the da_addr_body field of data type device_addr4:
pub struct NfsFileLayoutDsAddress {
    stripe_indices: Vec<u32>,
    multipath_ds_list: Vec<MultipathList>,
}

impl XdrSerialize for NfsFileLayoutDsAddress {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.stripe_indices.serialize(dest)?;
        xdr::serialize_vec_vec(dest, &self.multipath_ds_list)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFileLayoutDsAddress {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let stripe_indices = Vec::<u32>::deserialize(src)?;
        let multipath_ds_list = xdr::deserialize_vec_vec::<_, NetAddress>(src)?;

        Ok(Self {
            stripe_indices,
            multipath_ds_list,
        })
    }
}

pub struct NfsFileLayout {
    device_id: DeviceId,
    util: NflUtil,
    first_stripe_index: u32,
    pattern_offset: Offset,
    fh_list: Vec<NfsFh>,
}

impl XdrSerialize for NfsFileLayout {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.device_id.serialize(dest)?;
        self.util.serialize(dest)?;
        self.first_stripe_index.serialize(dest)?;
        self.pattern_offset.serialize(dest)?;
        xdr::serialize_vec(dest, &self.fh_list)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsFileLayout {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let device_id = DeviceId::deserialize(src)?;
        let util = NflUtil::deserialize(src)?;
        let first_stripe_index = u32::deserialize(src)?;
        let pattern_offset = Offset::deserialize(src)?;
        let fh_list = xdr::deserialize_vec::<_, NfsFh>(src)?;

        Ok(Self {
            device_id,
            util,
            first_stripe_index,
            pattern_offset,
            fh_list,
        })
    }
}

#[derive(Debug)]
pub struct AccessArgs {
    access: u32,
}

impl XdrSerialize for AccessArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.access.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for AccessArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let access = u32::deserialize(src)?;

        Ok(Self { access })
    }
}

#[derive(Debug)]
pub struct AccessOk {
    supported: u32,
    access: u32,
}

impl AsNfsStatus for AccessOk {}

impl XdrSerialize for AccessOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.supported.serialize(dest)?;
        self.access.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for AccessOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let supported = u32::deserialize(src)?;
        let access = u32::deserialize(src)?;

        Ok(Self { supported, access })
    }
}

pub type AccessResult = Result<AccessOk, NfsStatus>;

#[derive(Debug)]
pub struct CloseArgs {
    seq_id: SeqId,
    open_state_id: StateId,
}

impl XdrSerialize for CloseArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.seq_id.serialize(dest)?;
        self.open_state_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CloseArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let seq_id = SeqId::deserialize(src)?;
        let open_state_id = StateId::deserialize(src)?;

        Ok(Self {
            seq_id,
            open_state_id,
        })
    }
}

#[derive(Debug)]
pub struct CloseOk {
    state_id: StateId,
}

impl AsNfsStatus for CloseOk {}

impl XdrSerialize for CloseOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CloseOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;

        Ok(Self { state_id })
    }
}

pub type CloseResult = Result<CloseOk, NfsStatus>;

#[derive(Debug)]
pub struct CommitArgs {
    offset: Offset,
    count: Count,
}

impl XdrSerialize for CommitArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.offset.serialize(dest)?;
        self.count.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CommitArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let offset = Offset::deserialize(src)?;
        let count = Count::deserialize(src)?;

        Ok(Self { offset, count })
    }
}

#[derive(Debug)]
pub struct CommitOk {
    verifier: Verifier,
}

impl AsNfsStatus for CommitOk {}

impl XdrSerialize for CommitOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.verifier.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CommitOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let verifier = Verifier::deserialize(src)?;

        Ok(Self { verifier })
    }
}

pub type CommitResult = Result<CommitOk, NfsStatus>;

#[derive(Debug)]
pub enum CreateType {
    Link(LinkText),
    Block(NfsSpecData),
    Character(NfsSpecData),
    Socket,
    Fifo,
    Directory,
    Invalid(NfsFileType),
}

impl XdrSerialize for CreateType {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Link(val) => {
                NfsFileType::Link.serialize(dest)?;
                val.serialize(dest)
            }
            Self::Block(data) => {
                NfsFileType::Block.serialize(dest)?;
                data.serialize(dest)
            }
            Self::Character(data) => {
                NfsFileType::Character.serialize(dest)?;
                data.serialize(dest)
            }
            Self::Socket => NfsFileType::Socket.serialize(dest),
            Self::Fifo => NfsFileType::Fifo.serialize(dest),
            Self::Directory => NfsFileType::Directory.serialize(dest),
            Self::Invalid(_) => {
                panic!("Invalid CreateType during serialization.")
            }
        }
    }
}

impl XdrDeserialize for CreateType {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let ftype = NfsFileType::deserialize(src)?;
        let object_type = match ftype {
            NfsFileType::Link => {
                let lt = LinkText::deserialize(src)?;
                Self::Link(lt)
            }
            NfsFileType::Block => {
                let data = NfsSpecData::deserialize(src)?;
                Self::Block(data)
            }
            NfsFileType::Character => {
                let data = NfsSpecData::deserialize(src)?;
                Self::Character(data)
            }
            NfsFileType::Socket => Self::Socket,
            NfsFileType::Fifo => Self::Fifo,
            NfsFileType::Directory => Self::Directory,
            NfsFileType::Regular
            | NfsFileType::AttributeDirectory
            | NfsFileType::NamedAttribute => Self::Invalid(ftype),
        };

        Ok(object_type)
    }
}

#[derive(Debug)]
pub struct CreateArgs {
    object_type: CreateType,
    object_name: Component,
    create_attrs: FileAttrs,
}

impl XdrSerialize for CreateArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.object_type.serialize(dest)?;
        self.object_name.serialize(dest)?;
        self.create_attrs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CreateArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let object_type = CreateType::deserialize(src)?;
        let object_name = Component::deserialize(src)?;
        let create_attrs = FileAttrs::deserialize(src)?;

        Ok(Self {
            object_type,
            object_name,
            create_attrs,
        })
    }
}

#[derive(Debug)]
pub struct CreateOk {
    change_info: ChangeInfo,
    attrset: BitMap,
}

impl AsNfsStatus for CreateOk {}

impl XdrSerialize for CreateOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.change_info.serialize(dest)?;
        self.attrset.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CreateOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let change_info = ChangeInfo::deserialize(src)?;
        let attrset = BitMap::deserialize(src)?;

        Ok(Self {
            change_info,
            attrset,
        })
    }
}

pub type CreateResult = Result<CreateOk, NfsStatus>;

#[derive(Debug)]
pub struct PurgeDelegationsArgs {
    client_id: ClientId,
}

impl XdrSerialize for PurgeDelegationsArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for PurgeDelegationsArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_id = ClientId::deserialize(src)?;

        Ok(Self { client_id })
    }
}

pub type PurgeDelegationsResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct ReturnDelegationArgs {
    delegation_state_id: StateId,
}

impl XdrSerialize for ReturnDelegationArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.delegation_state_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ReturnDelegationArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let delegation_state_id = StateId::deserialize(src)?;

        Ok(Self {
            delegation_state_id,
        })
    }
}

pub type ReturnDelegationResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct GetAttributesArgs {
    pub attr_request: BitMap,
}

impl XdrSerialize for GetAttributesArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.attr_request.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetAttributesArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let attr_request = BitMap::deserialize(src)?;

        Ok(Self { attr_request })
    }
}

#[derive(Debug)]
pub struct GetAttributesOk {
    pub attributes: FileAttrs,
}

impl AsNfsStatus for GetAttributesOk {}

impl XdrSerialize for GetAttributesOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.attributes.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetAttributesOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let attributes = FileAttrs::deserialize(src)?;

        Ok(Self { attributes })
    }
}

pub type GetAttributesResult = Result<GetAttributesOk, NfsStatus>;

#[derive(Debug)]
pub struct GetFhOk {
    pub object: NfsFh,
}

impl AsNfsStatus for GetFhOk {}

impl XdrSerialize for GetFhOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.object.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetFhOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let object = NfsFh::deserialize(src)?;

        Ok(Self { object })
    }
}

pub type GetFhResult = Result<GetFhOk, NfsStatus>;

#[derive(Debug)]
pub struct LinkArgs {
    new_name: Component,
}

impl XdrSerialize for LinkArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.new_name.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LinkArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let new_name = Component::deserialize(src)?;

        Ok(Self { new_name })
    }
}

#[derive(Debug)]
pub struct LinkOk {
    change_info: ChangeInfo,
}

impl AsNfsStatus for LinkOk {}

impl XdrSerialize for LinkOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.change_info.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LinkOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let change_info = ChangeInfo::deserialize(src)?;

        Ok(Self { change_info })
    }
}

pub type LinkResult = Result<LinkOk, NfsStatus>;

#[derive(Debug)]
pub struct OpenToLockOwner {
    open_seq_id: SeqId,
    open_state_id: StateId,
    lock_seq_id: SeqId,
    lock_owner: LockOwner,
}

impl XdrSerialize for OpenToLockOwner {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.open_seq_id.serialize(dest)?;
        self.open_state_id.serialize(dest)?;
        self.lock_seq_id.serialize(dest)?;
        self.lock_owner.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenToLockOwner {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let open_seq_id = SeqId::deserialize(src)?;
        let open_state_id = StateId::deserialize(src)?;
        let lock_seq_id = SeqId::deserialize(src)?;
        let lock_owner = LockOwner::deserialize(src)?;

        Ok(Self {
            open_seq_id,
            open_state_id,
            lock_seq_id,
            lock_owner,
        })
    }
}

#[derive(Debug)]
pub struct ExistingLockOwner {
    state_id: StateId,
    seq_id: SeqId,
}

impl XdrSerialize for ExistingLockOwner {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;
        self.seq_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ExistingLockOwner {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;
        let seq_id = SeqId::deserialize(src)?;

        Ok(Self { state_id, seq_id })
    }
}

#[derive(Debug)]
pub enum Locker {
    OpenOwner(OpenToLockOwner),
    LockOwner(ExistingLockOwner),
}

impl XdrSerialize for Locker {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::OpenOwner(owner) => {
                true.serialize(dest)?;
                owner.serialize(dest)?;
            }
            Self::LockOwner(owner) => {
                false.serialize(dest)?;
                owner.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for Locker {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let flag = bool::deserialize(src)?;

        if flag {
            let owner = OpenToLockOwner::deserialize(src)?;
            Ok(Self::OpenOwner(owner))
        } else {
            let owner = ExistingLockOwner::deserialize(src)?;
            Ok(Self::LockOwner(owner))
        }
    }
}

#[derive(Debug)]
pub struct LockArgs {
    lock_type: NfsLockType,
    reclaim: bool,
    offset: Offset,
    length: Length,
    locker: Locker,
}

impl XdrSerialize for LockArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.lock_type.serialize(dest)?;
        self.reclaim.serialize(dest)?;
        self.offset.serialize(dest)?;
        self.length.serialize(dest)?;
        self.locker.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LockArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let lock_type = NfsLockType::deserialize(src)?;
        let reclaim = bool::deserialize(src)?;
        let offset = Offset::deserialize(src)?;
        let length = Length::deserialize(src)?;
        let locker = Locker::deserialize(src)?;

        Ok(Self {
            lock_type,
            reclaim,
            offset,
            length,
            locker,
        })
    }
}

#[derive(Debug)]
pub struct LockOk {
    state_id: StateId,
}

impl AsNfsStatus for LockOk {}

impl XdrSerialize for LockOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LockOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;

        Ok(Self { state_id })
    }
}

#[derive(Debug)]
pub struct LockDenied {
    offset: Offset,
    length: Length,
    lock_type: NfsLockType,
    owner: LockOwner,
}

impl XdrSerialize for LockDenied {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.offset.serialize(dest)?;
        self.length.serialize(dest)?;
        self.lock_type.serialize(dest)?;
        self.owner.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LockDenied {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let offset = Offset::deserialize(src)?;
        let length = Length::deserialize(src)?;
        let lock_type = NfsLockType::deserialize(src)?;
        let owner = LockOwner::deserialize(src)?;

        Ok(Self {
            offset,
            length,
            lock_type,
            owner,
        })
    }
}

#[derive(Debug)]
pub enum LockReturn {
    Ok(LockOk),
    Denied(LockDenied),
}

impl AsNfsStatus for LockReturn {
    fn as_status(&self) -> NfsStatus {
        match self {
            Self::Ok(_) => NfsStatus::Ok,
            Self::Denied(_) => NfsStatus::Denied,
        }
    }
}

impl XdrSerialize for LockReturn {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Ok(ok) => ok.serialize(dest),
            Self::Denied(denied) => denied.serialize(dest),
        }
    }
}

pub type LockResult = Result<LockReturn, NfsStatus>;

#[derive(Debug)]
pub struct LockTestArgs {
    lock_type: NfsLockType,
    offset: Offset,
    length: Length,
    owner: LockOwner,
}

impl XdrSerialize for LockTestArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.lock_type.serialize(dest)?;
        self.offset.serialize(dest)?;
        self.length.serialize(dest)?;
        self.owner.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LockTestArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let lock_type = NfsLockType::deserialize(src)?;
        let offset = Offset::deserialize(src)?;
        let length = Length::deserialize(src)?;
        let owner = LockOwner::deserialize(src)?;

        Ok(Self {
            lock_type,
            offset,
            length,
            owner,
        })
    }
}

#[derive(Debug)]
pub enum LockTestReturn {
    Ok,
    Denied(LockDenied),
}

impl AsNfsStatus for LockTestReturn {
    fn as_status(&self) -> NfsStatus {
        match self {
            Self::Ok => NfsStatus::Ok,
            Self::Denied(_) => NfsStatus::Denied,
        }
    }
}

impl XdrSerialize for LockTestReturn {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Ok => Ok(()),
            Self::Denied(denied) => denied.serialize(dest),
        }
    }
}

pub type LockTestResult = Result<LockTestReturn, NfsStatus>;

#[derive(Debug)]
pub struct LockReleaseArgs {
    lock_type: NfsLockType,
    seq_id: SeqId,
    state_id: StateId,
    offset: Offset,
    length: Length,
}

impl XdrSerialize for LockReleaseArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.lock_type.serialize(dest)?;
        self.seq_id.serialize(dest)?;
        self.state_id.serialize(dest)?;
        self.offset.serialize(dest)?;
        self.length.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LockReleaseArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let lock_type = NfsLockType::deserialize(src)?;
        let seq_id = SeqId::deserialize(src)?;
        let state_id = StateId::deserialize(src)?;
        let offset = Offset::deserialize(src)?;
        let length = Length::deserialize(src)?;

        Ok(Self {
            lock_type,
            seq_id,
            state_id,
            offset,
            length,
        })
    }
}

#[derive(Debug)]
pub struct LockReleaseOk {
    state_id: StateId,
}

impl XdrSerialize for LockReleaseOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;

        Ok(())
    }
}

impl AsNfsStatus for LockReleaseOk {}

impl XdrDeserialize for LockReleaseOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;

        Ok(Self { state_id })
    }
}

pub type LockReleaseResult = Result<LockReleaseOk, NfsStatus>;

#[derive(Debug)]
pub struct LookupArgs {
    object_name: Component,
}

impl XdrSerialize for LookupArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.object_name.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LookupArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let object_name = Component::deserialize(src)?;

        Ok(Self { object_name })
    }
}

pub type LookupResult = Result<NfsStatus, NfsStatus>;

pub type LookupParentResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct VerifyAttributeDifferenceArgs {
    object_attrs: FileAttrs,
}

impl XdrSerialize for VerifyAttributeDifferenceArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.object_attrs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for VerifyAttributeDifferenceArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let object_attrs = FileAttrs::deserialize(src)?;

        Ok(Self { object_attrs })
    }
}

pub type VerifyAttributeDifferenceResult = Result<NfsStatus, NfsStatus>;

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum CreateMode {
    Unchecked = 0,
    Guarded = 1,
    Exclusive4 = 2,
    Exclusive4_1 = 3,
}
xdr::serde_enum!(CreateMode);

#[derive(Debug)]
pub struct CreateVerifiedFileAttrs {
    verifier: Verifier,
    attrs: FileAttrs,
}

impl XdrSerialize for CreateVerifiedFileAttrs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.verifier.serialize(dest)?;
        self.attrs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CreateVerifiedFileAttrs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let verifier = Verifier::deserialize(src)?;
        let attrs = FileAttrs::deserialize(src)?;

        Ok(Self { verifier, attrs })
    }
}

#[derive(Debug)]
pub enum CreateHow {
    Unchecked(FileAttrs),
    Guarded(FileAttrs),
    Exclusive4(Verifier),
    Exclusive4_1(CreateVerifiedFileAttrs),
}

impl XdrSerialize for CreateHow {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Unchecked(attrs) => {
                CreateMode::Unchecked.serialize(dest)?;
                attrs.serialize(dest)?;
            }
            Self::Guarded(attrs) => {
                CreateMode::Guarded.serialize(dest)?;
                attrs.serialize(dest)?;
            }
            Self::Exclusive4(verifier) => {
                CreateMode::Exclusive4.serialize(dest)?;
                verifier.serialize(dest)?;
            }
            Self::Exclusive4_1(verified_attrs) => {
                CreateMode::Exclusive4_1.serialize(dest)?;
                verified_attrs.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for CreateHow {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let cmode = CreateMode::deserialize(src)?;
        let how = match cmode {
            CreateMode::Unchecked => {
                let attrs = FileAttrs::deserialize(src)?;
                Self::Unchecked(attrs)
            }
            CreateMode::Guarded => {
                let attrs = FileAttrs::deserialize(src)?;
                Self::Guarded(attrs)
            }
            CreateMode::Exclusive4 => {
                let verifier = Verifier::deserialize(src)?;
                Self::Exclusive4(verifier)
            }
            CreateMode::Exclusive4_1 => {
                let verified_attrs = CreateVerifiedFileAttrs::deserialize(src)?;
                Self::Exclusive4_1(verified_attrs)
            }
        };

        Ok(how)
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum OpenType {
    NoCreate = 0,
    Create = 1,
}
xdr::serde_enum!(OpenType);

#[derive(Debug)]
pub enum OpenFlag {
    NoCreate,
    Create(CreateHow),
}

impl XdrSerialize for OpenFlag {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::NoCreate => OpenType::NoCreate.serialize(dest),
            Self::Create(how) => {
                OpenType::Create.serialize(dest)?;
                how.serialize(dest)
            }
        }
    }
}

impl XdrDeserialize for OpenFlag {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let open_type = OpenType::deserialize(src)?;
        match open_type {
            OpenType::NoCreate => Ok(Self::NoCreate),
            OpenType::Create => {
                let how = CreateHow::deserialize(src)?;
                Ok(Self::Create(how))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum LimitBy {
    Size = 1,
    Blocks = 2,
}
xdr::serde_enum!(LimitBy);

#[derive(Debug)]
pub struct NfsModifiedLimit {
    num_blocks: u32,
    bytes_per_block: u32,
}

impl XdrSerialize for NfsModifiedLimit {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.num_blocks.serialize(dest)?;
        self.bytes_per_block.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for NfsModifiedLimit {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let num_blocks = u32::deserialize(src)?;
        let bytes_per_block = u32::deserialize(src)?;

        Ok(Self {
            num_blocks,
            bytes_per_block,
        })
    }
}

#[derive(Debug)]
pub enum NfsSpaceLimit {
    Size(u64),
    Blocks(NfsModifiedLimit),
}

impl XdrSerialize for NfsSpaceLimit {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Size(size) => {
                LimitBy::Size.serialize(dest)?;
                size.serialize(dest)?;
            }
            Self::Blocks(limit) => {
                LimitBy::Blocks.serialize(dest)?;
                limit.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for NfsSpaceLimit {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let limit_by = LimitBy::deserialize(src)?;
        match limit_by {
            LimitBy::Size => {
                let size = u64::deserialize(src)?;
                Ok(Self::Size(size))
            }
            LimitBy::Blocks => {
                let limit = NfsModifiedLimit::deserialize(src)?;
                Ok(Self::Blocks(limit))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum OpenDelegationType {
    None = 0,
    Read = 1,
    Write = 2,
    NoneExt = 3,
}
xdr::serde_enum!(OpenDelegationType);

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum OpenClaimType {
    Null = 0,
    Previous = 1,
    CurrentDelegate = 2,
    PreviousDelegate = 3,
    CurrentFh = 4,
    CurrentDelegateFh = 5,
    PreviousDelegateFh = 6,
}
xdr::serde_enum!(OpenClaimType);

#[derive(Debug)]
pub struct CurrentDelegateOpenClaim {
    delegate_state_id: StateId,
    file: Component,
}

impl XdrSerialize for CurrentDelegateOpenClaim {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.delegate_state_id.serialize(dest)?;
        self.file.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CurrentDelegateOpenClaim {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let delegate_state_id = StateId::deserialize(src)?;
        let file = Component::deserialize(src)?;

        Ok(Self {
            delegate_state_id,
            file,
        })
    }
}

#[derive(Debug)]
pub enum OpenClaim {
    // No special rights to file.
    // Ordinary OPEN of the specified file.
    Null(Component),

    // Right to the file established by an
    // open previous to server reboot.  File
    // identified by filehandle obtained at
    // that time rather than by name.
    Previous(OpenDelegationType),

    // Right to file based on a delegation
    // granted by the server.  File is
    // specified by name.
    CurrentDelegate(CurrentDelegateOpenClaim),

    // Right to file based on a delegation
    // granted to a previous boot instance
    // of the client.  File is specified by name.
    PreviousDelegate(Component),

    // Like Null.  No special rights
    // to file.  Ordinary OPEN of the
    // specified file by current filehandle.
    CurrentFh,

    // Like CLAIM_DELEGATE_CUR.  Right to file based on
    // a delegation granted by the server.
    // File is identified by filehandle.
    CurrentDelegateFh(StateId),

    // Like CLAIM_DELEGATE_PREV.  Right to file based on a
    // delegation granted to a previous boot
    // instance of the client.  File is identified
    // by filehandle.
    PreviousDelegateFh,
}

impl XdrSerialize for OpenClaim {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Null(component) => {
                OpenClaimType::Null.serialize(dest)?;
                component.serialize(dest)?;
            }
            Self::Previous(delegation_type) => {
                OpenClaimType::Previous.serialize(dest)?;
                delegation_type.serialize(dest)?;
            }
            Self::CurrentDelegate(current_claim) => {
                OpenClaimType::CurrentDelegate.serialize(dest)?;
                current_claim.serialize(dest)?;
            }
            Self::PreviousDelegate(component) => {
                OpenClaimType::PreviousDelegate.serialize(dest)?;
                component.serialize(dest)?;
            }
            Self::CurrentFh => {
                OpenClaimType::CurrentFh.serialize(dest)?;
            }
            Self::CurrentDelegateFh(state_id) => {
                OpenClaimType::CurrentDelegateFh.serialize(dest)?;
                state_id.serialize(dest)?;
            }
            Self::PreviousDelegateFh => {
                OpenClaimType::PreviousDelegateFh.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for OpenClaim {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let claim_type = OpenClaimType::deserialize(src)?;
        let claim = match claim_type {
            OpenClaimType::Null => {
                let component = Component::deserialize(src)?;
                Self::Null(component)
            }
            OpenClaimType::Previous => {
                let delegation_type = OpenDelegationType::deserialize(src)?;
                Self::Previous(delegation_type)
            }
            OpenClaimType::CurrentDelegate => {
                let current_claim = CurrentDelegateOpenClaim::deserialize(src)?;
                Self::CurrentDelegate(current_claim)
            }
            OpenClaimType::PreviousDelegate => {
                let component = Component::deserialize(src)?;
                Self::PreviousDelegate(component)
            }
            OpenClaimType::CurrentFh => Self::CurrentFh,
            OpenClaimType::CurrentDelegateFh => {
                let state_id = StateId::deserialize(src)?;
                Self::CurrentDelegateFh(state_id)
            }
            OpenClaimType::PreviousDelegateFh => Self::PreviousDelegateFh,
        };

        Ok(claim)
    }
}

#[derive(Debug)]
pub struct OpenArgs {
    seq_id: SeqId,
    share_access: u32,
    share_deny: u32,
    owner: OpenOwner,
    how: OpenFlag,
    claim: OpenClaim,
}

impl XdrSerialize for OpenArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.seq_id.serialize(dest)?;
        self.share_access.serialize(dest)?;
        self.share_deny.serialize(dest)?;
        self.owner.serialize(dest)?;
        self.how.serialize(dest)?;
        self.claim.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let seq_id = SeqId::deserialize(src)?;
        let share_access = u32::deserialize(src)?;
        let share_deny = u32::deserialize(src)?;
        let owner = OpenOwner::deserialize(src)?;
        let how = OpenFlag::deserialize(src)?;
        let claim = OpenClaim::deserialize(src)?;

        Ok(Self {
            seq_id,
            share_access,
            share_deny,
            owner,
            how,
            claim,
        })
    }
}

#[derive(Debug)]
pub struct OpenReadDelegation {
    state_id: StateId,
    recall: bool,
    permissions: NfsAce,
}

impl XdrSerialize for OpenReadDelegation {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;
        self.recall.serialize(dest)?;
        self.permissions.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenReadDelegation {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;
        let recall = bool::deserialize(src)?;
        let permissions = NfsAce::deserialize(src)?;

        Ok(Self {
            state_id,
            recall,
            permissions,
        })
    }
}

#[derive(Debug)]
pub struct OpenWriteDelegation {
    state_id: StateId,
    recall: bool,
    space_limit: NfsSpaceLimit,
    permissions: NfsAce,
}

impl XdrSerialize for OpenWriteDelegation {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;
        self.recall.serialize(dest)?;
        self.space_limit.serialize(dest)?;
        self.permissions.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenWriteDelegation {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;
        let recall = bool::deserialize(src)?;
        let space_limit = NfsSpaceLimit::deserialize(src)?;
        let permissions = NfsAce::deserialize(src)?;

        Ok(Self {
            state_id,
            recall,
            space_limit,
            permissions,
        })
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum WhyNoDelegation {
    NotWanted = 0,
    Contention = 1,
    Resource = 2,
    NotSupportedFileType = 3,
    WriteDelegationNotSupportedFileType = 4,
    NotSupportedUpgrade = 5,
    NotSupportedDowngrade = 6,
    Cancelled = 7,
    IsDirectory = 8,
}
xdr::serde_enum!(WhyNoDelegation);

#[derive(Debug)]
pub enum OpenNoneDelegation {
    NotWanted,
    Contention(bool),
    Resource(bool),
    NotSupportedFileType,
    WriteDelegationNotSupportedFileType,
    NotSupportedUpgrade,
    NotSupportedDowngrade,
    Cancelled,
    IsDirectory,
}

impl XdrSerialize for OpenNoneDelegation {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::NotWanted => {
                WhyNoDelegation::NotWanted.serialize(dest)?;
            }
            Self::Contention(will_signal) => {
                WhyNoDelegation::Contention.serialize(dest)?;
                will_signal.serialize(dest)?;
            }
            Self::Resource(will_signal) => {
                WhyNoDelegation::Resource.serialize(dest)?;
                will_signal.serialize(dest)?;
            }
            Self::NotSupportedFileType => {
                WhyNoDelegation::NotSupportedFileType.serialize(dest)?;
            }
            Self::WriteDelegationNotSupportedFileType => {
                WhyNoDelegation::WriteDelegationNotSupportedFileType
                    .serialize(dest)?;
            }
            Self::NotSupportedUpgrade => {
                WhyNoDelegation::NotSupportedUpgrade.serialize(dest)?;
            }
            Self::NotSupportedDowngrade => {
                WhyNoDelegation::NotSupportedDowngrade.serialize(dest)?;
            }
            Self::Cancelled => {
                WhyNoDelegation::Cancelled.serialize(dest)?;
            }
            Self::IsDirectory => {
                WhyNoDelegation::IsDirectory.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for OpenNoneDelegation {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let why_no_deleg = WhyNoDelegation::deserialize(src)?;

        let delegation = match why_no_deleg {
            WhyNoDelegation::NotWanted => Self::NotWanted,
            WhyNoDelegation::Contention => {
                let will_signal = bool::deserialize(src)?;
                Self::Contention(will_signal)
            }
            WhyNoDelegation::Resource => {
                let will_signal = bool::deserialize(src)?;
                Self::Resource(will_signal)
            }
            WhyNoDelegation::NotSupportedFileType => Self::NotSupportedFileType,
            WhyNoDelegation::WriteDelegationNotSupportedFileType => {
                Self::WriteDelegationNotSupportedFileType
            }
            WhyNoDelegation::NotSupportedUpgrade => Self::NotSupportedUpgrade,
            WhyNoDelegation::NotSupportedDowngrade => {
                Self::NotSupportedDowngrade
            }
            WhyNoDelegation::Cancelled => Self::Cancelled,
            WhyNoDelegation::IsDirectory => Self::IsDirectory,
        };

        Ok(delegation)
    }
}

#[derive(Debug)]
pub enum OpenDelegation {
    None,
    Read(OpenReadDelegation),
    Write(OpenWriteDelegation),
    NoneExt(OpenNoneDelegation),
}

impl XdrSerialize for OpenDelegation {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::None => {
                OpenDelegationType::None.serialize(dest)?;
            }
            Self::Read(delegation) => {
                OpenDelegationType::Read.serialize(dest)?;
                delegation.serialize(dest)?;
            }
            Self::Write(delegation) => {
                OpenDelegationType::Write.serialize(dest)?;
                delegation.serialize(dest)?;
            }
            Self::NoneExt(delegation) => {
                OpenDelegationType::NoneExt.serialize(dest)?;
                delegation.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for OpenDelegation {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let delegation_type = OpenDelegationType::deserialize(src)?;
        let delegation = match delegation_type {
            OpenDelegationType::None => Self::None,
            OpenDelegationType::Read => {
                let delegation = OpenReadDelegation::deserialize(src)?;
                Self::Read(delegation)
            }
            OpenDelegationType::Write => {
                let delegation = OpenWriteDelegation::deserialize(src)?;
                Self::Write(delegation)
            }
            OpenDelegationType::NoneExt => {
                let delegation = OpenNoneDelegation::deserialize(src)?;
                Self::NoneExt(delegation)
            }
        };

        Ok(delegation)
    }
}

#[derive(Debug)]
pub struct OpenResultFlags(u32);

impl OpenResultFlags {
    pub fn must_confirm(&self) -> bool {
        self.0 & OPEN_RESULT_CONFIRM != 0
    }

    pub fn is_posix_lock(&self) -> bool {
        self.0 & OPEN_RESULT_LOCKTYPE_POSIX != 0
    }

    pub fn will_preserve_unlinked(&self) -> bool {
        self.0 & OPEN_RESULT_PRESERVE_UNLINKED != 0
    }

    pub fn may_notify(&self) -> bool {
        self.0 & OPEN_RESULT_MAY_NOTIFY_LOCK != 0
    }
}

impl XdrSerialize for OpenResultFlags {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.0.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenResultFlags {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let flags = u32::deserialize(src)?;

        Ok(Self(flags))
    }
}

#[derive(Debug)]
pub struct OpenOk {
    state_id: StateId,
    change_info: ChangeInfo,
    rflags: OpenResultFlags,
    attrset: BitMap,
    delegation: OpenDelegation,
}

impl AsNfsStatus for OpenOk {}

impl XdrSerialize for OpenOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;
        self.change_info.serialize(dest)?;
        self.rflags.serialize(dest)?;
        self.attrset.serialize(dest)?;
        self.delegation.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;
        let change_info = ChangeInfo::deserialize(src)?;
        let rflags = OpenResultFlags::deserialize(src)?;
        let attrset = BitMap::deserialize(src)?;
        let delegation = OpenDelegation::deserialize(src)?;

        Ok(Self {
            state_id,
            change_info,
            rflags,
            attrset,
            delegation,
        })
    }
}

pub type OpenResult = Result<OpenOk, NfsStatus>;

#[derive(Debug)]
pub struct OpenAttributesArgs {
    create_dir: bool,
}

impl XdrSerialize for OpenAttributesArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.create_dir.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenAttributesArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let create_dir = bool::deserialize(src)?;

        Ok(Self { create_dir })
    }
}

pub type OpenAttributesResult = Result<NfsStatus, NfsStatus>;

pub struct OpenConfirmArgs {
    open_state_id: StateId,
    seq_id: SeqId,
}

impl XdrSerialize for OpenConfirmArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.open_state_id.serialize(dest)?;
        self.seq_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenConfirmArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let open_state_id = StateId::deserialize(src)?;
        let seq_id = SeqId::deserialize(src)?;

        Ok(Self {
            open_state_id,
            seq_id,
        })
    }
}

pub type OpenConfirmResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct OpenDowngradeArgs {
    open_state_id: StateId,
    seq_id: SeqId,
}

impl XdrSerialize for OpenDowngradeArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.open_state_id.serialize(dest)?;
        self.seq_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenDowngradeArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let open_state_id = StateId::deserialize(src)?;
        let seq_id = SeqId::deserialize(src)?;

        Ok(Self {
            open_state_id,
            seq_id,
        })
    }
}

#[derive(Debug)]
pub struct OpenDowngradeOk {
    open_state_id: StateId,
}

impl AsNfsStatus for OpenDowngradeOk {}

impl XdrSerialize for OpenDowngradeOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.open_state_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for OpenDowngradeOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let open_state_id = StateId::deserialize(src)?;

        Ok(Self { open_state_id })
    }
}

pub type OpenDowngradeResult = Result<OpenDowngradeOk, NfsStatus>;

#[derive(Debug)]
pub struct PutFhArgs {
    pub object: NfsFh,
}

impl XdrSerialize for PutFhArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.object.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for PutFhArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let object = NfsFh::deserialize(src)?;

        Ok(Self { object })
    }
}

pub type PutFhResult = Result<NfsStatus, NfsStatus>;

pub type PutPublicFhResult = Result<NfsStatus, NfsStatus>;

pub type PutRootFhResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct ReadArgs {
    state_id: StateId,
    offset: Offset,
    count: Count,
}

impl XdrSerialize for ReadArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;
        self.offset.serialize(dest)?;
        self.count.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ReadArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;
        let offset = Offset::deserialize(src)?;
        let count = Count::deserialize(src)?;

        Ok(Self {
            state_id,
            offset,
            count,
        })
    }
}

#[derive(Debug)]
pub struct ReadOk {
    eof: bool,
    data: XdrOpaque,
}

impl AsNfsStatus for ReadOk {}

impl XdrSerialize for ReadOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.eof.serialize(dest)?;
        self.data.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ReadOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let eof = bool::deserialize(src)?;
        let data = XdrOpaque::deserialize(src)?;

        Ok(Self { eof, data })
    }
}

pub type ReadResult = Result<ReadOk, NfsStatus>;

#[derive(Debug)]
pub struct ReadDirectoryArgs {
    cookie: NfsCookie,
    cookie_verifier: Verifier,
    directory_count: Count,
    max_count: Count,
    attr_request: BitMap,
}

impl XdrSerialize for ReadDirectoryArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.cookie.serialize(dest)?;
        self.cookie_verifier.serialize(dest)?;
        self.directory_count.serialize(dest)?;
        self.max_count.serialize(dest)?;
        self.attr_request.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ReadDirectoryArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let cookie = NfsCookie::deserialize(src)?;
        let cookie_verifier = Verifier::deserialize(src)?;
        let directory_count = Count::deserialize(src)?;
        let max_count = Count::deserialize(src)?;
        let attr_request = BitMap::deserialize(src)?;

        Ok(Self {
            cookie,
            cookie_verifier,
            directory_count,
            max_count,
            attr_request,
        })
    }
}

#[derive(Debug)]
pub struct Entry {
    cookie: NfsCookie,
    name: Component,
    attrs: FileAttrs,
}

impl XdrSerialize for Entry {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.cookie.serialize(dest)?;
        self.name.serialize(dest)?;
        self.attrs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for Entry {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let cookie = NfsCookie::deserialize(src)?;
        let name = Component::deserialize(src)?;
        let attrs = FileAttrs::deserialize(src)?;

        Ok(Self {
            cookie,
            name,
            attrs,
        })
    }
}

#[derive(Debug)]
pub struct DirectoryList {
    entries: xdr::OptionalData<Entry>,
    eof: bool,
}

impl XdrSerialize for DirectoryList {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.entries.serialize(dest)?;
        self.eof.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for DirectoryList {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let entries = <xdr::OptionalData<Entry>>::deserialize(src)?;
        let eof = bool::deserialize(src)?;

        Ok(Self { entries, eof })
    }
}

#[derive(Debug)]
pub struct ReadDirectoryOk {
    cookie_verifier: Verifier,
    reply: DirectoryList,
}

impl AsNfsStatus for ReadDirectoryOk {}

impl XdrSerialize for ReadDirectoryOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.cookie_verifier.serialize(dest)?;
        self.reply.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ReadDirectoryOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let cookie_verifier = Verifier::deserialize(src)?;
        let reply = DirectoryList::deserialize(src)?;

        Ok(Self {
            cookie_verifier,
            reply,
        })
    }
}

pub type ReadDirectoryResult = Result<ReadDirectoryOk, NfsStatus>;

#[derive(Debug)]
pub struct ReadLinkOk {
    link: LinkText,
}

impl AsNfsStatus for ReadLinkOk {}

impl XdrSerialize for ReadLinkOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.link.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ReadLinkOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let link = LinkText::deserialize(src)?;

        Ok(Self { link })
    }
}

pub type ReadLinkResult = Result<ReadLinkOk, NfsStatus>;

#[derive(Debug)]
pub struct RemoveArgs {
    target: Component,
}

impl XdrSerialize for RemoveArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.target.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for RemoveArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let target = Component::deserialize(src)?;

        Ok(Self { target })
    }
}

#[derive(Debug)]
pub struct RemoveOk {
    change_info: ChangeInfo,
}

impl AsNfsStatus for RemoveOk {}

impl XdrSerialize for RemoveOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.change_info.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for RemoveOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let change_info = ChangeInfo::deserialize(src)?;

        Ok(Self { change_info })
    }
}

pub type RemoveResult = Result<RemoveOk, NfsStatus>;

#[derive(Debug)]
pub struct RenameArgs {
    old_name: Component,
    new_name: Component,
}

impl XdrSerialize for RenameArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.old_name.serialize(dest)?;
        self.new_name.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for RenameArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let old_name = Component::deserialize(src)?;
        let new_name = Component::deserialize(src)?;

        Ok(Self { old_name, new_name })
    }
}

#[derive(Debug)]
pub struct RenameOk {
    source_change_info: ChangeInfo,
    target_change_info: ChangeInfo,
}

impl AsNfsStatus for RenameOk {}

impl XdrSerialize for RenameOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.source_change_info.serialize(dest)?;
        self.target_change_info.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for RenameOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let source_change_info = ChangeInfo::deserialize(src)?;
        let target_change_info = ChangeInfo::deserialize(src)?;

        Ok(Self {
            source_change_info,
            target_change_info,
        })
    }
}

pub type RenameResult = Result<RenameOk, NfsStatus>;

/// Obsolete in v4.1
pub struct RenewArgs {
    client_id: ClientId,
}

impl XdrSerialize for RenewArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for RenewArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_id = ClientId::deserialize(src)?;

        Ok(Self { client_id })
    }
}

pub type RenewResult = Result<NfsStatus, NfsStatus>;

pub type RestoreFhResult = Result<NfsStatus, NfsStatus>;

pub type SaveFhResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct SecurityInfoArgs {
    name: Component,
}

impl XdrSerialize for SecurityInfoArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.name.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SecurityInfoArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let name = Component::deserialize(src)?;

        Ok(Self { name })
    }
}

/// From RFC 2203
#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum RpcGssSvc {
    None = 1,
    Integrity = 2,
    Privacy = 3,
}
xdr::serde_enum!(RpcGssSvc);

#[derive(Debug)]
pub struct RpcSecGssInfo {
    oid: SecOid,
    qop: QOp,
    service: RpcGssSvc,
}

impl XdrSerialize for RpcSecGssInfo {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.oid.serialize(dest)?;
        self.qop.serialize(dest)?;
        self.service.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for RpcSecGssInfo {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let oid = SecOid::deserialize(src)?;
        let qop = QOp::deserialize(src)?;
        let service = RpcGssSvc::deserialize(src)?;

        Ok(Self { oid, qop, service })
    }
}

#[derive(Debug)]
pub enum SecurityInfo {
    RpcSecGss(RpcSecGssInfo),
    Unknown,
}

impl XdrSerialize for SecurityInfo {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::RpcSecGss(info) => {
                RPCSEC_GSS.serialize(dest)?;
                info.serialize(dest)?;
            }
            Self::Unknown => {
                // TODO: Figure this out.
                0u32.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for SecurityInfo {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let flavor = u32::deserialize(src)?;
        let secinfo = if flavor == RPCSEC_GSS {
            let info = RpcSecGssInfo::deserialize(src)?;
            Self::RpcSecGss(info)
        } else {
            Self::Unknown
        };

        Ok(secinfo)
    }
}

#[derive(Debug)]
pub struct SecurityInfoOk(Vec<SecurityInfo>);

impl AsNfsStatus for SecurityInfoOk {}

impl XdrSerialize for SecurityInfoOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        xdr::serialize_vec(dest, &self.0)?;

        Ok(())
    }
}

impl XdrDeserialize for SecurityInfoOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let infos = xdr::deserialize_vec::<_, SecurityInfo>(src)?;

        Ok(Self(infos))
    }
}

pub type SecurityInfoResult = Result<SecurityInfoOk, NfsStatus>;

pub struct SetAttributesArgs {
    state_id: StateId,
    object_attrs: FileAttrs,
}

impl XdrSerialize for SetAttributesArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;
        self.object_attrs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SetAttributesArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;
        let object_attrs = FileAttrs::deserialize(src)?;

        Ok(Self {
            state_id,
            object_attrs,
        })
    }
}

pub struct SetAttributesResult {
    status: NfsStatus,
    attrsset: BitMap,
}

impl AsNfsStatus for SetAttributesResult {
    fn as_status(&self) -> NfsStatus {
        self.status
    }
}

impl XdrSerialize for SetAttributesResult {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.attrsset.serialize(dest)?;

        Ok(())
    }
}

/// Obsolete in NFSv4.1
pub struct SetClientIdArgs {
    client: NfsClientId,
    callback: CallbackClient,
    callback_ident: u32,
}

impl XdrSerialize for SetClientIdArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client.serialize(dest)?;
        self.callback.serialize(dest)?;
        self.callback_ident.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SetClientIdArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client = NfsClientId::deserialize(src)?;
        let callback = CallbackClient::deserialize(src)?;
        let callback_ident = u32::deserialize(src)?;

        Ok(Self {
            client,
            callback,
            callback_ident,
        })
    }
}

pub struct SetClientIdOk {
    client_id: ClientId,
    set_client_id_confirm: Verifier,
}

impl XdrSerialize for SetClientIdOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_id.serialize(dest)?;
        self.set_client_id_confirm.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SetClientIdOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_id = ClientId::deserialize(src)?;
        let set_client_id_confirm = Verifier::deserialize(src)?;

        Ok(Self {
            client_id,
            set_client_id_confirm,
        })
    }
}

pub type SetClientIdResult = Result<SetClientIdOk, NfsStatus>;

/// Obsolete in NFSv4.1
pub struct SetClientIdConfirmArgs {
    client_id: ClientId,
    set_client_id_confirm: Verifier,
}

impl XdrSerialize for SetClientIdConfirmArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_id.serialize(dest)?;
        self.set_client_id_confirm.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SetClientIdConfirmArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_id = ClientId::deserialize(src)?;
        let set_client_id_confirm = Verifier::deserialize(src)?;

        Ok(Self {
            client_id,
            set_client_id_confirm,
        })
    }
}

pub type SetClientIdConfirmResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct VerifyArgs {
    object_attrs: FileAttrs,
}

impl XdrSerialize for VerifyArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.object_attrs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for VerifyArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let object_attrs = FileAttrs::deserialize(src)?;

        Ok(Self { object_attrs })
    }
}

pub type VerifyResult = Result<NfsStatus, NfsStatus>;

/// From RFC 2203
#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum StableHow {
    Unstable = 0,
    DataSync = 1,
    FileSync = 2,
}
xdr::serde_enum!(StableHow);

#[derive(Debug)]
pub struct WriteArgs {
    state_id: StateId,
    offset: Offset,
    stable: StableHow,
    data: XdrOpaque,
}

impl XdrSerialize for WriteArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;
        self.offset.serialize(dest)?;
        self.stable.serialize(dest)?;
        self.data.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for WriteArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;
        let offset = Offset::deserialize(src)?;
        let stable = StableHow::deserialize(src)?;
        let data = XdrOpaque::deserialize(src)?;

        Ok(Self {
            state_id,
            offset,
            stable,
            data,
        })
    }
}

#[derive(Debug)]
pub struct WriteOk {
    count: Count,
    committed: StableHow,
    write_verifier: Verifier,
}

impl AsNfsStatus for WriteOk {}

impl XdrSerialize for WriteOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.count.serialize(dest)?;
        self.committed.serialize(dest)?;
        self.write_verifier.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for WriteOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let count = Count::deserialize(src)?;
        let committed = StableHow::deserialize(src)?;
        let write_verifier = Verifier::deserialize(src)?;

        Ok(Self {
            count,
            committed,
            write_verifier,
        })
    }
}

pub type WriteResult = Result<WriteOk, NfsStatus>;

/// Obsolete in NFSv4.1
pub struct ReleaseLockOwnerArgs {
    lock_owner: LockOwner,
}

impl XdrSerialize for ReleaseLockOwnerArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.lock_owner.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ReleaseLockOwnerArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let lock_owner = LockOwner::deserialize(src)?;

        Ok(Self { lock_owner })
    }
}

pub type ReleaseLockOwnerResult = Result<NfsStatus, NfsStatus>;

pub type IllegalInstructionResult = Result<NfsStatus, NfsStatus>;

pub type GssHandle = XdrOpaque;

#[derive(Clone, Debug)]
pub struct GssCallbackHandles {
    gcbp_service: RpcGssSvc,
    gcbp_handle_from_server: GssHandle,
    gcbp_handle_from_client: GssHandle,
}

impl XdrSerialize for GssCallbackHandles {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.gcbp_service.serialize(dest)?;
        self.gcbp_handle_from_server.serialize(dest)?;
        self.gcbp_handle_from_client.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GssCallbackHandles {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let gcbp_service = RpcGssSvc::deserialize(src)?;
        let gcbp_handle_from_server = GssHandle::deserialize(src)?;
        let gcbp_handle_from_client = GssHandle::deserialize(src)?;

        Ok(Self {
            gcbp_service,
            gcbp_handle_from_server,
            gcbp_handle_from_client,
        })
    }
}

#[derive(Clone, Debug)]
pub enum CallbackSecurityParameters {
    None,
    Sys(AuthUnix),
    RpcSecGss(GssCallbackHandles),
}

impl XdrSerialize for CallbackSecurityParameters {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::None => {
                AuthFlavor::Null.serialize(dest)?;
            }
            Self::Sys(params) => {
                AuthFlavor::Unix.serialize(dest)?;
                params.serialize(dest)?;
            }
            Self::RpcSecGss(handles) => {
                AuthFlavor::RpcSecGss.serialize(dest)?;
                handles.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for CallbackSecurityParameters {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let flavor = AuthFlavor::deserialize(src)?;

        let params = match flavor {
            AuthFlavor::Null => Self::None,
            AuthFlavor::Unix => {
                let auth = AuthUnix::deserialize(src)?;
                Self::Sys(auth)
            }
            AuthFlavor::RpcSecGss => {
                let handles = GssCallbackHandles::deserialize(src)?;
                Self::RpcSecGss(handles)
            }
            AuthFlavor::Short | AuthFlavor::Des => {
                return Err(std::io::Error::other(format!(
                    "Invalid authentication flavor for CallbackSecurityParameters: {flavor:?}"
                )));
            }
        };

        Ok(params)
    }
}

#[derive(Debug)]
pub struct BackchannelControlArgs {
    callback_program: u32,
    security_parameters: CallbackSecurityParameters,
}

impl XdrSerialize for BackchannelControlArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.callback_program.serialize(dest)?;
        self.security_parameters.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for BackchannelControlArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let callback_program = u32::deserialize(src)?;
        let security_parameters = CallbackSecurityParameters::deserialize(src)?;

        Ok(Self {
            callback_program,
            security_parameters,
        })
    }
}

pub type BackchannelControlResult = Result<NfsStatus, NfsStatus>;

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum ChannelDirectionFromClient {
    Fore = 1,
    Back = 2,
    ForeOrBoth = 3,
    BackOrBoth = 7,
}
xdr::serde_enum!(ChannelDirectionFromClient);

#[derive(Debug)]
pub struct BindConnectionToSessionArgs {
    session_id: SessionId,
    channel_direction_from_client: ChannelDirectionFromClient,
    use_conn_in_rdma: bool,
}

impl XdrSerialize for BindConnectionToSessionArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.session_id.serialize(dest)?;
        self.channel_direction_from_client.serialize(dest)?;
        self.use_conn_in_rdma.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for BindConnectionToSessionArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let session_id = SessionId::deserialize(src)?;
        let channel_direction_from_client =
            ChannelDirectionFromClient::deserialize(src)?;
        let use_conn_in_rdma = bool::deserialize(src)?;

        Ok(Self {
            session_id,
            channel_direction_from_client,
            use_conn_in_rdma,
        })
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum ChannelDirectionFromServer {
    Fore = 1,
    Back = 2,
    Both = 3,
}
xdr::serde_enum!(ChannelDirectionFromServer);

#[derive(Debug)]
pub struct BindConnectionToSessionOk {
    session_id: SessionId,
    channel_direction_from_server: ChannelDirectionFromServer,
    use_conn_in_rdma: bool,
}

impl AsNfsStatus for BindConnectionToSessionOk {}

impl XdrSerialize for BindConnectionToSessionOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.session_id.serialize(dest)?;
        self.channel_direction_from_server.serialize(dest)?;
        self.use_conn_in_rdma.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for BindConnectionToSessionOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let session_id = SessionId::deserialize(src)?;
        let channel_direction_from_server =
            ChannelDirectionFromServer::deserialize(src)?;
        let use_conn_in_rdma = bool::deserialize(src)?;

        Ok(Self {
            session_id,
            channel_direction_from_server,
            use_conn_in_rdma,
        })
    }
}

pub type BindConnectionToSessionResult =
    Result<BindConnectionToSessionOk, NfsStatus>;

#[derive(Debug)]
pub struct StateProtectOps {
    must_enforce: BitMap,
    must_allow: BitMap,
}

impl XdrSerialize for StateProtectOps {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.must_enforce.serialize(dest)?;
        self.must_allow.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for StateProtectOps {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let must_enforce = BitMap::deserialize(src)?;
        let must_allow = BitMap::deserialize(src)?;

        Ok(Self {
            must_enforce,
            must_allow,
        })
    }
}

#[derive(Debug)]
pub struct SsvStateProtectParameters {
    ops: StateProtectOps,
    hash_algorithms: Vec<SecOid>,
    encryption_algorithms: Vec<SecOid>,
    window: u32,
    num_gss_handles: u32,
}

impl XdrSerialize for SsvStateProtectParameters {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.ops.serialize(dest)?;
        xdr::serialize_vec(dest, &self.hash_algorithms)?;
        xdr::serialize_vec(dest, &self.encryption_algorithms)?;
        self.window.serialize(dest)?;
        self.num_gss_handles.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SsvStateProtectParameters {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let ops = StateProtectOps::deserialize(src)?;
        let hash_algorithms = xdr::deserialize_vec::<_, SecOid>(src)?;
        let encryption_algorithms = xdr::deserialize_vec::<_, SecOid>(src)?;
        let window = u32::deserialize(src)?;
        let num_gss_handles = u32::deserialize(src)?;

        Ok(Self {
            ops,
            hash_algorithms,
            encryption_algorithms,
            window,
            num_gss_handles,
        })
    }
}

#[derive(Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum StateProtectHow {
    None = 0,
    MachineCredentials = 1,
    Ssv = 2,
}
xdr::serde_enum!(StateProtectHow);

#[derive(Debug)]
pub enum StateProtectionArg {
    None,
    MachineCredentials(StateProtectOps),
    Ssv(SsvStateProtectParameters),
}

impl XdrSerialize for StateProtectionArg {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::None => {
                StateProtectHow::None.serialize(dest)?;
            }
            Self::MachineCredentials(ops) => {
                StateProtectHow::MachineCredentials.serialize(dest)?;
                ops.serialize(dest)?;
            }
            Self::Ssv(parameters) => {
                StateProtectHow::Ssv.serialize(dest)?;
                parameters.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for StateProtectionArg {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let how = StateProtectHow::deserialize(src)?;
        let arg = match how {
            StateProtectHow::None => Self::None,
            StateProtectHow::MachineCredentials => {
                let ops = StateProtectOps::deserialize(src)?;
                Self::MachineCredentials(ops)
            }
            StateProtectHow::Ssv => {
                let parameters = SsvStateProtectParameters::deserialize(src)?;
                Self::Ssv(parameters)
            }
        };

        Ok(arg)
    }
}

#[derive(Debug)]
pub struct ExchangeIdArgs {
    pub client_owner: ClientOwner,
    pub flags: u32,
    pub state_protect: StateProtectionArg,
    pub client_impl_id: Option<NfsImplId>,
}

impl XdrSerialize for ExchangeIdArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_owner.serialize(dest)?;
        self.flags.serialize(dest)?;
        self.state_protect.serialize(dest)?;
        self.client_impl_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ExchangeIdArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_owner = ClientOwner::deserialize(src)?;
        let flags = u32::deserialize(src)?;
        let state_protect = StateProtectionArg::deserialize(src)?;
        let client_impl_id = <Option<NfsImplId>>::deserialize(src)?;

        Ok(Self {
            client_owner,
            flags,
            state_protect,
            client_impl_id,
        })
    }
}

#[derive(Debug)]
pub struct SsvProtectionInfo {
    ops: StateProtectOps,
    hash_algorithm: u32,
    encryption_algorithm: u32,
    ssv_len: u32,
    window: u32,
    handles: Vec<GssHandle>,
}

impl XdrSerialize for SsvProtectionInfo {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.ops.serialize(dest)?;
        self.hash_algorithm.serialize(dest)?;
        self.encryption_algorithm.serialize(dest)?;
        self.ssv_len.serialize(dest)?;
        self.window.serialize(dest)?;
        xdr::serialize_vec(dest, &self.handles)?;

        Ok(())
    }
}

impl XdrDeserialize for SsvProtectionInfo {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let ops = StateProtectOps::deserialize(src)?;
        let hash_algorithm = u32::deserialize(src)?;
        let encryption_algorithm = u32::deserialize(src)?;
        let ssv_len = u32::deserialize(src)?;
        let window = u32::deserialize(src)?;
        let handles = xdr::deserialize_vec::<_, GssHandle>(src)?;

        Ok(Self {
            ops,
            hash_algorithm,
            encryption_algorithm,
            ssv_len,
            window,
            handles,
        })
    }
}

#[derive(Debug)]
pub enum StateProtectionResult {
    None,
    MachineCredentials(StateProtectOps),
    Ssv(SsvProtectionInfo),
}

impl XdrSerialize for StateProtectionResult {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::None => {
                StateProtectHow::None.serialize(dest)?;
            }
            Self::MachineCredentials(ops) => {
                StateProtectHow::MachineCredentials.serialize(dest)?;
                ops.serialize(dest)?;
            }
            Self::Ssv(info) => {
                StateProtectHow::Ssv.serialize(dest)?;
                info.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for StateProtectionResult {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let how = StateProtectHow::deserialize(src)?;
        let result = match how {
            StateProtectHow::None => Self::None,
            StateProtectHow::MachineCredentials => {
                let ops = StateProtectOps::deserialize(src)?;
                Self::MachineCredentials(ops)
            }
            StateProtectHow::Ssv => {
                let info = SsvProtectionInfo::deserialize(src)?;
                Self::Ssv(info)
            }
        };

        Ok(result)
    }
}

#[derive(Debug)]
pub struct ExchangeIdOk {
    pub client_id: ClientId,
    pub sequence_id: SequenceId,
    pub flags: u32,
    pub state_protection: StateProtectionResult,
    pub server_owner: ServerOwner,
    pub server_scope: MaxLenBytes<NFS_OPAQUE_LIMIT>,
    pub server_impl_id: Option<NfsImplId>,
}

impl AsNfsStatus for ExchangeIdOk {}

impl XdrSerialize for ExchangeIdOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_id.serialize(dest)?;
        self.sequence_id.serialize(dest)?;
        self.flags.serialize(dest)?;
        self.state_protection.serialize(dest)?;
        self.server_owner.serialize(dest)?;
        self.server_scope.serialize(dest)?;
        self.server_impl_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ExchangeIdOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_id = ClientId::deserialize(src)?;
        let sequence_id = SequenceId::deserialize(src)?;
        let flags = u32::deserialize(src)?;
        let state_protection = StateProtectionResult::deserialize(src)?;
        let server_owner = ServerOwner::deserialize(src)?;
        let server_scope = <MaxLenBytes<NFS_OPAQUE_LIMIT>>::deserialize(src)?;
        let server_impl_id = <Option<NfsImplId>>::deserialize(src)?;

        Ok(Self {
            client_id,
            sequence_id,
            flags,
            state_protection,
            server_owner,
            server_scope,
            server_impl_id,
        })
    }
}

pub type ExchangeIdResult = Result<ExchangeIdOk, NfsStatus>;

#[derive(Clone, Debug)]
pub struct ChannelAttrs {
    pub header_pad_size: Count,
    pub max_request_size: Count,
    pub max_response_size: Count,
    pub max_response_size_cached: Count,
    pub max_operations: Count,
    pub max_requests: Count,
    pub rdma_ird: Option<u32>,
}

impl XdrSerialize for ChannelAttrs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.header_pad_size.serialize(dest)?;
        self.max_request_size.serialize(dest)?;
        self.max_response_size.serialize(dest)?;
        self.max_response_size_cached.serialize(dest)?;
        self.max_operations.serialize(dest)?;
        self.max_requests.serialize(dest)?;
        self.rdma_ird.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ChannelAttrs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let header_pad_size = Count::deserialize(src)?;
        let max_request_size = Count::deserialize(src)?;
        let max_response_size = Count::deserialize(src)?;
        let max_response_size_cached = Count::deserialize(src)?;
        let max_operations = Count::deserialize(src)?;
        let max_requests = Count::deserialize(src)?;
        let rdma_ird = <Option<u32>>::deserialize(src)?;

        Ok(Self {
            header_pad_size,
            max_request_size,
            max_response_size,
            max_response_size_cached,
            max_operations,
            max_requests,
            rdma_ird,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CreateSessionArgs {
    pub client_id: ClientId,
    pub sequence: SequenceId,
    pub flags: u32,
    pub fore_channel_attrs: ChannelAttrs,
    pub back_channel_attrs: ChannelAttrs,
    pub callback_program: u32,
    pub security_parameters: Vec<CallbackSecurityParameters>,
}

impl XdrSerialize for CreateSessionArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_id.serialize(dest)?;
        self.sequence.serialize(dest)?;
        self.flags.serialize(dest)?;
        self.fore_channel_attrs.serialize(dest)?;
        self.back_channel_attrs.serialize(dest)?;
        self.callback_program.serialize(dest)?;
        xdr::serialize_vec(dest, &self.security_parameters)?;

        Ok(())
    }
}

impl XdrDeserialize for CreateSessionArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_id = ClientId::deserialize(src)?;
        let sequence = SequenceId::deserialize(src)?;
        let flags = u32::deserialize(src)?;
        let fore_channel_attrs = ChannelAttrs::deserialize(src)?;
        let back_channel_attrs = ChannelAttrs::deserialize(src)?;
        let callback_program = u32::deserialize(src)?;
        let security_parameters =
            xdr::deserialize_vec::<_, CallbackSecurityParameters>(src)?;

        Ok(Self {
            client_id,
            sequence,
            flags,
            fore_channel_attrs,
            back_channel_attrs,
            callback_program,
            security_parameters,
        })
    }
}

#[derive(Debug)]
pub struct CreateSessionOk {
    pub session_id: SessionId,
    pub sequence: SequenceId,
    pub flags: u32,
    pub fore_channel_attrs: ChannelAttrs,
    pub back_channel_attrs: ChannelAttrs,
}

impl AsNfsStatus for CreateSessionOk {}

impl XdrSerialize for CreateSessionOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.session_id.serialize(dest)?;
        self.sequence.serialize(dest)?;
        self.flags.serialize(dest)?;
        self.fore_channel_attrs.serialize(dest)?;
        self.back_channel_attrs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for CreateSessionOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let session_id = SessionId::deserialize(src)?;
        let sequence = SequenceId::deserialize(src)?;
        let flags = u32::deserialize(src)?;
        let fore_channel_attrs = ChannelAttrs::deserialize(src)?;
        let back_channel_attrs = ChannelAttrs::deserialize(src)?;

        Ok(Self {
            session_id,
            sequence,
            flags,
            fore_channel_attrs,
            back_channel_attrs,
        })
    }
}

pub type CreateSessionResult = Result<CreateSessionOk, NfsStatus>;

#[derive(Debug)]
pub struct DestroySessionArgs {
    pub session_id: SessionId,
}

impl XdrSerialize for DestroySessionArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.session_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for DestroySessionArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let session_id = SessionId::deserialize(src)?;

        Ok(Self { session_id })
    }
}

pub type DestroySessionResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct FreeStateIdArgs {
    state_id: StateId,
}

impl XdrSerialize for FreeStateIdArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.state_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for FreeStateIdArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_id = StateId::deserialize(src)?;

        Ok(Self { state_id })
    }
}

pub type FreeStateIdResult = Result<NfsStatus, NfsStatus>;

pub type AttributeNotice = NfsTime;

#[derive(Debug)]
pub struct GetDirectoryDelegationArgs {
    signal_delegation_available: bool,
    notification_types: BitMap,
    child_attribute_delay: AttributeNotice,
    directory_attribute_delay: AttributeNotice,
    child_attributes: BitMap,
    directory_attributes: BitMap,
}

impl XdrSerialize for GetDirectoryDelegationArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.signal_delegation_available.serialize(dest)?;
        self.notification_types.serialize(dest)?;
        self.child_attribute_delay.serialize(dest)?;
        self.directory_attribute_delay.serialize(dest)?;
        self.child_attributes.serialize(dest)?;
        self.directory_attributes.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetDirectoryDelegationArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let signal_delegation_available = bool::deserialize(src)?;
        let notification_types = BitMap::deserialize(src)?;
        let child_attribute_delay = AttributeNotice::deserialize(src)?;
        let directory_attribute_delay = AttributeNotice::deserialize(src)?;
        let child_attributes = BitMap::deserialize(src)?;
        let directory_attributes = BitMap::deserialize(src)?;

        Ok(Self {
            signal_delegation_available,
            notification_types,
            child_attribute_delay,
            directory_attribute_delay,
            child_attributes,
            directory_attributes,
        })
    }
}

#[derive(Debug)]
pub struct GetDirectoryDelegationOk {
    cookie_verifier: Verifier,
    state_id: StateId,
    notification: BitMap,
    child_attributes: BitMap,
}

impl XdrSerialize for GetDirectoryDelegationOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.cookie_verifier.serialize(dest)?;
        self.state_id.serialize(dest)?;
        self.notification.serialize(dest)?;
        self.child_attributes.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetDirectoryDelegationOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let cookie_verifier = Verifier::deserialize(src)?;
        let state_id = StateId::deserialize(src)?;
        let notification = BitMap::deserialize(src)?;
        let child_attributes = BitMap::deserialize(src)?;

        Ok(Self {
            cookie_verifier,
            state_id,
            notification,
            child_attributes,
        })
    }
}

#[derive(Clone, Copy, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum GetDirectoryDelegationStatus {
    Ok = 0,
    Unavailable = 1,
}
xdr::serde_enum!(GetDirectoryDelegationStatus);

#[derive(Debug)]
pub enum GetDirectoryDelegationNonFatal {
    Ok(GetDirectoryDelegationOk),
    Unavailable(bool),
}

impl AsNfsStatus for GetDirectoryDelegationNonFatal {
    fn as_status(&self) -> NfsStatus {
        match self {
            Self::Ok(_) => NfsStatus::Ok,
            Self::Unavailable(_) => NfsStatus::DirDelegationUnavailable,
        }
    }
}

impl XdrSerialize for GetDirectoryDelegationNonFatal {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Ok(ok) => {
                GetDirectoryDelegationStatus::Ok.serialize(dest)?;
                ok.serialize(dest)?;
            }
            Self::Unavailable(will_signal) => {
                GetDirectoryDelegationStatus::Unavailable.serialize(dest)?;
                will_signal.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for GetDirectoryDelegationNonFatal {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let status = GetDirectoryDelegationStatus::deserialize(src)?;
        let result = match status {
            GetDirectoryDelegationStatus::Ok => {
                let ok = GetDirectoryDelegationOk::deserialize(src)?;
                Self::Ok(ok)
            }
            GetDirectoryDelegationStatus::Unavailable => {
                let will_signal = bool::deserialize(src)?;
                Self::Unavailable(will_signal)
            }
        };

        Ok(result)
    }
}

pub type GetDirectoryDelegationResult =
    Result<GetDirectoryDelegationNonFatal, NfsStatus>;

#[derive(Debug)]
pub struct GetDeviceInfoArgs {
    device_id: DeviceId,
    layout_type: NfsLayoutType,
    max_count: Count,
    notify_types: BitMap,
}

impl XdrSerialize for GetDeviceInfoArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.device_id.serialize(dest)?;
        self.layout_type.serialize(dest)?;
        self.max_count.serialize(dest)?;
        self.notify_types.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetDeviceInfoArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let device_id = DeviceId::deserialize(src)?;
        let layout_type = NfsLayoutType::deserialize(src)?;
        let max_count = Count::deserialize(src)?;
        let notify_types = BitMap::deserialize(src)?;

        Ok(Self {
            device_id,
            layout_type,
            max_count,
            notify_types,
        })
    }
}

#[derive(Debug)]
pub struct GetDeviceInfoOk {
    device_address: DeviceAddress,
    notification: BitMap,
}

impl XdrSerialize for GetDeviceInfoOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.device_address.serialize(dest)?;
        self.notification.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetDeviceInfoOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let device_address = DeviceAddress::deserialize(src)?;
        let notification = BitMap::deserialize(src)?;

        Ok(Self {
            device_address,
            notification,
        })
    }
}

#[derive(Debug)]
pub enum GetDeviceInfoReturn {
    Ok(GetDeviceInfoOk),
    TooSmall(Count),
}

impl AsNfsStatus for GetDeviceInfoReturn {
    fn as_status(&self) -> NfsStatus {
        match self {
            Self::Ok(_) => NfsStatus::Ok,
            Self::TooSmall(_) => NfsStatus::TooSmall,
        }
    }
}

impl XdrSerialize for GetDeviceInfoReturn {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Ok(ok) => ok.serialize(dest),
            Self::TooSmall(count) => count.serialize(dest),
        }
    }
}

pub type GetDeviceInfoResult = Result<GetDeviceInfoReturn, NfsStatus>;

#[derive(Debug)]
pub struct GetDeviceListArgs {
    layout_type: NfsLayoutType,
    max_devices: Count,
    cookie: NfsCookie,
    cookie_verifier: Verifier,
}

impl XdrSerialize for GetDeviceListArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.layout_type.serialize(dest)?;
        self.max_devices.serialize(dest)?;
        self.cookie.serialize(dest)?;
        self.cookie_verifier.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetDeviceListArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let layout_type = NfsLayoutType::deserialize(src)?;
        let max_devices = Count::deserialize(src)?;
        let cookie = NfsCookie::deserialize(src)?;
        let cookie_verifier = Verifier::deserialize(src)?;

        Ok(Self {
            layout_type,
            max_devices,
            cookie,
            cookie_verifier,
        })
    }
}

#[derive(Debug)]
pub struct GetDeviceListOk {
    cookie: NfsCookie,
    cookie_verifier: Verifier,
    device_ids: Vec<DeviceId>,
    eof: bool,
}

impl AsNfsStatus for GetDeviceListOk {}

impl XdrSerialize for GetDeviceListOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.cookie.serialize(dest)?;
        self.cookie_verifier.serialize(dest)?;
        xdr::serialize_vec(dest, &self.device_ids)?;
        self.eof.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for GetDeviceListOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let cookie = NfsCookie::deserialize(src)?;
        let cookie_verifier = Verifier::deserialize(src)?;
        let device_ids = xdr::deserialize_vec::<_, DeviceId>(src)?;
        let eof = bool::deserialize(src)?;

        Ok(Self {
            cookie,
            cookie_verifier,
            device_ids,
            eof,
        })
    }
}

pub type GetDeviceListResult = Result<GetDeviceListOk, NfsStatus>;

#[derive(Debug)]
pub enum NewTime {
    Changed(NfsTime),
    Unchanged,
}

impl XdrSerialize for NewTime {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Changed(time) => {
                true.serialize(dest)?;
                time.serialize(dest)?;
            }
            Self::Unchanged => {
                false.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for NewTime {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let changed = bool::deserialize(src)?;

        if changed {
            let time = NfsTime::deserialize(src)?;
            Ok(Self::Changed(time))
        } else {
            Ok(Self::Unchanged)
        }
    }
}

#[derive(Debug)]
pub enum NewOffset {
    Changed(Offset),
    Unchanged,
}

impl XdrSerialize for NewOffset {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Changed(offset) => {
                true.serialize(dest)?;
                offset.serialize(dest)?;
            }
            Self::Unchanged => {
                false.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for NewOffset {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let changed = bool::deserialize(src)?;

        if changed {
            let offset = Offset::deserialize(src)?;
            Ok(Self::Changed(offset))
        } else {
            Ok(Self::Unchanged)
        }
    }
}

#[derive(Debug)]
pub struct LayoutCommitArgs {
    offset: Offset,
    length: Length,
    state_id: StateId,
    last_write_offset: NewOffset,
    time_modify: NewTime,
    layout_update: LayoutUpdate,
}

impl XdrSerialize for LayoutCommitArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.offset.serialize(dest)?;
        self.length.serialize(dest)?;
        self.state_id.serialize(dest)?;
        self.last_write_offset.serialize(dest)?;
        self.time_modify.serialize(dest)?;
        self.layout_update.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LayoutCommitArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let offset = Offset::deserialize(src)?;
        let length = Length::deserialize(src)?;
        let state_id = StateId::deserialize(src)?;
        let last_write_offset = NewOffset::deserialize(src)?;
        let time_modify = NewTime::deserialize(src)?;
        let layout_update = LayoutUpdate::deserialize(src)?;

        Ok(Self {
            offset,
            length,
            state_id,
            last_write_offset,
            time_modify,
            layout_update,
        })
    }
}

#[derive(Debug)]
pub enum NewSize {
    Changed(Length),
    Unchanged,
}

impl XdrSerialize for NewSize {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Changed(length) => {
                true.serialize(dest)?;
                length.serialize(dest)?;
            }
            Self::Unchanged => {
                false.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for NewSize {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let changed = bool::deserialize(src)?;

        if changed {
            let length = Length::deserialize(src)?;
            Ok(Self::Changed(length))
        } else {
            Ok(Self::Unchanged)
        }
    }
}

#[derive(Debug)]
pub struct LayoutCommitOk {
    new_size: NewSize,
}

impl AsNfsStatus for LayoutCommitOk {}

impl XdrSerialize for LayoutCommitOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.new_size.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LayoutCommitOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let new_size = NewSize::deserialize(src)?;

        Ok(Self { new_size })
    }
}

pub type LayoutCommitResult = Result<LayoutCommitOk, NfsStatus>;

#[derive(Debug)]
pub struct LayoutGetArgs {
    signal_layout_available: bool,
    layout_type: NfsLayoutType,
    io_mode: LayoutIoMode,
    offset: Offset,
    length: Length,
    min_length: Length,
    state_id: StateId,
    max_count: Count,
}

impl XdrSerialize for LayoutGetArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.signal_layout_available.serialize(dest)?;
        self.layout_type.serialize(dest)?;
        self.io_mode.serialize(dest)?;
        self.offset.serialize(dest)?;
        self.length.serialize(dest)?;
        self.min_length.serialize(dest)?;
        self.state_id.serialize(dest)?;
        self.max_count.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LayoutGetArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let signal_layout_available = bool::deserialize(src)?;
        let layout_type = NfsLayoutType::deserialize(src)?;
        let io_mode = LayoutIoMode::deserialize(src)?;
        let offset = Offset::deserialize(src)?;
        let length = Length::deserialize(src)?;
        let min_length = Length::deserialize(src)?;
        let state_id = StateId::deserialize(src)?;
        let max_count = Count::deserialize(src)?;

        Ok(Self {
            signal_layout_available,
            layout_type,
            io_mode,
            offset,
            length,
            min_length,
            state_id,
            max_count,
        })
    }
}

#[derive(Debug)]
pub struct LayoutGetOk {
    return_on_close: bool,
    state_id: StateId,
    layout: Vec<Layout>,
}

impl AsNfsStatus for LayoutGetOk {}

impl XdrSerialize for LayoutGetOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.return_on_close.serialize(dest)?;
        self.state_id.serialize(dest)?;
        xdr::serialize_vec(dest, &self.layout)?;

        Ok(())
    }
}

impl XdrDeserialize for LayoutGetOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let return_on_close = bool::deserialize(src)?;
        let state_id = StateId::deserialize(src)?;
        let layout = xdr::deserialize_vec::<_, Layout>(src)?;

        Ok(Self {
            return_on_close,
            state_id,
            layout,
        })
    }
}

#[derive(Debug)]
pub enum LayoutGetReturn {
    Ok(LayoutGetOk),
    TryLater(bool),
}

impl AsNfsStatus for LayoutGetReturn {
    fn as_status(&self) -> NfsStatus {
        match self {
            Self::Ok(_) => NfsStatus::Ok,
            Self::TryLater(_) => NfsStatus::LayoutTryLater,
        }
    }
}

impl XdrSerialize for LayoutGetReturn {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Ok(ok) => ok.serialize(dest),
            Self::TryLater(will_signal) => will_signal.serialize(dest),
        }
    }
}

pub type LayoutGetResult = Result<LayoutGetReturn, NfsStatus>;

#[derive(Debug)]
pub struct LayoutReturnArgs {
    reclaim: bool,
    layout_type: NfsLayoutType,
    io_mode: LayoutIoMode,
    layout_return: LayoutReturn,
}

impl XdrSerialize for LayoutReturnArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.reclaim.serialize(dest)?;
        self.layout_type.serialize(dest)?;
        self.io_mode.serialize(dest)?;
        self.layout_return.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for LayoutReturnArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let reclaim = bool::deserialize(src)?;
        let layout_type = NfsLayoutType::deserialize(src)?;
        let io_mode = LayoutIoMode::deserialize(src)?;
        let layout_return = LayoutReturn::deserialize(src)?;

        Ok(Self {
            reclaim,
            layout_type,
            io_mode,
            layout_return,
        })
    }
}

#[derive(Debug)]
pub enum LayoutReturnReturn {
    WithStateId(StateId),
    WithoutStateId,
}

impl AsNfsStatus for LayoutReturnReturn {}

impl XdrSerialize for LayoutReturnReturn {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::WithStateId(state_id) => {
                true.serialize(dest)?;
                state_id.serialize(dest)?;
            }
            Self::WithoutStateId => {
                false.serialize(dest)?;
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for LayoutReturnReturn {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let with_state_id = bool::deserialize(src)?;

        if with_state_id {
            let state_id = StateId::deserialize(src)?;
            Ok(Self::WithStateId(state_id))
        } else {
            Ok(Self::WithoutStateId)
        }
    }
}

pub type LayoutReturnResult = Result<LayoutReturnReturn, NfsStatus>;

#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
pub enum SecurityInfoNoNameStyle {
    CurrentFh = 0,
    Parent = 1,
}
xdr::serde_enum!(SecurityInfoNoNameStyle);

pub type SecurityInfoNoNameArgs = SecurityInfoNoNameStyle;

pub type SecurityInfoNoNameResult = SecurityInfoResult;

#[derive(Debug)]
pub struct SequenceArgs {
    pub session_id: SessionId,
    pub sequence_id: SequenceId,
    pub slot_id: SlotId,
    pub highest_slot_id: SlotId,
    pub cache_this: bool,
}

impl XdrSerialize for SequenceArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.session_id.serialize(dest)?;
        self.sequence_id.serialize(dest)?;
        self.slot_id.serialize(dest)?;
        self.highest_slot_id.serialize(dest)?;
        self.cache_this.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SequenceArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let session_id = SessionId::deserialize(src)?;
        let sequence_id = SequenceId::deserialize(src)?;
        let slot_id = SlotId::deserialize(src)?;
        let highest_slot_id = SlotId::deserialize(src)?;
        let cache_this = bool::deserialize(src)?;

        Ok(Self {
            session_id,
            sequence_id,
            slot_id,
            highest_slot_id,
            cache_this,
        })
    }
}

#[derive(Debug)]
pub struct SequenceOk {
    pub session_id: SessionId,
    pub sequence_id: SequenceId,
    pub slot_id: SlotId,
    pub highest_slot_id: SlotId,
    pub target_highest_slot_id: SlotId,
    pub status_flags: u32,
}

impl AsNfsStatus for SequenceOk {}

impl XdrSerialize for SequenceOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.session_id.serialize(dest)?;
        self.sequence_id.serialize(dest)?;
        self.slot_id.serialize(dest)?;
        self.highest_slot_id.serialize(dest)?;
        self.target_highest_slot_id.serialize(dest)?;
        self.status_flags.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SequenceOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let session_id = SessionId::deserialize(src)?;
        let sequence_id = SequenceId::deserialize(src)?;
        let slot_id = SlotId::deserialize(src)?;
        let highest_slot_id = SlotId::deserialize(src)?;
        let target_highest_slot_id = SlotId::deserialize(src)?;
        let status_flags = u32::deserialize(src)?;

        Ok(Self {
            session_id,
            sequence_id,
            slot_id,
            highest_slot_id,
            target_highest_slot_id,
            status_flags,
        })
    }
}

pub type SequenceResult = Result<SequenceOk, NfsStatus>;

pub struct SsaDigestInput {
    sequence_args: SequenceArgs,
}

impl XdrSerialize for SsaDigestInput {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.sequence_args.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SsaDigestInput {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let sequence_args = SequenceArgs::deserialize(src)?;

        Ok(Self { sequence_args })
    }
}

#[derive(Debug)]
pub struct SetSsvArgs {
    ssv: XdrOpaque,
    digest: XdrOpaque,
}

impl XdrSerialize for SetSsvArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.ssv.serialize(dest)?;
        self.digest.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SetSsvArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let ssv = XdrOpaque::deserialize(src)?;
        let digest = XdrOpaque::deserialize(src)?;

        Ok(Self { ssv, digest })
    }
}

pub struct SsrDigestInput {
    sequence_result: SequenceResult,
}

#[derive(Debug)]
pub struct SetSsvOk {
    digest: XdrOpaque,
}

impl AsNfsStatus for SetSsvOk {}

impl XdrSerialize for SetSsvOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.digest.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for SetSsvOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let digest = XdrOpaque::deserialize(src)?;

        Ok(Self { digest })
    }
}

pub type SetSsvResult = Result<SetSsvOk, NfsStatus>;

#[derive(Debug)]
pub struct TestStateIdsArgs {
    state_ids: Vec<StateId>,
}

impl XdrSerialize for TestStateIdsArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        xdr::serialize_vec(dest, &self.state_ids)?;

        Ok(())
    }
}

impl XdrDeserialize for TestStateIdsArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_ids = xdr::deserialize_vec::<_, StateId>(src)?;

        Ok(Self { state_ids })
    }
}

#[derive(Debug)]
pub struct TestStateIdsOk {
    state_ids: Vec<StateId>,
}

impl AsNfsStatus for TestStateIdsOk {}

impl XdrSerialize for TestStateIdsOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        xdr::serialize_vec(dest, &self.state_ids)?;

        Ok(())
    }
}

impl XdrDeserialize for TestStateIdsOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let state_ids = xdr::deserialize_vec::<_, StateId>(src)?;

        Ok(Self { state_ids })
    }
}

pub type TestStateIdsResult = Result<TestStateIdsOk, NfsStatus>;

#[derive(Debug)]
pub enum DelegationClaim {
    Previous(OpenDelegationType),
    CurrentFh,
    PreviousDelegateFh,
    Invalid(OpenClaimType),
}

impl XdrSerialize for DelegationClaim {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        match self {
            Self::Previous(delegation) => {
                OpenClaimType::Previous.serialize(dest)?;
                delegation.serialize(dest)?;
            }
            Self::CurrentFh => {
                OpenClaimType::CurrentFh.serialize(dest)?;
            }
            Self::PreviousDelegateFh => {
                OpenClaimType::PreviousDelegateFh.serialize(dest)?;
            }
            Self::Invalid(bad) => {
                return Err(std::io::Error::other(format!(
                    "Invalid open claim type for delegation claim: {bad:?}"
                )));
            }
        }

        Ok(())
    }
}

impl XdrDeserialize for DelegationClaim {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let claim_type = OpenClaimType::deserialize(src)?;
        let claim = match claim_type {
            OpenClaimType::Previous => {
                let delegation = OpenDelegationType::deserialize(src)?;
                Self::Previous(delegation)
            }
            OpenClaimType::CurrentFh => Self::CurrentFh,
            OpenClaimType::PreviousDelegateFh => Self::PreviousDelegateFh,
            OpenClaimType::Null
            | OpenClaimType::CurrentDelegate
            | OpenClaimType::PreviousDelegate
            | OpenClaimType::CurrentDelegateFh => Self::Invalid(claim_type),
        };

        Ok(claim)
    }
}

#[derive(Debug)]
pub struct WantDelegationArgs {
    want: u32,
    claim: DelegationClaim,
}

impl XdrSerialize for WantDelegationArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.want.serialize(dest)?;
        self.claim.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for WantDelegationArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let want = u32::deserialize(src)?;
        let claim = DelegationClaim::deserialize(src)?;

        Ok(Self { want, claim })
    }
}

#[derive(Debug)]
pub struct WantDelegationOk {
    open_delegation: OpenDelegation,
}

impl AsNfsStatus for WantDelegationOk {}

impl XdrSerialize for WantDelegationOk {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.open_delegation.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for WantDelegationOk {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let open_delegation = OpenDelegation::deserialize(src)?;

        Ok(Self { open_delegation })
    }
}

pub type WantDelegationResult = Result<WantDelegationOk, NfsStatus>;

#[derive(Debug)]
pub struct DestroyClientIdArgs {
    pub client_id: ClientId,
}

impl XdrSerialize for DestroyClientIdArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.client_id.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for DestroyClientIdArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let client_id = ClientId::deserialize(src)?;

        Ok(Self { client_id })
    }
}

pub type DestroyClientIdResult = Result<NfsStatus, NfsStatus>;

#[derive(Debug)]
pub struct ReclaimCompleteArgs {
    pub one_fs: bool,
}

impl XdrSerialize for ReclaimCompleteArgs {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        self.one_fs.serialize(dest)?;

        Ok(())
    }
}

impl XdrDeserialize for ReclaimCompleteArgs {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let one_fs = bool::deserialize(src)?;

        Ok(Self { one_fs })
    }
}

pub type ReclaimCompleteResult = Result<NfsStatus, NfsStatus>;

pub enum NfsOpResults {}
