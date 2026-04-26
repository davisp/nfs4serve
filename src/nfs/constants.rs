// These are the RPC constants needed to call the NFS Version 3
// service.  They are given in decimal.
pub const PROGRAM: u32 = 100_003;
pub const PROGRAM_ACL: u32 = 100_227;
pub const PROGRAM_ID_MAP: u32 = 100_270;
pub const PROGRAM_METADATA: u32 = 200_024;
pub const VERSION: u32 = 4;
pub const VERSION_MINOR: u32 = 1;

// The rest of this file is all of the constants from RFC 5662 and friends.

pub const NFS_FHSIZE: usize = 128;
pub const NFS_VERIFIER_SIZE: usize = 8;
pub const NFS_OPAQUE_LIMIT: usize = 1024;
pub const NFS_SESSIONID_SIZE: usize = 16;

pub const NFS_INT64_MAX: u64 = 0x7f_ff_ff_ff_ff_ff_ff_ff;
pub const NFS_UINT64_MAX: u64 = 0xff_ff_ff_ff_ff_ff_ff_ff;
pub const NFS_INT32_MAX: u32 = 0x7f_ff_ff_ff;
pub const NFS_UINT32_MAX: u32 = 0xff_ff_ff_ff;

pub const NFS_MAXFILELEN: u64 = 0xff_ff_ff_ff_ff_ff_ff_ff;
pub const NFS_MAXFILEOFF: u64 = 0xff_ff_ff_ff_ff_ff_ff_fe;

pub const ACL_SUPPORT_ALLOW_ACL: u32 = 0x00_00_00_01;
pub const ACL_SUPPORT_DENY_ACL: u32 = 0x00_00_00_02;
pub const ACL_SUPPORT_AUDIT_ACL: u32 = 0x00_00_00_04;
pub const ACL_SUPPORT_ALARM_ACL: u32 = 0x00_00_00_08;

// Access Control Entry constants

// Mask that indicates which
// Access Control Entries are supported.
// Values for the fattr4_aclsupport attribute.

// acetype4 values, others can be added as needed.
pub const ACE_ACCESS_ALLOWED_ACE_TYPE: u32 = 0x00_00_00_00;
pub const ACE_ACCESS_DENIED_ACE_TYPE: u32 = 0x00_00_00_01;
pub const ACE_SYSTEM_AUDIT_ACE_TYPE: u32 = 0x00_00_00_02;
pub const ACE_SYSTEM_ALARM_ACE_TYPE: u32 = 0x00_00_00_03;

// ACE flag values
pub const ACE_FILE_INHERIT_ACE: u32 = 0x00_00_00_01;
pub const ACE_DIRECTORY_INHERIT_ACE: u32 = 0x00_00_00_02;
pub const ACE_NO_PROPAGATE_INHERIT_ACE: u32 = 0x00_00_00_04;
pub const ACE_INHERIT_ONLY_ACE: u32 = 0x00_00_00_08;
pub const ACE_SUCCESSFUL_ACCESS_ACE_FLAG: u32 = 0x00_00_00_10;
pub const ACE_FAILED_ACCESS_ACE_FLAG: u32 = 0x00_00_00_20;
pub const ACE_IDENTIFIER_GROUP: u32 = 0x00_00_00_40;
pub const ACE_INHERITED_ACE: u32 = 0x00_00_00_80;

// ACE mask values

pub const ACE_READ_DATA: u32 = 0x00_00_00_01;
pub const ACE_LIST_DIRECTORY: u32 = 0x00_00_00_01;
pub const ACE_WRITE_DATA: u32 = 0x00_00_00_02;
pub const ACE_ADD_FILE: u32 = 0x00_00_00_02;
pub const ACE_APPEND_DATA: u32 = 0x00_00_00_04;
pub const ACE_ADD_SUBDIRECTORY: u32 = 0x00_00_00_04;
pub const ACE_READ_NAMED_ATTRS: u32 = 0x00_00_00_08;
pub const ACE_WRITE_NAMED_ATTRS: u32 = 0x00_00_00_10;

pub const ACE_EXECUTE: u32 = 0x00_00_00_20;
pub const ACE_DELETE_CHILD: u32 = 0x00_00_00_40;
pub const ACE_READ_ATTRIBUTES: u32 = 0x00_00_00_80;
pub const ACE_WRITE_ATTRIBUTES: u32 = 0x00_00_01_00;
pub const ACE_WRITE_RETENTION: u32 = 0x00_00_02_00;
pub const ACE_WRITE_RETENTION_HOLD: u32 = 0x00_00_04_00;
pub const ACE_DELETE: u32 = 0x00_01_00_00;
pub const ACE_READ_ACL: u32 = 0x00_02_00_00;
pub const ACE_WRITE_ACL: u32 = 0x00_04_00_00;
pub const ACE_WRITE_OWNER: u32 = 0x00_08_00_00;
pub const ACE_SYNCHRONIZE: u32 = 0x00_10_00_00;

/// `ACE_GENERIC_READ` -- defined as combination of
///      `ACE_READ_ACL` |
///      `ACE_READ_DATA` |
///      `ACE_READ_ATTRIBUTES` |
///      `ACE_SYNCHRONIZE`
///
///
pub const ACE_GENERIC_READ: u32 = 0x00_12_00_81;

/// `ACE_GENERIC_WRITE` -- defined as combination of
///      `ACE_READ_ACL` |
///      `ACE_WRITE_DATA` |
///      `ACE_WRITE_ATTRIBUTES` |
///      `ACE_WRITE_ACL` |
///      `ACE_APPEND_DATA` |
///      `ACE_SYNCHRONIZE`
///
pub const ACE_GENERIC_WRITE: u32 = 0x00_16_01_06;

/// `ACE_GENERIC_EXECUTE` -- defined as combination of
///      `ACE_READ_ACL`
///      `ACE_READ_ATTRIBUTES`
///      `ACE_EXECUTE`
///      `ACE_SYNCHRONIZE`
///
pub const ACE_GENERIC_EXECUTE: u32 = 0x00_12_00_A0;

pub const ACL_AUTO_INHERIT: u32 = 0x00_00_00_01;
pub const ACL_PROTECTED: u32 = 0x00_00_00_02;
pub const ACL_DEFAULTED: u32 = 0x00_00_00_04;

pub const MODE_SUID: u32 = 0x800; /* set user id on execution */
pub const MODE_SGID: u32 = 0x400; /* set group id on execution */
pub const MODE_SVTX: u32 = 0x200; /* save text even after use */
pub const MODE_RUSR: u32 = 0x100; /* read permission: owner */
pub const MODE_WUSR: u32 = 0x080; /* write permission: owner */
pub const MODE_XUSR: u32 = 0x040; /* execute permission: owner */
pub const MODE_RGRP: u32 = 0x020; /* read permission: group */
pub const MODE_WGRP: u32 = 0x010; /* write permission: group */
pub const MODE_XGRP: u32 = 0x008; /* execute permission: group */
pub const MODE_ROTH: u32 = 0x004; /* read permission: other */
pub const MODE_WOTH: u32 = 0x002; /* write permission: other */
pub const MODE_XOTH: u32 = 0x001; /* execute permission: other */

pub const FH_PERSISTENT: u32 = 0x00_00_00_00;
pub const FH_NOEXPIRE_WITH_OPEN: u32 = 0x00_00_00_01;
pub const FH_VOLATILE_ANY: u32 = 0x00_00_00_02;
pub const FH_VOL_MIGRATION: u32 = 0x00_00_00_04;
pub const FH_VOL_RENAME: u32 = 0x00_00_00_08;

pub const NFS_DEVICEID_SIZE: usize = 16;

pub const LAYOUT_RET_REC_FILE: u32 = 1;
pub const LAYOUT_RET_REC_FSID: u32 = 2;
pub const LAYOUT_RET_REC_ALL: u32 = 3;

pub const TH_READ_SIZE: u32 = 0;
pub const TH_WRITE_SIZE: u32 = 1;
pub const TH_READ_IOSIZE: u32 = 2;
pub const TH_WRITE_IOSIZE: u32 = 3;

pub const RET_DURATION_INFINITE: u64 = 0xff_ff_ff_ff_ff_ff_ff_ff;

pub const FSCHARSET_CAP_CONTAINS_NON_UTF8: u32 = 0x1;
pub const FSCHARSET_CAP_ALLOWS_ONLY_UTF8: u32 = 0x2;

pub const FATTR_SUPPORTED_ATTRS: u32 = 0;
pub const FATTR_TYPE: u32 = 1;
pub const FATTR_FH_EXPIRE_TYPE: u32 = 2;
pub const FATTR_CHANGE: u32 = 3;
pub const FATTR_SIZE: u32 = 4;
pub const FATTR_LINK_SUPPORT: u32 = 5;
pub const FATTR_SYMLINK_SUPPORT: u32 = 6;
pub const FATTR_NAMED_ATTR: u32 = 7;
pub const FATTR_FSID: u32 = 8;
pub const FATTR_UNIQUE_HANDLES: u32 = 9;
pub const FATTR_LEASE_TIME: u32 = 10;
pub const FATTR_RDATTR_ERROR: u32 = 11;
pub const FATTR_FILEHANDLE: u32 = 19;
pub const FATTR_ACL: u32 = 12;
pub const FATTR_ACLSUPPORT: u32 = 13;
pub const FATTR_ARCHIVE: u32 = 14;
pub const FATTR_CANSETTIME: u32 = 15;
pub const FATTR_CASE_INSENSITIVE: u32 = 16;
pub const FATTR_CASE_PRESERVING: u32 = 17;
pub const FATTR_CHOWN_RESTRICTED: u32 = 18;
pub const FATTR_FILEID: u32 = 20;
pub const FATTR_FILES_AVAIL: u32 = 21;
pub const FATTR_FILES_FREE: u32 = 22;
pub const FATTR_FILES_TOTAL: u32 = 23;
pub const FATTR_FS_LOCATIONS: u32 = 24;
pub const FATTR_HIDDEN: u32 = 25;
pub const FATTR_HOMOGENEOUS: u32 = 26;
pub const FATTR_MAXFILESIZE: u32 = 27;
pub const FATTR_MAXLINK: u32 = 28;
pub const FATTR_MAXNAME: u32 = 29;
pub const FATTR_MAXREAD: u32 = 30;
pub const FATTR_MAXWRITE: u32 = 31;
pub const FATTR_MIMETYPE: u32 = 32;
pub const FATTR_MODE: u32 = 33;
pub const FATTR_NO_TRUNC: u32 = 34;
pub const FATTR_NUMLINKS: u32 = 35;
pub const FATTR_OWNER: u32 = 36;
pub const FATTR_OWNER_GROUP: u32 = 37;
pub const FATTR_QUOTA_AVAIL_HARD: u32 = 38;
pub const FATTR_QUOTA_AVAIL_SOFT: u32 = 39;
pub const FATTR_QUOTA_USED: u32 = 40;
pub const FATTR_RAWDEV: u32 = 41;
pub const FATTR_SPACE_AVAIL: u32 = 42;
pub const FATTR_SPACE_FREE: u32 = 43;
pub const FATTR_SPACE_TOTAL: u32 = 44;
pub const FATTR_SPACE_USED: u32 = 45;
pub const FATTR_SYSTEM: u32 = 46;
pub const FATTR_TIME_ACCESS: u32 = 47;
pub const FATTR_TIME_ACCESS_SET: u32 = 48;
pub const FATTR_TIME_BACKUP: u32 = 49;
pub const FATTR_TIME_CREATE: u32 = 50;
pub const FATTR_TIME_DELTA: u32 = 51;
pub const FATTR_TIME_METADATA: u32 = 52;
pub const FATTR_TIME_MODIFY: u32 = 53;
pub const FATTR_TIME_MODIFY_SET: u32 = 54;
pub const FATTR_MOUNTED_ON_FILEID: u32 = 55;
pub const FATTR_DIR_NOTIF_DELAY: u32 = 56;
pub const FATTR_DIRENT_NOTIF_DELAY: u32 = 57;
pub const FATTR_DACL: u32 = 58;
pub const FATTR_SACL: u32 = 59;
pub const FATTR_CHANGE_POLICY: u32 = 60;
pub const FATTR_FS_STATUS: u32 = 61;
pub const FATTR_FS_LAYOUT_TYPES: u32 = 62;
pub const FATTR_LAYOUT_HINT: u32 = 63;
pub const FATTR_LAYOUT_TYPES: u32 = 64;
pub const FATTR_LAYOUT_BLKSIZE: u32 = 65;
pub const FATTR_LAYOUT_ALIGNMENT: u32 = 66;
pub const FATTR_FS_LOCATIONS_INFO: u32 = 67;
pub const FATTR_MDSTHRESHOLD: u32 = 68;
pub const FATTR_RETENTION_GET: u32 = 69;
pub const FATTR_RETENTION_SET: u32 = 70;
pub const FATTR_RETENTEVT_GET: u32 = 71;
pub const FATTR_RETENTEVT_SET: u32 = 72;
pub const FATTR_RETENTION_HOLD: u32 = 73;
pub const FATTR_MODE_SET_MASKED: u32 = 74;
pub const FATTR_FS_CHARSET_CAP: u32 = 76;

// Byte indices of items within
// fls_info: flag fields, class numbers,
// bytes indicating ranks and orders.

pub const FSLI4BX_GFLAGS: usize = 0;
pub const FSLI4BX_TFLAGS: usize = 1;
pub const FSLI4BX_CLSIMUL: usize = 2;
pub const FSLI4BX_CLHANDLE: usize = 3;
pub const FSLI4BX_CLFILEID: usize = 4;
pub const FSLI4BX_CLWRITEVER: usize = 5;
pub const FSLI4BX_CLCHANGE: usize = 6;
pub const FSLI4BX_CLREADDIR: usize = 7;
pub const FSLI4BX_READRANK: usize = 8;
pub const FSLI4BX_WRITERANK: usize = 9;
pub const FSLI4BX_READORDER: usize = 10;
pub const FSLI4BX_WRITEORDER: usize = 11;

// Bits defined within the general flag byte.

pub const FSLI4GF_WRITABLE: u8 = 0x01;
pub const FSLI4GF_CUR_REQ: u8 = 0x02;
pub const FSLI4GF_ABSENT: u8 = 0x04;
pub const FSLI4GF_GOING: u8 = 0x08;
pub const FSLI4GF_SPLIT: u8 = 0x10;

// Bits defined within the transport flag byte.
pub const FSLI4TF_RDMA: u32 = 0x01;

// Flag bits in FsLocationsInfo::flags
pub const FSLI4IF_VAR_SUB: u32 = 0x00_00_00_01;

pub const NFL4_UFLG_MASK: u32 = 0x00_00_00_3F;
pub const NFL4_UFLG_DENSE: u32 = 0x00_00_00_01;
pub const NFL4_UFLG_COMMIT_THRU_MDS: u32 = 0x00_00_00_02;
pub const NFL4_UFLG_STRIPE_UNIT_SIZE_MASK: u32 = 0xFF_FF_FF_C0;

pub const ACCESS_READ: u32 = 0x00_00_00_01;
pub const ACCESS_LOOKUP: u32 = 0x00_00_00_02;
pub const ACCESS_MODIFY: u32 = 0x00_00_00_04;
pub const ACCESS_EXTEND: u32 = 0x00_00_00_08;
pub const ACCESS_DELETE: u32 = 0x00_00_00_10;
pub const ACCESS_EXECUTE: u32 = 0x00_00_00_20;

pub const OPEN_SHARE_ACCESS_READ: u32 = 0x00_00_00_01;
pub const OPEN_SHARE_ACCESS_WRITE: u32 = 0x00_00_00_02;
pub const OPEN_SHARE_ACCESS_BOTH: u32 = 0x00_00_00_03;

pub const OPEN_SHARE_DENY_NONE: u32 = 0x00_00_00_00;
pub const OPEN_SHARE_DENY_READ: u32 = 0x00_00_00_01;
pub const OPEN_SHARE_DENY_WRITE: u32 = 0x00_00_00_02;
pub const OPEN_SHARE_DENY_BOTH: u32 = 0x00_00_00_03;

pub const OPEN_SHARE_ACCESS_WANT_DELEG_MASK: u32 = 0x00_00_FF_00;
pub const OPEN_SHARE_ACCESS_WANT_NO_PREFERENCE: u32 = 0x00_00_00_00;
pub const OPEN_SHARE_ACCESS_WANT_READ_DELEG: u32 = 0x00_00_01_00;
pub const OPEN_SHARE_ACCESS_WANT_WRITE_DELEG: u32 = 0x00_00_02_00;
pub const OPEN_SHARE_ACCESS_WANT_ANY_DELEG: u32 = 0x00_00_03_00;
pub const OPEN_SHARE_ACCESS_WANT_NO_DELEG: u32 = 0x00_00_04_00;
pub const OPEN_SHARE_ACCESS_WANT_CANCEL: u32 = 0x00_00_05_00;
pub const OPEN_SHARE_ACCESS_WANT_SIGNAL_DELEG_WHEN_RESRC_AVAIL: u32 =
    0x00_00_01_00_00;
pub const OPEN_SHARE_ACCESS_WANT_PUSH_DELEG_WHEN_UNCONTENDED: u32 =
    0x00_02_00_00;

/// Client must confirm open
pub const OPEN_RESULT_CONFIRM: u32 = 0x00_00_00_02;

/// Type of file locking behavior at the server
pub const OPEN_RESULT_LOCKTYPE_POSIX: u32 = 0x00_00_00_04;

/// Server will preserve file if removed while open
pub const OPEN_RESULT_PRESERVE_UNLINKED: u32 = 0x00_00_00_08;

/// Server may use `CB_NOTIFY_LOCK` on locks
/// derived from this open
pub const OPEN_RESULT_MAY_NOTIFY_LOCK: u32 = 0x00_00_00_20;

/// From RFC 2203
pub const RPCSEC_GSS: u32 = 6;

pub const EXCHANGE_ID_FLAG_SUPP_MOVED_REFER: u32 = 0x00_00_00_01;
pub const EXCHANGE_ID_FLAG_SUPP_MOVED_MIGR: u32 = 0x00_00_00_02;
pub const EXCHANGE_ID_FLAG_BIND_PRINC_STATEID: u32 = 0x00_00_01_00;
pub const EXCHANGE_ID_FLAG_USE_NON_PNFS: u32 = 0x00_01_00_00;
pub const EXCHANGE_ID_FLAG_USE_PNFS_MDS: u32 = 0x00_02_00_00;
pub const EXCHANGE_ID_FLAG_USE_PNFS_DS: u32 = 0x00_04_00_00;
pub const EXCHANGE_ID_FLAG_MASK_PNFS: u32 = 0x00_07_00_00;
pub const EXCHANGE_ID_FLAG_UPD_CONFIRMED_REC_A: u32 = 0x40_00_00_00;
pub const EXCHANGE_ID_FLAG_CONFIRMED_R: u32 = 0x80_00_00_00;

pub const CREATE_SESSION_FLAG_PERSIST: u32 = 0x00_00_00_01;
pub const CREATE_SESSION_FLAG_CONN_BACK_CHAN: u32 = 0x00_00_00_02;
pub const CREATE_SESSION_FLAG_CONN_RDMA: u32 = 0x00_00_00_04;

pub const SEQ_STATUS_CB_PATH_DOWN: u32 = 0x00_00_00_01;
pub const SEQ_STATUS_CB_GSS_CONTEXTS_EXPIRING: u32 = 0x00_00_00_02;
pub const SEQ_STATUS_CB_GSS_CONTEXTS_EXPIRED: u32 = 0x00_00_00_04;
pub const SEQ_STATUS_EXPIRED_ALL_STATE_REVOKED: u32 = 0x00_00_00_08;
pub const SEQ_STATUS_EXPIRED_SOME_STATE_REVOKED: u32 = 0x00_00_00_10;
pub const SEQ_STATUS_ADMIN_STATE_REVOKED: u32 = 0x00_00_00_20;
pub const SEQ_STATUS_RECALLABLE_STATE_REVOKED: u32 = 0x00_00_00_40;
pub const SEQ_STATUS_LEASE_MOVED: u32 = 0x00_00_00_80;
pub const SEQ_STATUS_RESTART_RECLAIM_NEEDED: u32 = 0x00_00_01_00;
pub const SEQ_STATUS_CB_PATH_DOWN_SESSION: u32 = 0x00_00_02_00;
pub const SEQ_STATUS_BACKCHANNEL_FAULT: u32 = 0x00_00_04_00;
pub const SEQ_STATUS_DEVID_CHANGED: u32 = 0x00_00_08_00;
pub const SEQ_STATUS_DEVID_DELETED: u32 = 0x00_00_10_00;
