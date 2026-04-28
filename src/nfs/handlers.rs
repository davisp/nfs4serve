use crate::nfs::types::*;

pub trait NfsHandler {
    /// Check Access Rights
    ///
    /// [RFC 8881 Section 18.1](https://www.rfc-editor.org/rfc/rfc8881#OP_ACCESS)
    fn access(&self, args: AccessArgs) -> AccessResult;

    /// Backchannel Control
    ///
    /// [RFC 8881 Section 18.33](https://www.rfc-editor.org/rfc/rfc8881#OP_BACKCHANNEL_CTL)
    fn backchannel_control(
        &self,
        args: BackchannelControlArgs,
    ) -> BackchannelControlResult;

    /// Bind Connection To Session
    ///
    /// [RFC 8881 Section 18.34](https://www.rfc-editor.org/rfc/rfc8881#OP_BIND_CONN_TO_SESSION)
    fn bind_connection_to_session(
        &self,
        args: BindConnectionToSessionArgs,
    ) -> BindConnectionToSessionResult;

    /// Close File
    ///
    /// [RFC 8881 Section 18.2](https://www.rfc-editor.org/rfc/rfc8881#OP_CLOSE)
    fn close(&self, args: CloseArgs) -> CloseResult;

    /// Commit Cached Data
    ///
    /// [RFC 8881 Section 18.3](https://www.rfc-editor.org/rfc/rfc8881#OP_COMMIT)
    fn commit(&self, args: CommitArgs) -> CommitResult;

    /// Create Non-Regular File Object
    ///
    /// [RFC 8881 Section 18.4](https://www.rfc-editor.org/rfc/rfc8881#OP_CREATE)
    fn create(&self, args: CreateArgs) -> CreateResult;

    /// Create New Session and Confirm Client ID
    ///
    /// [RFC 8881 Section 18.36](https://www.rfc-editor.org/rfc/rfc8881#OP_CREATE_SESSION)
    fn create_session(&self, args: CreateSessionArgs) -> CreateSessionResult;

    /// Purge Delegations Awaiting Recovery
    ///
    /// [RFC 8881 Section 18.5](https://www.rfc-editor.org/rfc/rfc8881#OP_DELEGPURGE)
    fn purge_delegations(
        &self,
        args: PurgeDelegationsArgs,
    ) -> PurgeDelegationsResult;

    /// Return Delegation
    ///
    /// [RFC 8881 Section 18.6](https://www.rfc-editor.org/rfc/rfc8881#OP_DELEGRETURN)
    fn return_delegation(
        &self,
        args: ReturnDelegationArgs,
    ) -> ReturnDelegationResult;

    /// Destroy Client ID
    ///
    /// [RFC 8881 Section 18.50](https://www.rfc-editor.org/rfc/rfc8881#OP_DESTROY_CLIENTID)
    fn destroy_client_id(
        &self,
        args: DestroyClientIdArgs,
    ) -> DestroyClientIdResult;

    /// Destroy Session
    ///
    /// [RFC 8881 Section 18.37](https://www.rfc-editor.org/rfc/rfc8881#OP_DESTROY_SESSION)
    fn destroy_session(&self, args: DestroySessionArgs)
    -> DestroySessionResult;

    /// Instantiate a Client ID
    ///
    /// [RFC 8881 Section 18.35](https://www.rfc-editor.org/rfc/rfc8881#OP_EXCHANGE_ID)
    fn exchange_id(&self, args: ExchangeIdArgs) -> ExchangeIdResult;

    /// Free State ID with No Locks
    ///
    /// [RFC 8881 Section 18.38](https://www.rfc-editor.org/rfc/rfc8881#OP_FREE_STATEID)
    fn free_state_id(&self, args: FreeStateIdArgs) -> FreeStateIdResult;

    /// Get Attributes
    ///
    /// [RFC 8881 Section 18.7](https://www.rfc-editor.org/rfc/rfc8881#OP_GETATTR)
    fn get_attributes(&self, args: GetAttributesArgs) -> GetAttributesResult;

    /// Get Device Info
    ///
    /// [RFC 8881 Section 18.40](https://www.rfc-editor.org/rfc/rfc8881#OP_GETDEVICEINFO)
    fn get_device_info(&self, args: GetDeviceInfoArgs) -> GetDeviceInfoResult;

    /// Get Device List
    ///
    /// [RFC 8881 Section 18.41](https://www.rfc-editor.org/rfc/rfc8881#OP_GETDEVICELIST)
    fn get_device_list(&self, args: GetDeviceListArgs) -> GetDeviceListResult;

    /// Get Current Filehandle
    ///
    /// [RFC 8881 Section 18.8](https://www.rfc-editor.org/rfc/rfc8881#OP_GETFH)
    fn get_current_fh(&self) -> GetFhResult;

    /// Get Directory Delegation
    ///
    /// [RFC 8881 Section 18.39](https://www.rfc-editor.org/rfc/rfc8881#OP_GET_DIR_DELEGATION)
    fn get_directory_delegation(
        &self,
        args: GetDirectoryDelegationArgs,
    ) -> GetDirectoryDelegationResult;

    /// Commit Writes Made Using a Layout
    ///
    /// [RFC 8881 Section 18.42](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTCOMMIT)
    fn layout_commit(&self, args: LayoutCommitArgs) -> LayoutCommitResult;

    /// Get Layout Information
    ///
    /// [RFC 8881 Section 18.43](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTGET)
    fn layout_get(&self, args: LayoutGetArgs) -> LayoutGetResult;

    /// Release Layout Information
    ///
    /// [RFC 8881 Section 18.44](https://www.rfc-editor.org/rfc/rfc8881#OP_LAYOUTRETURN)
    fn layout_return(&self, args: LayoutReturnArgs) -> LayoutReturnResult;

    /// Create Link to File
    ///
    /// [RFC 8881 Section 18.9](https://www.rfc-editor.org/rfc/rfc8881#OP_LINK)
    fn link(&self, args: LinkArgs) -> LinkResult;

    /// Create a Lock
    ///
    /// [RFC 8881 Section 18.10](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCK)
    fn lock(&self, args: LockArgs) -> LockResult;

    /// Test for Lock
    ///
    /// [RFC 8881 Section 18.11](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCKT)
    fn lock_test(&self, args: LockTestArgs) -> LockTestResult;

    /// Release a Lock
    ///
    /// [RFC 8881 Section 18.12](https://www.rfc-editor.org/rfc/rfc8881#OP_LOCKU)
    fn lock_release(&self, args: LockReleaseArgs) -> LockReleaseResult;

    /// Lookup Filename
    ///
    /// [RFC 8881 Section 18.13](https://www.rfc-editor.org/rfc/rfc8881#OP_LOOKUP)
    fn lookup(&self, args: LookupArgs) -> LookupResult;

    /// Lookup Parent Directory
    ///
    /// [RFC 8881 Section 18.14](https://www.rfc-editor.org/rfc/rfc8881#OP_LOOKUPP)
    fn lookup_parent(&self) -> LookupParentResult;

    /// Verify Difference in Attributes
    ///
    /// [RFC 8881 Section 18.15](https://www.rfc-editor.org/rfc/rfc8881#OP_NVERIFY)
    fn verify_attribute_difference(
        &self,
        args: VerifyAttributeDifferenceArgs,
    ) -> VerifyAttributeDifferenceResult;

    /// Open a Regular File
    ///
    /// [RFC 8881 Section 18.16](https://www.rfc-editor.org/rfc/rfc8881#OP_OPEN)
    fn open(&self, args: OpenArgs) -> OpenResult;

    /// Open Named Attribute Directory
    ///
    /// [RFC 8881 Section 18.17](https://www.rfc-editor.org/rfc/rfc8881#OP_OPENATTR)
    fn open_attributes(&self, args: OpenAttributesArgs)
    -> OpenAttributesResult;

    /// Reduce Open File Access
    ///
    /// [RFC 8881 Section 18.18](https://www.rfc-editor.org/rfc/rfc8881#OP_OPEN_DOWNGRADE)
    fn open_downgrade(&self, args: OpenDowngradeArgs) -> OpenDowngradeResult;

    /// Set Current Filehandle
    ///
    /// [RFC 8881 Section 18.19](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTFH)
    fn put_fh(&self, args: PutFhArgs) -> PutFhResult;

    /// Set Public Filehandle
    ///
    /// [RFC 8881 Section 18.20](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTPUBFH)
    fn put_public_fh(&self) -> PutPublicFhResult;

    /// Set Root Filehandle
    ///
    /// [RFC 8881 Section 18.21](https://www.rfc-editor.org/rfc/rfc8881#OP_PUTROOTFH)
    fn put_root_fh(&self) -> PutRootFhResult;

    /// Read from File
    ///
    /// [RFC 8881 Section 18.22](https://www.rfc-editor.org/rfc/rfc8881#OP_READ)
    fn read(&self, args: ReadArgs) -> ReadResult;

    /// Read Directory
    ///
    /// [RFC 8881 Section 18.23](https://www.rfc-editor.org/rfc/rfc8881#OP_READDIR)
    fn read_directory(&self, args: ReadDirectoryArgs) -> ReadDirectoryResult;

    /// Read Symbolic Link
    ///
    /// [RFC 8881 Section 18.24](https://www.rfc-editor.org/rfc/rfc8881#OP_READLINK)
    fn read_link(&self) -> ReadLinkResult;

    /// Indicate Reclaims Finished
    ///
    /// [RFC 8881 Section 18.51](https://www.rfc-editor.org/rfc/rfc8881#OP_RECLAIM_COMPLETE)
    fn reclaim_complete(
        &self,
        args: ReclaimCompleteArgs,
    ) -> ReclaimCompleteResult;

    /// Remove File System Object
    ///
    /// [RFC 8881 Section 18.25](https://www.rfc-editor.org/rfc/rfc8881#OP_REMOVE)
    fn remove(&self, args: RemoveArgs) -> RemoveResult;

    /// Rename Directory Entry
    ///
    /// [RFC 8881 Section 18.26](https://www.rfc-editor.org/rfc/rfc8881#OP_RENAME)
    fn rename(&self, args: RenameArgs) -> RenameResult;

    /// Restore Saved Filehandle
    ///
    /// [RFC 8881 Section 18.27](https://www.rfc-editor.org/rfc/rfc8881#OP_RESTOREFH)
    fn restore_fh(&self) -> RestoreFhResult;

    /// Save Current Filehandle
    ///
    /// [RFC 8881 Section 18.28](https://www.rfc-editor.org/rfc/rfc8881#OP_SAVEFH)
    fn save_fh(&self) -> SaveFhResult;

    /// Obtain Available Security
    ///
    /// [RFC 8881 Section 18.29](https://www.rfc-editor.org/rfc/rfc8881#OP_SECINFO)
    fn security_info(&self, args: SecurityInfoArgs) -> SecurityInfoResult;

    /// Obtain Available Security on Unnamed Object
    ///
    /// [RFC 8881 Section 18.45](https://www.rfc-editor.org/rfc/rfc8881#OP_SECINFO_NO_NAME)
    ///
    /// See also [RFC 8881 Section 13.12](https://www.rfc-editor.org/rfc/rfc8881#file_security_considerations)
    fn security_info_no_name(
        &self,
        style: SecurityInfoNoNameStyle,
    ) -> SecurityInfoNoNameResult;

    /// Supply Per-Procedure Sequencing and Control
    ///
    /// [RFC 8881 Section 18.46](https://www.rfc-editor.org/rfc/rfc8881#OP_SEQUENCE)
    fn sequence(&self, args: SequenceArgs) -> SequenceResult;

    /// Set Attributes
    ///
    /// [RFC 8881 Section 18.30](https://www.rfc-editor.org/rfc/rfc8881#OP_SETATTR)
    fn set_attributes(&self, args: SetAttributesArgs) -> SetAttributesResult;

    /// Update SSV for a Client ID
    ///
    /// [RFC 8881 Section 18.47](https://www.rfc-editor.org/rfc/rfc8881#OP_SET_SSV)
    fn set_ssv(&self, args: SetSsvArgs) -> SetSsvResult;

    /// Test `StateId`s for Validity
    ///
    /// [RFC 8881 Section 18.48](https://www.rfc-editor.org/rfc/rfc8881#OP_TEST_STATEID)
    fn test_state_ids(&self, args: TestStateIdsArgs) -> TestStateIdsResult;

    /// Verify Same Attributes
    ///
    /// [RFC 8881 Section 18.31](https://www.rfc-editor.org/rfc/rfc8881#OP_VERIFY)
    fn verify(&self, args: VerifyArgs) -> VerifyResult;

    /// Request Delegation
    ///
    /// [RFC 8881 Section 18.49](https://www.rfc-editor.org/rfc/rfc8881#OP_WANT_DELEGATION)
    fn want_delegation(&self, args: WantDelegationArgs)
    -> WantDelegationResult;

    /// Write to File
    ///
    /// [RFC 8881 Section 18.32](https://www.rfc-editor.org/rfc/rfc8881#OP_WRITE)
    fn write(&self, args: WriteArgs) -> WriteResult;
}
