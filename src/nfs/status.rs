use num_derive::{FromPrimitive, ToPrimitive};

use crate::nfs::AsNfsStatus;
use crate::xdr::{self, XdrDeserialize, XdrSerialize};

/// NFS Status Codes
#[derive(Clone, Copy, Debug, FromPrimitive, ToPrimitive)]
#[repr(u32)]
//#[expect(clippy::upper_case_acronyms, reason = "Errno variants")]
pub enum NfsStatus {
    Ok = 0,                                 // everything is okay
    EPERM = 1,                              // caller not privileged
    ENOENT = 2,                             // no such file/directory
    EIO = 5,                                // hard I/O error
    ENXIO = 6,                              // no such device
    EACCESS = 13,                           // access denied
    EEXIST = 17,                            // file already exists
    EXDEV = 18,                             // different filesystems
    ENOTDIR = 20,                           // should be a directory
    EISDIR = 21,                            // should not be directory
    EINVAL = 22,                            // invalid argument
    EFBIG = 27,                             // file exceeds server max
    ENOSPC = 28,                            // no space on filesystem
    EROFS = 30,                             // read-only filesystem
    EMLINK = 31,                            // too many hard links
    ENAMETOOLONG = 63,                      // name exceeds server max
    ENOTEMPTY = 66,                         // directory not empty
    EDQUOT = 69,                            // hard quota limit reached
    ESTALE = 70,                            // file no longer exists
    EBADHANDLE = 10001,                     // Illegal filehandle
    BadCookie = 10003,                      // READDIR cookie is stale
    NotSupported = 10004,                   // operation not supported
    TooSmall = 10005,                       // response limit exceeded
    ServerFault = 10006,                    // undefined server error
    BadType = 10007,                        // type invalid for CREATE
    Delay = 10008,                          // file "busy" - retry
    AttrsSame = 10009,                      // nverify says attrs same
    Denied = 10010,                         // lock unavailable
    Expired = 10011,                        // lock lease expired
    Locked = 10012,                         // I/O failed due to lock
    InGracePeriod = 10013,                  // in grace period
    FhExpired = 10014,                      // filehandle expired
    ShareDenied = 10015,                    // share reserve denied
    WrongSecurity = 10016,                  // wrong security flavor
    ClientIdInUse = 10017,                  // clientid in use
    Moved = 10019,                          // filesystem relocated
    NoFilehandle = 10020,                   // current FH is not set
    MinorVersionMismatch = 10021,           // minor vers not supp
    StaleClientId = 10022,                  // server has rebooted
    StaleStateId = 10023,                   // server has rebooted
    OldStateId = 10024,                     // state is out of sync
    BadStateId = 10025,                     // incorrect stateid
    BadSeqId = 10026,                       // request is out of seq.
    NotSame = 10027,                        // verify - attrs not same
    LockRange = 10028,                      // overlapping lock range
    Symlink = 10029,                        // should be file/directory
    RestoreFh = 10030,                      // no saved filehandle
    LeaseMoved = 10031,                     // some filesystem moved
    AttrNotSupported = 10032,               // recommended attr not sup
    NoGrace = 10033,                        // reclaim outside of grace
    ReclaimBad = 10034,                     // reclaim error at server
    ReclaimConflict = 10035,                // conflict on reclaim
    BadXdr = 10036,                         // XDR decode failed
    LocksHeld = 10037,                      // file locks held at CLOSE
    OpenMode = 10038,                       // conflict in OPEN and I/O
    BadOwner = 10039,                       // owner translation bad
    BadChar = 10040,                        // utf-8 char not supported
    BadName = 10041,                        // name not supported
    BadRange = 10042,                       // lock range not supported
    LockNotSupported = 10043,               // no atomic up/downgrade
    OpIllegal = 10044,                      // undefined operation
    DeadLock = 10045,                       // file locking deadlock
    FileOpen = 10046,                       // open file blocks op.
    AdminRevoked = 10047,                   // lockowner state revoked
    CallbackPathDown = 10048,               // callback path down
    BadIoMode = 10049,                      //
    BadLayout = 10050,                      //
    BadSessionDigest = 10051,               //
    BadSession = 10052,                     //
    BadSlot = 10053,                        //
    CompleteAlready = 10054,                //
    ConnNotBoundToSession = 10055,          //
    DelegationAlreadyWanted = 10056,        //
    BackChannelBusy = 10057,                // backchan reqs outstanding
    LayoutTryLater = 10058,                 //
    LayoutUnavailable = 10059,              //
    NoMatchingLayout = 10060,               //
    RecallConflict = 10061,                 //
    UnknownLayoutType = 10062,              //
    SeqMisordered = 10063,                  // unexpected seq.ID in req
    SequencePos = 10064,                    // [CB_]SEQ. op not 1st op
    RequestTooBig = 10065,                  // request too big
    ReplyTooBig = 10066,                    // reply too big
    ReplyNotAllCached = 10067,              // rep. not all cached
    RetryUncachedReply = 10068,             // retry & rep. uncached
    UnsafeCompound = 10069,                 // retry/recovery too hard
    TooManyOps = 10070,                     // too many ops in [CB_]COMP
    NotInSession = 10071,                   // op needs [CB_]SEQ. op
    HashAlgorithmUnsupported = 10072,       // hash alg. not supp.
    ClientIdBusy = 10074,                   // clientid has state
    ParallelNFSIoHole = 10075,              // IO to _SPARSE file hole
    SeqFalseRetry = 10076,                  // Retry != original req.
    BadHighSlot = 10077,                    // req has bad highest_slot
    DeadSession = 10078,                    // new req sent to dead sess
    EncryptionAlgorithmUnsupported = 10079, // encr alg. not supp.
    ParallelNFSNoLayout = 10080,            // I/O without a layout
    NotOnlyOp = 10081,                      // addl ops not allowed
    WrongCredential = 10082,                // op done by wrong cred
    WrongType = 10083,                      // op on wrong type object
    DirDelegationUnavailable = 10084,       // delegation not avail.
    RejectDelegation = 10085,               // cb rejected delegation
    ReturnConflict = 10086,                 // layout get before return
    DelegationRevoked = 10087,              // deleg./layout revoked
}

xdr::serde_enum!(NfsStatus);

impl AsNfsStatus for NfsStatus {
    fn as_status(&self) -> Self {
        *self
    }

    fn has_body(&self) -> bool {
        false
    }
}
