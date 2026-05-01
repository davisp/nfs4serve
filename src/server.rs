use std::collections::HashMap;
use std::net::{SocketAddr, ToSocketAddrs as _};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result, anyhow};
use rand::RngExt as _;
use tokio::net::TcpListener;

use crate::client::Client;
use crate::nfs::api::NfsHandler;
use crate::nfs::constants::{
    EXCHANGE_ID_FLAG_CONFIRMED_R, EXCHANGE_ID_FLAG_USE_NON_PNFS,
    NFS_OPAQUE_LIMIT,
};
use crate::nfs::types::{
    AccessArgs, AccessResult, BackchannelControlArgs, BackchannelControlResult,
    BindConnectionToSessionArgs, BindConnectionToSessionResult, ClientId,
    CloseArgs, CloseResult, CommitArgs, CommitResult, CreateArgs, CreateResult,
    CreateSessionArgs, CreateSessionOk, CreateSessionResult,
    DestroyClientIdArgs, DestroyClientIdResult, DestroySessionArgs,
    DestroySessionResult, ExchangeIdArgs, ExchangeIdOk, ExchangeIdResult,
    FreeStateIdArgs, FreeStateIdResult, GetAttributesArgs, GetAttributesOk,
    GetAttributesResult, GetDeviceInfoArgs, GetDeviceInfoResult,
    GetDeviceListArgs, GetDeviceListResult, GetDirectoryDelegationArgs,
    GetDirectoryDelegationResult, GetFhOk, GetFhResult, LayoutCommitArgs,
    LayoutCommitResult, LayoutGetArgs, LayoutGetResult, LayoutReturnArgs,
    LayoutReturnResult, LinkArgs, LinkResult, LockArgs, LockReleaseArgs,
    LockReleaseResult, LockResult, LockTestArgs, LockTestResult, LookupArgs,
    LookupParentResult, LookupResult, OpenArgs, OpenAttributesArgs,
    OpenAttributesResult, OpenDowngradeArgs, OpenDowngradeResult, OpenResult,
    PurgeDelegationsArgs, PurgeDelegationsResult, PutFhArgs, PutFhResult,
    PutPublicFhResult, PutRootFhResult, ReadArgs, ReadDirectoryArgs,
    ReadDirectoryResult, ReadLinkResult, ReadResult, ReclaimCompleteArgs,
    ReclaimCompleteResult, RemoveArgs, RemoveResult, RenameArgs, RenameResult,
    RestoreFhResult, ReturnDelegationArgs, ReturnDelegationResult,
    SaveFhResult, SecurityInfoArgs, SecurityInfoNoNameArgs,
    SecurityInfoNoNameResult, SecurityInfoNoNameStyle, SecurityInfoResult,
    SequenceArgs, SequenceOk, SequenceResult, ServerOwner, SessionId,
    SetAttributesArgs, SetAttributesResult, SetSsvArgs, SetSsvResult,
    StateProtectionArg, StateProtectionResult, TestStateIdsArgs,
    TestStateIdsResult, VerifyArgs, VerifyAttributeDifferenceArgs,
    VerifyAttributeDifferenceResult, VerifyResult, WantDelegationArgs,
    WantDelegationResult, WriteArgs, WriteResult,
};
use crate::nfs::{self, NfsConnection, NfsOperation, NfsRequest, NfsStatus};
use crate::session::Session;
use crate::xdr::MaxLenBytes;

macro_rules! handle {
    ($self:expr, $req:expr, $op:expr, $args:ty, $call:ident) => {
        log::trace!("Handling NFS COMPOUND operation: {:?}", $op);

        let args = match $req.read::<$args>() {
            Ok(args) => args,
            Err(err) => {
                log::error!("Error parsing arguments for {:?}: {err:?}", $op);
                $req.ack($op, NfsStatus::ServerFault)?;
                return Ok(());
            }
        };

        log::trace!("NFS COMPOUND operation args:\n{args:#?}");

        match $self.$call($req, args).await {
            Ok(ok) => {
                log::trace!("Op {:?} OK reply: {ok:#?}", $op);
                $req.reply($op, &ok)
                    .context(format!("Error replying to op {:?}", $op))?;
            }
            Err(err) => {
                log::trace!("Op {:?} ERR reply: {err:#?}", $op);
                $req.ack($op, err).context(format!(
                    "Error acking error for op {:?}: {err:?}",
                    $op
                ))?
            }
        }
    };
}

macro_rules! handle_no_args {
    ($self:expr, $req:expr, $op:expr, $call:ident) => {
        log::trace!("Handling NFS COMPOUND operation: {:?} (No args)", $op);

        match $self.$call($req).await {
            Ok(ok) => {
                log::trace!("Op {:?} OK reply: {ok:#?}", $op);
                $req.reply($op, &ok)
                    .context(format!("Error replying to op {:?}", $op))?
            }
            Err(err) => {
                log::trace!("Op {:?} ERR reply: {err:#?}", $op);
                $req.ack($op, err).context(format!(
                    "Error acking error for op {:?}: {err:?}",
                    $op
                ))?
            }
        }
    };
}

#[derive(Debug)]
pub struct NFSv41ServerInner {
    server_owner: ServerOwner,
    server_scope: MaxLenBytes<NFS_OPAQUE_LIMIT>,
    address: SocketAddr,
    clients: HashMap<ClientId, Client>,
    client_ids_by_owner: HashMap<MaxLenBytes<NFS_OPAQUE_LIMIT>, ClientId>,
    next_client_id: u64,
    sessions: HashMap<SessionId, Session>,
    next_session_id: u128,
}

impl NFSv41ServerInner {
    /// Add a client to the server state.
    fn add_client(&mut self, client: Client) {
        assert!(
            !self.clients.contains_key(&client.client_id),
            "Duplicate client id's detected."
        );

        self.client_ids_by_owner
            .insert(client.owner_id.clone(), client.client_id);
        self.clients.insert(client.client_id, client);
    }

    /// Remove a client from server state.
    ///
    /// Currently this is not async mostly because I don't want to fight the
    /// server mutex in async Rust. I'll probably have to figure that out
    /// eventually.
    fn remove_client(&mut self, client_id: ClientId) {
        let Some(client) = self.clients.remove(&client_id) else {
            log::error!(
                "Attempted to remove an unknown client_id: {client_id}"
            );
            return;
        };

        self.client_ids_by_owner.remove(&client.owner_id);
    }
}

#[derive(Clone)]
pub struct NFSv41Server {
    inner: Arc<Mutex<NFSv41ServerInner>>,
    handler: Arc<dyn NfsHandler>,
}

impl NFSv41Server {
    /// Create a new server.
    pub fn new(handler: Arc<dyn NfsHandler>, addr: &str) -> Result<Self> {
        #[expect(clippy::missing_panics_doc, reason = "It won't panic.")]
        let server_owner = ServerOwner {
            minor_id: rand::rng().random::<u64>(),
            major_id: MaxLenBytes::<NFS_OPAQUE_LIMIT>::new(Vec::from(
                "some-sort-of-unique-strng",
            ))
            .unwrap(),
        };

        #[expect(clippy::missing_panics_doc, reason = "It won't panic.")]
        let server_scope =
            MaxLenBytes::<NFS_OPAQUE_LIMIT>::new(Vec::from("localhost"))
                .unwrap();

        let Some(address) = addr
            .to_socket_addrs()
            .context("Error parsing or resolving server listen address.")?
            .nth(0)
        else {
            return Err(anyhow!(
                "No addresses found for the provided server listen address."
            ));
        };

        let inner = NFSv41ServerInner {
            server_owner,
            server_scope,
            address,
            clients: HashMap::new(),
            client_ids_by_owner: HashMap::new(),
            next_client_id: 1,
            sessions: HashMap::new(),
            next_session_id: 1,
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
            handler,
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
                        req.ack(NfsOperation::Illegal, NfsStatus::OpIllegal)?;
                        conn.send(req).await?;
                        return Err(anyhow!(
                            "Client attempted to use an illegal operation."
                        ));
                    }
                };

                log::debug!("Dispatching COMPOUND op: {op:?}");
                self.dispatch(&mut req, op).await?;
            }

            conn.send(req).await?;
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
    async fn dispatch(
        &self,
        req: &mut NfsRequest,
        op: NfsOperation,
    ) -> Result<()> {
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
                        req.ack(op, NfsStatus::ServerFault)?;
                        return Ok(());
                    }
                };

                let resp = self.set_attributes(req, args).await;
                req.reply(op, &resp)
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
                .ack(op, NfsStatus::OpIllegal)
                .context(format!("Error replying to op {op:?}"))?,
        }

        Ok(())
    }

    // All of the NFS operations are implemented below (or forwarded to the
    // NFSHandler instance).

    /// Check Access Rights
    ///
    /// [RFC 8881 Section 18.1](https://www.rfc-editor.org/rfc/rfc8881#OP_ACCESS)
    async fn access(
        &self,
        _req: &mut NfsRequest,
        _args: AccessArgs,
    ) -> AccessResult {
        todo!()
    }

    /// Backchannel Control
    ///
    /// [RFC 8881 Section 18.33](https://www.rfc-editor.org/rfc/rfc8881#OP_BACKCHANNEL_CTL)
    async fn backchannel_control(
        &self,
        _req: &mut NfsRequest,
        _args: BackchannelControlArgs,
    ) -> BackchannelControlResult {
        todo!()
    }

    /// Bind Connection To Session
    ///
    /// [RFC 8881 Section 18.34](https://www.rfc-editor.org/rfc/rfc8881#OP_BIND_CONN_TO_SESSION)
    async fn bind_connection_to_session(
        &self,
        _req: &mut NfsRequest,
        _args: BindConnectionToSessionArgs,
    ) -> BindConnectionToSessionResult {
        todo!()
    }

    /// Close File
    ///
    /// [RFC 8881 Section 18.2](https://www.rfc-editor.org/rfc/rfc8881#OP_CLOSE)
    async fn close(
        &self,
        _req: &mut NfsRequest,
        _args: CloseArgs,
    ) -> CloseResult {
        todo!()
    }

    /// Commit Cached Data
    ///
    /// [RFC 8881 Section 18.3](https://www.rfc-editor.org/rfc/rfc8881#OP_COMMIT)
    async fn commit(
        &self,
        _req: &mut NfsRequest,
        _args: CommitArgs,
    ) -> CommitResult {
        todo!()
    }

    /// Create Non-Regular File Object
    ///
    /// [RFC 8881 Section 18.4](https://www.rfc-editor.org/rfc/rfc8881#OP_CREATE)
    async fn create(
        &self,
        _req: &mut NfsRequest,
        _args: CreateArgs,
    ) -> CreateResult {
        todo!()
    }

    /// Create New Session and Confirm Client ID
    ///
    /// [RFC 8881 Section 18.36](https://www.rfc-editor.org/rfc/rfc8881#OP_CREATE_SESSION)
    async fn create_session(
        &self,
        _req: &mut NfsRequest,
        args: CreateSessionArgs,
    ) -> CreateSessionResult {
        log::trace!("Create session for client_id: {}", args.client_id);

        let mut server = self.inner.lock().expect("Server mutex was poisoned.");

        let keys = server.clients.keys().copied().collect::<Vec<_>>();
        log::trace!("Existing client ids: {keys:#?}");

        // Technically we could burn this if the client doesn't exist. But it
        // simplifies the ownership issues. Now is not the time to take a hard
        // left into trying to figure out why we can't partial borrow in a
        // single function scope. I'm sure there are reasons.
        let session_id = server.next_session_id;
        server.next_session_id += 1;

        let session_id = session_id.to_be_bytes();

        if let Some(client) = server.clients.get_mut(&args.client_id) {
            if !client.confirmed {
                client.confirmed = true;
            }

            let session = Session::new(args.clone());
            server.sessions.insert(session_id, session);

            drop(server);

            let resp = CreateSessionOk {
                session_id,
                sequence: args.sequence,
                flags: 0,
                fore_channel_attrs: args.fore_channel_attrs,
                back_channel_attrs: args.back_channel_attrs,
            };

            log::trace!("Replying with: {resp:#?}");
            Ok(resp)
        } else {
            Err(NfsStatus::StaleClientId)
        }
    }

    /// Purge Delegations Awaiting Recovery
    ///
    /// [RFC 8881 Section 18.5](https://www.rfc-editor.org/rfc/rfc8881#OP_DELEGPURGE)
    async fn purge_delegations(
        &self,
        _req: &mut NfsRequest,
        _args: PurgeDelegationsArgs,
    ) -> PurgeDelegationsResult {
        todo!()
    }

    /// Return Delegation
    ///
    /// [RFC 8881 Section 18.6](https://www.rfc-editor.org/rfc/rfc8881#OP_DELEGRETURN)
    async fn return_delegation(
        &self,
        _req: &mut NfsRequest,
        _args: ReturnDelegationArgs,
    ) -> ReturnDelegationResult {
        todo!()
    }

    /// Destroy Client ID
    ///
    /// [RFC 8881 Section 18.50](https://www.rfc-editor.org/rfc/rfc8881#OP_DESTROY_CLIENTID)
    async fn destroy_client_id(
        &self,
        _req: &mut NfsRequest,
        _args: DestroyClientIdArgs,
    ) -> DestroyClientIdResult {
        todo!()
    }

    /// Destroy Session
    ///
    /// [RFC 8881 Section 18.37](https://www.rfc-editor.org/rfc/rfc8881#OP_DESTROY_SESSION)
    async fn destroy_session(
        &self,
        req: &mut NfsRequest,
        args: DestroySessionArgs,
    ) -> DestroySessionResult {
        let mut server = self.inner.lock().expect("Server mutex was poisoned.");
        let Some(_session) = server.sessions.remove(&args.session_id) else {
            return Err(NfsStatus::BadSession);
        };

        // Eventually, cleanup anything related to the session here.

        drop(server);

        req.clear_session();

        Ok(NfsStatus::Ok)
    }

    /// Instantiate a Client ID
    ///
    /// [RFC 8881 Section 18.35](https://www.rfc-editor.org/rfc/rfc8881#OP_EXCHANGE_ID)
    ///
    /// So, this is a bit of a doozy for a first handler to implement. There's
    /// a whole lot of complexity due to pNFS and friends. For the moment I'm
    /// only attempting to handle initial connections and reconnections.
    async fn exchange_id(
        &self,
        _req: &mut NfsRequest,
        args: ExchangeIdArgs,
    ) -> ExchangeIdResult {
        if !matches!(args.state_protect, StateProtectionArg::None) {
            return Err(NfsStatus::EINVAL);
        }

        let mut server = self.inner.lock().expect("Server mutex was poisoned.");
        let maybe_client_id = server
            .client_ids_by_owner
            .get(&args.client_owner.owner_id)
            .copied();
        if let Some(client_id) = maybe_client_id {
            let Some(client) = server.clients.get(&client_id) else {
                panic!("Server client lists mismatch.")
            };

            if args.client_owner.verifier == client.verifier {
                let mut flags = EXCHANGE_ID_FLAG_USE_NON_PNFS;

                // I think something like this? I should probably also be
                // handling the update request? Oh well, we can always patch
                // this logic later.
                if client.confirmed {
                    flags |= EXCHANGE_ID_FLAG_CONFIRMED_R;
                }

                return Ok(ExchangeIdOk {
                    client_id,
                    sequence_id: client.sequence,
                    flags,
                    state_protection: StateProtectionResult::None,
                    server_owner: server.server_owner.clone(),
                    server_scope: server.server_scope.clone(),
                    server_impl_id: None,
                });
            }

            // Client changed verifiers. So delete all recorded state of the
            // client and fall through to reestablish a new ClientId.
            server.remove_client(client_id);
        }

        // Either a new client, or a client with a new verifier reconnecting.
        let client_id = server.next_client_id;
        server.next_client_id += 1;

        let client = Client::new(args.client_owner, client_id);

        server.clients.insert(client_id, client.clone());

        let flags = EXCHANGE_ID_FLAG_USE_NON_PNFS;

        Ok(ExchangeIdOk {
            client_id,
            sequence_id: client.sequence,
            flags,
            state_protection: StateProtectionResult::None,
            server_owner: server.server_owner.clone(),
            server_scope: server.server_scope.clone(),
            server_impl_id: None,
        })
    }

    /// Free State ID with No Locks
    ///
    /// [RFC 8881 Section 18.38](https://www.rfc-editor.org/rfc/rfc8881#OP_FREE_STATEID)
    async fn free_state_id(
        &self,
        _req: &mut NfsRequest,
        _args: FreeStateIdArgs,
    ) -> FreeStateIdResult {
        todo!()
    }

    /// Get Attributes
    ///
    /// [RFC 8881 Section 18.7](https://www.rfc-editor.org/rfc/rfc8881#OP_GETATTR)
    async fn get_attributes(
        &self,
        req: &NfsRequest,
        args: GetAttributesArgs,
    ) -> GetAttributesResult {
        if req.session_id.is_none() {
            return Err(NfsStatus::NotInSession);
        }

        let Some(fh) = req.current_fh.as_ref() else {
            return Err(NfsStatus::NoFilehandle);
        };

        let attrs = nfs::attrs::protocol_to_api(&args.attr_request);
        let resp = match self.handler.get_attributes(fh, &attrs).await {
            Ok(vals) => vals,
            Err(err) => {
                log::error!(
                    "Handler error in NfsHandler::get_attributes: {err:?}"
                );
                return Err(NfsStatus::ServerFault);
            }
        };

        if resp.is_empty() {
            // No idea which error is best for this case.
            return Err(NfsStatus::ESTALE);
        }

        let attributes = match nfs::attrs::values_to_protocol(resp) {
            Ok(attrs) => attrs,
            Err(err) => {
                log::error!(
                    "nfs4server error converting attribute values to XDR {err:?}",
                );
                return Err(NfsStatus::ServerFault);
            }
        };

        Ok(GetAttributesOk { attributes })
    }

    /// Get Device Info
    ///
    /// [RFC 8881 Section 18.40](https://www.rfc-editor.org/rfc/rfc8881#OP_GETDEVICEINFO)
    async fn get_device_info(
        &self,
        _req: &mut NfsRequest,
        _args: GetDeviceInfoArgs,
    ) -> GetDeviceInfoResult {
        todo!()
    }

    /// Get Device List
    ///
    /// [RFC 8881 Section 18.41](https://www.rfc-editor.org/rfc/rfc8881#OP_GETDEVICELIST)
    async fn get_device_list(
        &self,
        _req: &mut NfsRequest,
        _args: GetDeviceListArgs,
    ) -> GetDeviceListResult {
        todo!()
    }

    /// Get Current Filehandle
    ///
    /// [RFC 8881 Section 18.8](https://www.rfc-editor.org/rfc/rfc8881#OP_GETFH)
    async fn get_current_fh(&self, req: &NfsRequest) -> GetFhResult {
        if req.session_id.is_none() {
            return Err(NfsStatus::NotInSession);
        }

        let Some(fh) = req.current_fh.as_ref() else {
            return Err(NfsStatus::NoFilehandle);
        };

        Ok(GetFhOk { object: fh.clone() })
    }

    /// Get Directory Delegation
    ///
    /// [RFC 8881 Section 18.39](https://www.rfc-editor.org/rfc/rfc8881#OP_GET_DIR_DELEGATION)
    async fn get_directory_delegation(
        &self,
        _req: &mut NfsRequest,
        _args: GetDirectoryDelegationArgs,
    ) -> GetDirectoryDelegationResult {
        todo!()
    }

    /// Commit Writes Made Using a Layout
    ///
    /// [RFC 8881 Section 18.42](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTCOMMIT)
    async fn layout_commit(
        &self,
        _req: &mut NfsRequest,
        _args: LayoutCommitArgs,
    ) -> LayoutCommitResult {
        todo!()
    }

    /// Get Layout Information
    ///
    /// [RFC 8881 Section 18.43](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTGET)
    async fn layout_get(
        &self,
        _req: &mut NfsRequest,
        _args: LayoutGetArgs,
    ) -> LayoutGetResult {
        todo!()
    }

    /// Release Layout Information
    ///
    /// [RFC 8881 Section 18.44](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTRETURN)
    async fn layout_return(
        &self,
        _req: &mut NfsRequest,
        _args: LayoutReturnArgs,
    ) -> LayoutReturnResult {
        todo!()
    }

    /// Create Link to File
    ///
    /// [RFC 8881 Section 18.9](https://www.rfc-editor.org/rfc/rfc8881#OP_LINK)
    async fn link(&self, _req: &mut NfsRequest, _args: LinkArgs) -> LinkResult {
        todo!()
    }

    /// Create a Lock
    ///
    /// [RFC 8881 Section 18.10](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCK)
    async fn lock(&self, _req: &mut NfsRequest, _args: LockArgs) -> LockResult {
        todo!()
    }

    /// Test for Lock
    ///
    /// [RFC 8881 Section 18.11](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCKT)
    async fn lock_test(
        &self,
        _req: &mut NfsRequest,
        _args: LockTestArgs,
    ) -> LockTestResult {
        todo!()
    }

    /// Release a Lock
    ///
    /// [RFC 8881 Section 18.12](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCKU)
    async fn lock_release(
        &self,
        _req: &mut NfsRequest,
        _args: LockReleaseArgs,
    ) -> LockReleaseResult {
        todo!()
    }

    /// Lookup Filename
    ///
    /// [RFC 8881 Section 18.13](https://www.rfc-editor.org/rfc/rfc8881#OP_LOOKUP)
    async fn lookup(
        &self,
        _req: &mut NfsRequest,
        _args: LookupArgs,
    ) -> LookupResult {
        todo!()
    }

    /// Lookup Parent Directory
    ///
    /// [RFC 8881 Section 18.14](https://www.rfc-editor.org/rfc/rfc8881#OP_LOOKUPP)
    async fn lookup_parent(&self, _req: &mut NfsRequest) -> LookupParentResult {
        todo!()
    }

    /// Verify Difference in Attributes
    ///
    /// [RFC 8881 Section 18.15](https://www.rfc-editor.org/rfc/rfc8881#OP_NVERIFY)
    async fn verify_attribute_difference(
        &self,
        _req: &mut NfsRequest,
        _args: VerifyAttributeDifferenceArgs,
    ) -> VerifyAttributeDifferenceResult {
        todo!()
    }

    /// Open a Regular File
    ///
    /// [RFC 8881 Section 18.16](https://www.rfc-editor.org/rfc/rfc8881#OP_OPEN)
    async fn open(&self, _req: &mut NfsRequest, _args: OpenArgs) -> OpenResult {
        todo!()
    }

    /// Open Named Attribute Directory
    ///
    /// [RFC 8881 Section 18.17](https://www.rfc-editor.org/rfc/rfc8881#OP_OPENATTR)
    async fn open_attributes(
        &self,
        _req: &mut NfsRequest,
        _args: OpenAttributesArgs,
    ) -> OpenAttributesResult {
        todo!()
    }

    /// Reduce Open File Access
    ///
    /// [RFC 8881 Section 18.18](https://www.rfc-editor.org/rfc/rfc8881#OP_OPEN_DOWNGRADE)
    async fn open_downgrade(
        &self,
        _req: &mut NfsRequest,
        _args: OpenDowngradeArgs,
    ) -> OpenDowngradeResult {
        todo!()
    }

    /// Set Current Filehandle
    ///
    /// [RFC 8881 Section 18.19](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTFH)
    async fn put_fh(
        &self,
        req: &mut NfsRequest,
        args: PutFhArgs,
    ) -> PutFhResult {
        if req.session_id.is_none() {
            return Err(NfsStatus::NotInSession);
        }

        req.current_fh = Some(args.object);

        Ok(NfsStatus::Ok)
    }

    /// Set Public Filehandle
    ///
    /// [RFC 8881 Section 18.20](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTPUBFH)
    async fn put_public_fh(&self, _req: &mut NfsRequest) -> PutPublicFhResult {
        todo!()
    }

    /// Set Root Filehandle
    ///
    /// [RFC 8881 Section 18.21](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTROOTFH)
    async fn put_root_fh(&self, req: &mut NfsRequest) -> PutRootFhResult {
        let fh = self.handler.root_fh();

        req.current_fh = Some(fh);
        req.current_state_id = None;

        Ok(NfsStatus::Ok)
    }

    /// Read from File
    ///
    /// [RFC 8881 Section 18.22](https://www.rfc-editor.org/rfc/rfc8881#OP_READ)
    async fn read(&self, _req: &mut NfsRequest, _args: ReadArgs) -> ReadResult {
        todo!()
    }

    /// Read Directory
    ///
    /// [RFC 8881 Section 18.23](https://www.rfc-editor.org/rfc/rfc8881#OP_READDIR)
    async fn read_directory(
        &self,
        _req: &mut NfsRequest,
        _args: ReadDirectoryArgs,
    ) -> ReadDirectoryResult {
        todo!()
    }

    /// Read Symbolic Link
    ///
    /// [RFC 8881 Section 18.24](https://www.rfc-editor.org/rfc/rfc8881#OP_READLINK)
    async fn read_link(&self, _req: &mut NfsRequest) -> ReadLinkResult {
        todo!()
    }

    /// Indicate Reclaims Finished
    ///
    /// [RFC 8881 Section 18.51](https://www.rfc-editor.org/rfc/rfc8881#OP_RECLAIM_COMPLETE)
    async fn reclaim_complete(
        &self,
        req: &NfsRequest,
        args: ReclaimCompleteArgs,
    ) -> ReclaimCompleteResult {
        if args.one_fs {
            return Err(NfsStatus::EINVAL);
        }

        if req.session_id.is_none() {
            return Err(NfsStatus::NotInSession);
        }

        let Some(client_id) = req.client_id else {
            log::error!("Request has a session_id but no client_id");
            return Err(NfsStatus::ServerFault);
        };

        let mut server = self.inner.lock().expect("Server mutex was poisoned.");
        let Some(client) = server.clients.get_mut(&client_id) else {
            log::error!("Found an active session but client doesn't exist.");
            return Err(NfsStatus::ServerFault);
        };

        if client.reclaim_complete {
            return Err(NfsStatus::CompleteAlready);
        }

        client.reclaim_complete = true;

        drop(server);

        Ok(NfsStatus::Ok)
    }

    /// Remove File System Object
    ///
    /// [RFC 8881 Section 18.25](https://www.rfc-editor.org/rfc/rfc8881#OP_REMOVE)
    async fn remove(
        &self,
        _req: &mut NfsRequest,
        _args: RemoveArgs,
    ) -> RemoveResult {
        todo!()
    }

    /// Rename Directory Entry
    ///
    /// [RFC 8881 Section 18.26](https://www.rfc-editor.org/rfc/rfc8881#OP_RENAME)
    async fn rename(
        &self,
        _req: &mut NfsRequest,
        _args: RenameArgs,
    ) -> RenameResult {
        todo!()
    }

    /// Restore Saved Filehandle
    ///
    /// [RFC 8881 Section 18.27](https://www.rfc-editor.org/rfc/rfc8881#OP_RESTOREFH)
    async fn restore_fh(&self, _req: &mut NfsRequest) -> RestoreFhResult {
        todo!()
    }

    /// Save Current Filehandle
    ///
    /// [RFC 8881 Section 18.28](https://www.rfc-editor.org/rfc/rfc8881#OP_SAVEFH)
    async fn save_fh(&self, _req: &mut NfsRequest) -> SaveFhResult {
        todo!()
    }

    /// Obtain Available Security
    ///
    /// [RFC 8881 Section 18.29](https://www.rfc-editor.org/rfc/rfc8881#OP_SECINFO)
    async fn security_info(
        &self,
        _req: &mut NfsRequest,
        _args: SecurityInfoArgs,
    ) -> SecurityInfoResult {
        todo!()
    }

    /// Obtain Available Security on Unnamed Object
    ///
    /// [RFC 8881 Section 18.45](https://www.rfc-editor.org/rfc/rfc8881#OP_SECINFO_NO_NAME)
    ///
    /// See also [RFC 8881 Section 13.12](https://www.rfc-editor.org/rfc/rfc8881#file_security_considerations)
    async fn security_info_no_name(
        &self,
        _req: &mut NfsRequest,
        _style: SecurityInfoNoNameStyle,
    ) -> SecurityInfoNoNameResult {
        todo!()
    }

    /// Supply Per-Procedure Sequencing and Control
    ///
    /// [RFC 8881 Section 18.46](https://www.rfc-editor.org/rfc/rfc8881#OP_SEQUENCE)
    async fn sequence(
        &self,
        req: &mut NfsRequest,
        args: SequenceArgs,
    ) -> SequenceResult {
        let mut server = self.inner.lock().expect("Server mutex was poisoned.");

        let Some(session) = server.sessions.get_mut(&args.session_id) else {
            return Err(NfsStatus::BadSession);
        };

        let slot_id = args.slot_id as usize;
        if slot_id >= session.slots.len() {
            session.slots.resize(slot_id + 1, None);
        }

        if let Some(prev_sequence_id) = session.slots[slot_id] {
            if args.sequence_id == prev_sequence_id.wrapping_add(1) {
                session.slots[slot_id] = Some(args.sequence_id);
            } else if args.sequence_id == prev_sequence_id {
                return Err(NfsStatus::RetryUncachedReply);
            } else {
                return Err(NfsStatus::SeqMisordered);
            }
        } else {
            session.slots[slot_id] = Some(args.sequence_id);
        }

        // Store the current session id on the request for any future
        // operations in the same COMPOUND procedure.
        req.set_session(session.client_id, args.session_id);

        #[expect(clippy::cast_possible_truncation, reason = "Fix later.")]
        let highest_slot_id = (session.slots.len() - 1) as u32;

        drop(server);

        Ok(SequenceOk {
            session_id: args.session_id,
            sequence_id: args.sequence_id,
            slot_id: args.slot_id,
            highest_slot_id,
            target_highest_slot_id: highest_slot_id,
            status_flags: 0,
        })
    }

    /// Set Attributes
    ///
    /// [RFC 8881 Section 18.30](https://www.rfc-editor.org/rfc/rfc8881#OP_SETATTR)
    async fn set_attributes(
        &self,
        _req: &mut NfsRequest,
        _args: SetAttributesArgs,
    ) -> SetAttributesResult {
        todo!()
    }

    /// Update SSV for a Client ID
    ///
    /// [RFC 8881 Section 18.47](https://www.rfc-editor.org/rfc/rfc8881#OP_SET_SSV)
    async fn set_ssv(
        &self,
        _req: &mut NfsRequest,
        _args: SetSsvArgs,
    ) -> SetSsvResult {
        todo!()
    }

    /// Test `StateId`s for Validity
    ///
    /// [RFC 8881 Section 18.48](https://www.rfc-editor.org/rfc/rfc8881#OP_TEST_STATEID)
    async fn test_state_ids(
        &self,
        _req: &mut NfsRequest,
        _args: TestStateIdsArgs,
    ) -> TestStateIdsResult {
        todo!()
    }

    /// Verify Same Attributes
    ///
    /// [RFC 8881 Section 18.31](https://www.rfc-editor.org/rfc/rfc8881#OP_VERIFY)
    async fn verify(
        &self,
        _req: &mut NfsRequest,
        _args: VerifyArgs,
    ) -> VerifyResult {
        todo!()
    }

    /// Request Delegation
    ///
    /// [RFC 8881 Section 18.49](https://www.rfc-editor.org/rfc/rfc8881#OP_WANT_DELEGATION)
    async fn want_delegation(
        &self,
        _req: &mut NfsRequest,
        _args: WantDelegationArgs,
    ) -> WantDelegationResult {
        todo!()
    }

    /// Write to File
    ///
    /// [RFC 8881 Section 18.32](https://www.rfc-editor.org/rfc/rfc8881#OP_WRITE)
    async fn write(
        &self,
        _req: &mut NfsRequest,
        _args: WriteArgs,
    ) -> WriteResult {
        todo!()
    }
}
