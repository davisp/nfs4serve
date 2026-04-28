use std::net::{SocketAddr, ToSocketAddrs as _};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result, anyhow};
use tokio::net::TcpListener;

use crate::nfs::types::*;
use crate::nfs::{NfsConnection, NfsOperation, NfsRequest, NfsStatus};

macro_rules! handle {
    ($self:expr, $req:expr, $op:expr, $args:ty, $call:ident) => {
        let args = match $req.read::<$args>() {
            Ok(args) => args,
            Err(err) => {
                log::error!("Error parsing arguments for {:?}: {err:?}", $op);
                $req.ack(NfsStatus::ServerFault)?;
                return Ok(());
            }
        };

        match $self.$call(args) {
            Ok(ok) => $req
                .reply(&ok)
                .context(format!("Error replying to op {:?}", $op))?,
            Err(err) => $req.ack(err).context(format!(
                "Error acking error for op {:?}: {err:?}",
                $op
            ))?,
        }
    };
}

macro_rules! handle_no_args {
    ($self:expr, $req:expr, $op:expr, $call:ident) => {
        match $self.$call() {
            Ok(ok) => $req
                .reply(&ok)
                .context(format!("Error replying to op {:?}", $op))?,
            Err(err) => $req.ack(err).context(format!(
                "Error acking error for op {:?}: {err:?}",
                $op
            ))?,
        }
    };
}

#[derive(Debug)]
pub struct NFSv41ServerInner {
    address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct NFSv41Server {
    inner: Arc<Mutex<NFSv41ServerInner>>,
}

impl NFSv41Server {
    pub async fn new(addr: &str) -> Result<Self> {
        let Some(address) = addr
            .to_socket_addrs()
            .context("Error parsing or resolving server listen address.")?
            .nth(0)
        else {
            return Err(anyhow!(
                "No addresses found for the provided server listen address."
            ));
        };

        let inner = NFSv41ServerInner { address };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    /// Main loop
    ///
    /// # Panics
    ///
    /// If the mutex is poisoned.
    pub async fn serve(&self) -> Result<()> {
        let address = {
            let guard = self.inner.lock().expect("Server lock was poisoned.");
            guard.address
        };

        let listener = TcpListener::bind(address)
            .await
            .context("Error binding server listener socket.")?;

        log::info!("Server started. Waiting for connections.");
        loop {
            let (socket, addr) = listener
                .accept()
                .await
                .context("Error accepting next client connection.")?;

            // The socket should already be non-blocking, but we set it here just
            // to be certain.
            socket
                .set_nodelay(true)
                .context("Error setting nodelay on client socket.")?;

            let server = self.clone();
            tokio::spawn(async move {
                let conn = NfsConnection::new(socket, addr);
                match server.handle(conn).await {
                    Ok(()) => (),
                    Err(err) => {
                        log::info!("Client exited with error: {err:#?}");
                    }
                }
            });
        }
    }

    async fn handle(&self, mut conn: NfsConnection) -> Result<()> {
        loop {
            let mut req = conn
                .read()
                .await
                .context("Error reading next NFS Request.")?;

            for _ in 0..req.num_ops() {
                let op = req.next_op();
                let op = match op {
                    Ok(op) => op,
                    Err(err) => {
                        log::error!(
                            "Failed to read next COMPOUND operation: {err:?}"
                        );
                        req.ack(NfsStatus::OpIllegal)?;
                        conn.send(req).await?;
                        break;
                    }
                };

                let resp = self.dispatch(&mut req, op);
            }
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "It's a dispatch function, Michael!"
    )]
    #[expect(
        clippy::cognitive_complexity,
        reason = "It's still a dispatch function, Michael!"
    )]
    fn dispatch(&self, req: &mut NfsRequest, op: NfsOperation) -> Result<()> {
        match op {
            NfsOperation::Access => {
                handle!(self, req, op, AccessArgs, access);
            }
            NfsOperation::Close => {
                handle!(self, req, op, CloseArgs, close);
            }
            NfsOperation::Commit => {
                handle!(self, req, op, CommitArgs, commit);
            }
            NfsOperation::Create => {
                handle!(self, req, op, CreateArgs, create);
            }
            NfsOperation::PurgeDelegations => {
                handle!(self, req, op, PurgeDelegationsArgs, purge_delegations);
            }
            NfsOperation::ReturnDelegation => {
                handle!(self, req, op, ReturnDelegationArgs, return_delegation);
            }
            NfsOperation::GetAttributes => {
                handle!(self, req, op, GetAttributesArgs, get_attributes);
            }
            NfsOperation::GetFh => {
                handle_no_args!(self, req, op, get_current_fh);
            }
            NfsOperation::Link => {
                handle!(self, req, op, LinkArgs, link);
            }
            NfsOperation::LockCreate => {
                handle!(self, req, op, LockArgs, lock);
            }
            NfsOperation::LockTest => {
                handle!(self, req, op, LockTestArgs, lock_test);
            }
            NfsOperation::LockRelease => {
                handle!(self, req, op, LockReleaseArgs, lock_release);
            }
            NfsOperation::Lookup => {
                handle!(self, req, op, LookupArgs, lookup);
            }
            NfsOperation::LookupParent => {
                handle_no_args!(self, req, op, lookup_parent);
            }
            NfsOperation::VerifyAttributeDiff => {
                handle!(
                    self,
                    req,
                    op,
                    VerifyAttributeDifferenceArgs,
                    verify_attribute_difference
                );
            }
            NfsOperation::Open => {
                handle!(self, req, op, OpenArgs, open);
            }
            NfsOperation::OpenAttrs => {
                handle!(self, req, op, OpenAttributesArgs, open_attributes);
            }
            NfsOperation::OpenDowngrade => {
                handle!(self, req, op, OpenDowngradeArgs, open_downgrade);
            }
            NfsOperation::PutFh => {
                handle!(self, req, op, PutFhArgs, put_fh);
            }
            NfsOperation::PutPublicFh => {
                handle_no_args!(self, req, op, put_public_fh);
            }
            NfsOperation::PutRootFh => {
                handle_no_args!(self, req, op, put_root_fh);
            }
            NfsOperation::Read => {
                handle!(self, req, op, ReadArgs, read);
            }
            NfsOperation::ReadDir => {
                handle!(self, req, op, ReadDirectoryArgs, read_directory);
            }
            NfsOperation::ReadLink => {
                handle_no_args!(self, req, op, read_link);
            }
            NfsOperation::Remove => {
                handle!(self, req, op, RemoveArgs, remove);
            }
            NfsOperation::Rename => {
                handle!(self, req, op, RenameArgs, rename);
            }
            NfsOperation::RestoreFh => {
                handle_no_args!(self, req, op, restore_fh);
            }
            NfsOperation::SaveFh => {
                handle_no_args!(self, req, op, save_fh);
            }
            NfsOperation::SecurityInfo => {
                handle!(self, req, op, SecurityInfoArgs, security_info);
            }
            NfsOperation::SetAttr => {
                // This is a one off weird API in NFS for some reason.
                let args = match req.read::<SetAttributesArgs>() {
                    Ok(args) => args,
                    Err(err) => {
                        log::error!(
                            "Error parsing arguments for {op:?}: {err:?}"
                        );
                        req.ack(NfsStatus::ServerFault)?;
                        return Ok(());
                    }
                };

                let resp = self.set_attributes(args);
                req.reply(&resp)
                    .context(format!("Error replying to op {op:?}"))?;
            }
            NfsOperation::Verify => {
                handle!(self, req, op, VerifyArgs, verify);
            }
            NfsOperation::Write => {
                handle!(self, req, op, WriteArgs, write);
            }
            NfsOperation::BackchannelControl => {
                handle!(
                    self,
                    req,
                    op,
                    BackchannelControlArgs,
                    backchannel_control
                );
            }
            NfsOperation::BindConnectionToSession => {
                handle!(
                    self,
                    req,
                    op,
                    BindConnectionToSessionArgs,
                    bind_connection_to_session
                );
            }
            NfsOperation::ExchangeId => {
                handle!(self, req, op, ExchangeIdArgs, exchange_id);
            }
            NfsOperation::CreateSession => {
                handle!(self, req, op, CreateSessionArgs, create_session);
            }
            NfsOperation::DestroySession => {
                handle!(self, req, op, DestroySessionArgs, destroy_session);
            }
            NfsOperation::FreeStateId => {
                handle!(self, req, op, FreeStateIdArgs, free_state_id);
            }
            NfsOperation::GetDirDelegation => {
                handle!(
                    self,
                    req,
                    op,
                    GetDirectoryDelegationArgs,
                    get_directory_delegation
                );
            }
            NfsOperation::GetDeviceInfo => {
                handle!(self, req, op, GetDeviceInfoArgs, get_device_info);
            }
            NfsOperation::GetDeviceList => {
                handle!(self, req, op, GetDeviceListArgs, get_device_list);
            }
            NfsOperation::LayoutCommit => {
                handle!(self, req, op, LayoutCommitArgs, layout_commit);
            }
            NfsOperation::LayoutGet => {
                handle!(self, req, op, LayoutGetArgs, layout_get);
            }
            NfsOperation::LayoutReturn => {
                handle!(self, req, op, LayoutReturnArgs, layout_return);
            }
            NfsOperation::SecurityInfoNoName => {
                handle!(
                    self,
                    req,
                    op,
                    SecurityInfoNoNameArgs,
                    security_info_no_name
                );
            }
            NfsOperation::Sequence => {
                handle!(self, req, op, SequenceArgs, sequence);
            }
            NfsOperation::SetSSV => {
                handle!(self, req, op, SetSsvArgs, set_ssv);
            }
            NfsOperation::TestStateIds => {
                handle!(self, req, op, TestStateIdsArgs, test_state_ids);
            }
            NfsOperation::WantDelegation => {
                handle!(self, req, op, WantDelegationArgs, want_delegation);
            }
            NfsOperation::DestroyClientId => {
                handle!(self, req, op, DestroyClientIdArgs, destroy_client_id);
            }
            NfsOperation::ReclaimComplete => {
                handle!(self, req, op, ReclaimCompleteArgs, reclaim_complete);
            }
            NfsOperation::Illegal => req
                .ack(NfsStatus::OpIllegal)
                .context(format!("Error replying to op {op:?}"))?,
        }

        Ok(())
    }

    // All of the NFS operations are implemented below (or forwarded to the
    // NFSHandler instance).

    /// Check Access Rights
    ///
    /// [RFC 8881 Section 18.1](https://www.rfc-editor.org/rfc/rfc8881#OP_ACCESS)
    fn access(&self, args: AccessArgs) -> AccessResult {
        todo!()
    }

    /// Backchannel Control
    ///
    /// [RFC 8881 Section 18.33](https://www.rfc-editor.org/rfc/rfc8881#OP_BACKCHANNEL_CTL)
    fn backchannel_control(
        &self,
        args: BackchannelControlArgs,
    ) -> BackchannelControlResult {
        todo!()
    }

    /// Bind Connection To Session
    ///
    /// [RFC 8881 Section 18.34](https://www.rfc-editor.org/rfc/rfc8881#OP_BIND_CONN_TO_SESSION)
    fn bind_connection_to_session(
        &self,
        args: BindConnectionToSessionArgs,
    ) -> BindConnectionToSessionResult {
        todo!()
    }

    /// Close File
    ///
    /// [RFC 8881 Section 18.2](https://www.rfc-editor.org/rfc/rfc8881#OP_CLOSE)
    fn close(&self, args: CloseArgs) -> CloseResult {
        todo!()
    }

    /// Commit Cached Data
    ///
    /// [RFC 8881 Section 18.3](https://www.rfc-editor.org/rfc/rfc8881#OP_COMMIT)
    fn commit(&self, args: CommitArgs) -> CommitResult {
        todo!()
    }

    /// Create Non-Regular File Object
    ///
    /// [RFC 8881 Section 18.4](https://www.rfc-editor.org/rfc/rfc8881#OP_CREATE)
    fn create(&self, args: CreateArgs) -> CreateResult {
        todo!()
    }

    /// Create New Session and Confirm Client ID
    ///
    /// [RFC 8881 Section 18.36](https://www.rfc-editor.org/rfc/rfc8881#OP_CREATE_SESSION)
    fn create_session(&self, args: CreateSessionArgs) -> CreateSessionResult {
        todo!()
    }

    /// Purge Delegations Awaiting Recovery
    ///
    /// [RFC 8881 Section 18.5](https://www.rfc-editor.org/rfc/rfc8881#OP_DELEGPURGE)
    fn purge_delegations(
        &self,
        args: PurgeDelegationsArgs,
    ) -> PurgeDelegationsResult {
        todo!()
    }

    /// Return Delegation
    ///
    /// [RFC 8881 Section 18.6](https://www.rfc-editor.org/rfc/rfc8881#OP_DELEGRETURN)
    fn return_delegation(
        &self,
        args: ReturnDelegationArgs,
    ) -> ReturnDelegationResult {
        todo!()
    }

    /// Destroy Client ID
    ///
    /// [RFC 8881 Section 18.50](https://www.rfc-editor.org/rfc/rfc8881#OP_DESTROY_CLIENTID)
    fn destroy_client_id(
        &self,
        args: DestroyClientIdArgs,
    ) -> DestroyClientIdResult {
        todo!()
    }

    /// Destroy Session
    ///
    /// [RFC 8881 Section 18.37](https://www.rfc-editor.org/rfc/rfc8881#OP_DESTROY_SESSION)
    fn destroy_session(
        &self,
        args: DestroySessionArgs,
    ) -> DestroySessionResult {
        todo!()
    }

    /// Instantiate a Client ID
    ///
    /// [RFC 8881 Section 18.35](https://www.rfc-editor.org/rfc/rfc8881#OP_EXCHANGE_ID)
    fn exchange_id(&self, args: ExchangeIdArgs) -> ExchangeIdResult {
        todo!()
    }

    /// Free State ID with No Locks
    ///
    /// [RFC 8881 Section 18.38](https://www.rfc-editor.org/rfc/rfc8881#OP_FREE_STATEID)
    fn free_state_id(&self, args: FreeStateIdArgs) -> FreeStateIdResult {
        todo!()
    }

    /// Get Attributes
    ///
    /// [RFC 8881 Section 18.7](https://www.rfc-editor.org/rfc/rfc8881#OP_GETATTR)
    fn get_attributes(&self, args: GetAttributesArgs) -> GetAttributesResult {
        todo!()
    }

    /// Get Device Info
    ///
    /// [RFC 8881 Section 18.40](https://www.rfc-editor.org/rfc/rfc8881#OP_GETDEVICEINFO)
    fn get_device_info(&self, args: GetDeviceInfoArgs) -> GetDeviceInfoResult {
        todo!()
    }

    /// Get Device List
    ///
    /// [RFC 8881 Section 18.41](https://www.rfc-editor.org/rfc/rfc8881#OP_GETDEVICELIST)
    fn get_device_list(&self, args: GetDeviceListArgs) -> GetDeviceListResult {
        todo!()
    }

    /// Get Current Filehandle
    ///
    /// [RFC 8881 Section 18.8](https://www.rfc-editor.org/rfc/rfc8881#OP_GETFH)
    fn get_current_fh(&self) -> GetFhResult {
        todo!()
    }

    /// Get Directory Delegation
    ///
    /// [RFC 8881 Section 18.39](https://www.rfc-editor.org/rfc/rfc8881#OP_GET_DIR_DELEGATION)
    fn get_directory_delegation(
        &self,
        args: GetDirectoryDelegationArgs,
    ) -> GetDirectoryDelegationResult {
        todo!()
    }

    /// Commit Writes Made Using a Layout
    ///
    /// [RFC 8881 Section 18.42](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTCOMMIT)
    fn layout_commit(&self, args: LayoutCommitArgs) -> LayoutCommitResult {
        todo!()
    }

    /// Get Layout Information
    ///
    /// [RFC 8881 Section 18.43](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTGET)
    fn layout_get(&self, args: LayoutGetArgs) -> LayoutGetResult {
        todo!()
    }

    /// Release Layout Information
    ///
    /// [RFC 8881 Section 18.44](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTRETURN)
    fn layout_return(&self, args: LayoutReturnArgs) -> LayoutReturnResult {
        todo!()
    }

    /// Create Link to File
    ///
    /// [RFC 8881 Section 18.9](https://www.rfc-editor.org/rfc/rfc8881#OP_LINK)
    fn link(&self, args: LinkArgs) -> LinkResult {
        todo!()
    }

    /// Create a Lock
    ///
    /// [RFC 8881 Section 18.10](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCK)
    fn lock(&self, args: LockArgs) -> LockResult {
        todo!()
    }

    /// Test for Lock
    ///
    /// [RFC 8881 Section 18.11](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCKT)
    fn lock_test(&self, args: LockTestArgs) -> LockTestResult {
        todo!()
    }

    /// Release a Lock
    ///
    /// [RFC 8881 Section 18.12](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCKU)
    fn lock_release(&self, args: LockReleaseArgs) -> LockReleaseResult {
        todo!()
    }

    /// Lookup Filename
    ///
    /// [RFC 8881 Section 18.13](https://www.rfc-editor.org/rfc/rfc8881#OP_LOOKUP)
    fn lookup(&self, args: LookupArgs) -> LookupResult {
        todo!()
    }

    /// Lookup Parent Directory
    ///
    /// [RFC 8881 Section 18.14](https://www.rfc-editor.org/rfc/rfc8881#OP_LOOKUPP)
    fn lookup_parent(&self) -> LookupParentResult {
        todo!()
    }

    /// Verify Difference in Attributes
    ///
    /// [RFC 8881 Section 18.15](https://www.rfc-editor.org/rfc/rfc8881#OP_NVERIFY)
    fn verify_attribute_difference(
        &self,
        args: VerifyAttributeDifferenceArgs,
    ) -> VerifyAttributeDifferenceResult {
        todo!()
    }

    /// Open a Regular File
    ///
    /// [RFC 8881 Section 18.16](https://www.rfc-editor.org/rfc/rfc8881#OP_OPEN)
    fn open(&self, args: OpenArgs) -> OpenResult {
        todo!()
    }

    /// Open Named Attribute Directory
    ///
    /// [RFC 8881 Section 18.17](https://www.rfc-editor.org/rfc/rfc8881#OP_OPENATTR)
    fn open_attributes(
        &self,
        args: OpenAttributesArgs,
    ) -> OpenAttributesResult {
        todo!()
    }

    /// Reduce Open File Access
    ///
    /// [RFC 8881 Section 18.18](https://www.rfc-editor.org/rfc/rfc8881#OP_OPEN_DOWNGRADE)
    fn open_downgrade(&self, args: OpenDowngradeArgs) -> OpenDowngradeResult {
        todo!()
    }

    /// Set Current Filehandle
    ///
    /// [RFC 8881 Section 18.19](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTFH)
    fn put_fh(&self, args: PutFhArgs) -> PutFhResult {
        todo!()
    }

    /// Set Public Filehandle
    ///
    /// [RFC 8881 Section 18.20](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTPUBFH)
    fn put_public_fh(&self) -> PutPublicFhResult {
        todo!()
    }

    /// Set Root Filehandle
    ///
    /// [RFC 8881 Section 18.21](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTROOTFH)
    fn put_root_fh(&self) -> PutRootFhResult {
        todo!()
    }

    /// Read from File
    ///
    /// [RFC 8881 Section 18.22](https://www.rfc-editor.org/rfc/rfc8881#OP_READ)
    fn read(&self, args: ReadArgs) -> ReadResult {
        todo!()
    }

    /// Read Directory
    ///
    /// [RFC 8881 Section 18.23](https://www.rfc-editor.org/rfc/rfc8881#OP_READDIR)
    fn read_directory(&self, args: ReadDirectoryArgs) -> ReadDirectoryResult {
        todo!()
    }

    /// Read Symbolic Link
    ///
    /// [RFC 8881 Section 18.24](https://www.rfc-editor.org/rfc/rfc8881#OP_READLINK)
    fn read_link(&self) -> ReadLinkResult {
        todo!()
    }

    /// Indicate Reclaims Finished
    ///
    /// [RFC 8881 Section 18.51](https://www.rfc-editor.org/rfc/rfc8881#OP_RECLAIM_COMPLETE)
    fn reclaim_complete(
        &self,
        args: ReclaimCompleteArgs,
    ) -> ReclaimCompleteResult {
        todo!()
    }

    /// Remove File System Object
    ///
    /// [RFC 8881 Section 18.25](https://www.rfc-editor.org/rfc/rfc8881#OP_REMOVE)
    fn remove(&self, args: RemoveArgs) -> RemoveResult {
        todo!()
    }

    /// Rename Directory Entry
    ///
    /// [RFC 8881 Section 18.26](https://www.rfc-editor.org/rfc/rfc8881#OP_RENAME)
    fn rename(&self, args: RenameArgs) -> RenameResult {
        todo!()
    }

    /// Restore Saved Filehandle
    ///
    /// [RFC 8881 Section 18.27](https://www.rfc-editor.org/rfc/rfc8881#OP_RESTOREFH)
    fn restore_fh(&self) -> RestoreFhResult {
        todo!()
    }

    /// Save Current Filehandle
    ///
    /// [RFC 8881 Section 18.28](https://www.rfc-editor.org/rfc/rfc8881#OP_SAVEFH)
    fn save_fh(&self) -> SaveFhResult {
        todo!()
    }

    /// Obtain Available Security
    ///
    /// [RFC 8881 Section 18.29](https://www.rfc-editor.org/rfc/rfc8881#OP_SECINFO)
    fn security_info(&self, args: SecurityInfoArgs) -> SecurityInfoResult {
        todo!()
    }

    /// Obtain Available Security on Unnamed Object
    ///
    /// [RFC 8881 Section 18.45](https://www.rfc-editor.org/rfc/rfc8881#OP_SECINFO_NO_NAME)
    ///
    /// See also [RFC 8881 Section 13.12](https://www.rfc-editor.org/rfc/rfc8881#file_security_considerations)
    fn security_info_no_name(
        &self,
        style: SecurityInfoNoNameStyle,
    ) -> SecurityInfoNoNameResult {
        todo!()
    }

    /// Supply Per-Procedure Sequencing and Control
    ///
    /// [RFC 8881 Section 18.46](https://www.rfc-editor.org/rfc/rfc8881#OP_SEQUENCE)
    fn sequence(&self, args: SequenceArgs) -> SequenceResult {
        todo!()
    }

    /// Set Attributes
    ///
    /// [RFC 8881 Section 18.30](https://www.rfc-editor.org/rfc/rfc8881#OP_SETATTR)
    fn set_attributes(&self, args: SetAttributesArgs) -> SetAttributesResult {
        todo!()
    }

    /// Update SSV for a Client ID
    ///
    /// [RFC 8881 Section 18.47](https://www.rfc-editor.org/rfc/rfc8881#OP_SET_SSV)
    fn set_ssv(&self, args: SetSsvArgs) -> SetSsvResult {
        todo!()
    }

    /// Test `StateId`s for Validity
    ///
    /// [RFC 8881 Section 18.48](https://www.rfc-editor.org/rfc/rfc8881#OP_TEST_STATEID)
    fn test_state_ids(&self, args: TestStateIdsArgs) -> TestStateIdsResult {
        todo!()
    }

    /// Verify Same Attributes
    ///
    /// [RFC 8881 Section 18.31](https://www.rfc-editor.org/rfc/rfc8881#OP_VERIFY)
    fn verify(&self, args: VerifyArgs) -> VerifyResult {
        todo!()
    }

    /// Request Delegation
    ///
    /// [RFC 8881 Section 18.49](https://www.rfc-editor.org/rfc/rfc8881#OP_WANT_DELEGATION)
    fn want_delegation(
        &self,
        args: WantDelegationArgs,
    ) -> WantDelegationResult {
        todo!()
    }

    /// Write to File
    ///
    /// [RFC 8881 Section 18.32](https://www.rfc-editor.org/rfc/rfc8881#OP_WRITE)
    fn write(&self, args: WriteArgs) -> WriteResult {
        todo!()
    }
}
