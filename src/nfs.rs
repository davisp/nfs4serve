use std::io::Read;

use anyhow::{Context as _, Result};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::FromPrimitive;

use crate::rpc;
use crate::xdr::{self, XdrSerde as _};

/// These are the RPC constants needed to call the NFS Version 3
///  service.  They are given in decimal.
pub const PROGRAM: u32 = 100_003;
pub const PROGRAM_ACL: u32 = 100_227;
pub const PROGRAM_ID_MAP: u32 = 100_270;
pub const PROGRAM_METADATA: u32 = 200_024;

pub const VERSION: u32 = 4;
pub const VERSION_MINOR: u32 = 1;

pub fn handle(
    xid: u32,
    call: rpc::RpcBodyCall,
    auth: Option<rpc::AuthUnix>,
    reader: &mut impl Read,
) -> Result<rpc::RpcMessage> {
    if call.version != VERSION {
        log::warn!(
            "Client attempted an unsupported version of NFS: {} != {VERSION}",
            call.version
        );
        return Ok(rpc::RpcMessage::program_mismatch_reply(xid, VERSION));
    }

    let prog =
        NFSProgram::from_u32(call.procedure).unwrap_or(NFSProgram::Invalid);

    log::info!("NFS program: {prog:?}");

    match prog {
        NFSProgram::Null => Ok(rpc::RpcMessage::successful_reply(xid)),
        NFSProgram::Compound => {
            let tag = Vec::<u8>::deserialize(reader)
                .context("Error decoding request tag.")?;
            let version = u32::deserialize(reader)
                .context("Error decoding NFS minor version.")?;
            if version != VERSION_MINOR {
                todo!();
            }

            todo!();
        }
        NFSProgram::Invalid => {
            Ok(rpc::RpcMessage::procedure_unavailable_reply(xid))
        }
    }
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum NFSProgram {
    Null = 0,
    Compound = 1,
    Invalid = 255,
}

enum NFSStatus {
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
    BadCooki = 10003,                       // READDIR cookie is stale
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
    NoFileHandle = 10020,                   // current FH is not set
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

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum NFSOperation {
    Access = 3,
    Close = 4,
    Commit = 5,
    Create = 6,
    PurgeDelegations = 7,
    ReturnDelegation = 8,
    GetAttribute = 9,
    GetFh = 10,
    Link = 11,
    LockCreate = 12,
    LockTest = 13,
    LockRelease = 14,
    Lookup = 15,
    LookupParent = 16,
    VerifyAttributeDiff = 17,
    Open = 18,
    OpenAttrs = 19,
    OpenDowngrade = 21,
    PutFh = 22,
    PutPublicFh = 23,
    PutRootFh = 24,
    Read = 25,
    ReadDir = 26,
    ReadLink = 27,
    Remove = 28,
    Rename = 29,
    RestoreFh = 31,
    SaveFh = 32,
    SecurityInfo = 33,
    SetAttr = 34,
    Verify = 37,
    Write = 38,

    BackchannelControl = 40,
    BindConnectionToSession = 41,
    ExchangeId = 42,
    CreateSession = 43,
    DestroySession = 44,
    FreeStateId = 45,
    GetDirDelegation = 46,
    GetDeviceInfo = 47,
    GetDeviceList = 48,
    LayoutCommit = 49,
    LayoutGet = 50,
    LayoutReturn = 51,
    SecurityInfoUnnamed = 52,
    Sequence = 53,
    SetSSV = 54,
    TestStateId = 55,
    WantDelegation = 56,
    DestroyClient = 57,
    ReclaimComplete = 58,
    Illegal = 10044,
}
