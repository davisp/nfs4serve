use std::io::Cursor;

use anyhow::Result;
use num_traits::cast::{FromPrimitive as _, ToPrimitive as _};

use crate::nfs::api::{NfsAttribute, NfsAttributes};
use crate::nfs::types::FileAttrs;
use crate::xdr::{self, XdrSerialize as _};

pub fn protocol_to_api(attrs: &[u32]) -> Vec<NfsAttribute> {
    let mut ret = Vec::new();

    for (idx, elem) in attrs.iter().enumerate() {
        for i in 0..31 {
            if ((1 << i) & elem) != 0 {
                ret.push(
                    #[expect(clippy::cast_possible_truncation, reason = "Yep")]
                    NfsAttribute::from_u32((idx * 32 + i) as u32)
                        .unwrap_or(NfsAttribute::Illegal),
                );
            }
        }
    }

    ret
}

#[expect(clippy::too_many_lines, reason = "It's a big match.")]
pub fn values_to_protocol(
    mut requested: Vec<NfsAttribute>,
    values: NfsAttributes,
) -> Result<FileAttrs> {
    requested.sort_by_key(|attr| attr.to_u32().unwrap());

    let mut mask = vec![0u32; 3];
    let mut attrs = Cursor::new(Vec::new());

    for attr in requested {
        match attr {
            NfsAttribute::SupportedAttributes => {
                set_bit(attr, &mut mask);

                let mut bitset = Vec::new();
                for sattr in &values.supported_attributes {
                    set_bit(*sattr, &mut bitset);
                }
                bitset.serialize(&mut attrs)?;
            }
            NfsAttribute::FileType => {
                set_bit(attr, &mut mask);
                values.file_type.serialize(&mut attrs)?;
            }
            NfsAttribute::ExpirationPolicy => {
                set_bit(attr, &mut mask);
                values.expiration_policy.serialize(&mut attrs)?;
            }
            NfsAttribute::Changed => {
                set_bit(attr, &mut mask);
                values.changed.serialize(&mut attrs)?;
            }
            NfsAttribute::Size => {
                set_bit(attr, &mut mask);
                values.size.serialize(&mut attrs)?;
            }
            NfsAttribute::LinkSupport => {
                set_bit(attr, &mut mask);
                values.link_support.serialize(&mut attrs)?;
            }
            NfsAttribute::SymlinkSupport => {
                set_bit(attr, &mut mask);
                values.symlink_support.serialize(&mut attrs)?;
            }
            NfsAttribute::NamedAttributes => {
                set_bit(attr, &mut mask);
                values.named_attributes.serialize(&mut attrs)?;
            }
            NfsAttribute::FileSystemId => {
                set_bit(attr, &mut mask);
                values.file_system_id.serialize(&mut attrs)?;
            }
            NfsAttribute::UniqueHandles => {
                set_bit(attr, &mut mask);
                values.unique_handles.serialize(&mut attrs)?;
            }
            NfsAttribute::LeaseTime => {
                set_bit(attr, &mut mask);
                values.lease_time.serialize(&mut attrs)?;
            }
            NfsAttribute::FileHandle => {
                set_bit(attr, &mut mask);
                values.file_handle.serialize(&mut attrs)?;
            }
            NfsAttribute::ExclusiveCreateAttributes => {
                set_bit(attr, &mut mask);

                let mut bitset = Vec::new();
                for sattr in &values.exclusive_create_attributes {
                    set_bit(*sattr, &mut bitset);
                }
                bitset.serialize(&mut attrs)?;
            }
            NfsAttribute::Acl => {
                values
                    .acl
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        xdr::serialize_vec(&mut attrs, val)
                    })
                    .transpose()?;
            }
            NfsAttribute::AclSupport => {
                values
                    .acl_support
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::Archive => {
                values
                    .archive
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::CanSetTime => {
                values
                    .can_set_time
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::CaseInsensitive => {
                values
                    .case_insensitive
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::CasePreserving => {
                values
                    .case_preserving
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::ChangePolicy => {
                values
                    .change_policy
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::ChownRestricted => {
                values
                    .chown_restricted
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::DAcl => {
                values
                    .dacl
                    .as_ref()
                    .map(|(flag, aces)| -> std::io::Result<_> {
                        set_bit(attr, &mut mask);
                        flag.serialize(&mut attrs)?;
                        xdr::serialize_vec(&mut attrs, aces)
                    })
                    .transpose()?;
            }
            NfsAttribute::DirectoryNotificationDelay => {
                values
                    .directory_notification_delay
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::DirectoryEntryNotificationDelay => {
                values
                    .directory_entry_notification_delay
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileId => {
                values
                    .file_id
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FilesAvailable => {
                values
                    .files_available
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FilesFree => {
                values
                    .files_free
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FilesTotal => {
                values
                    .files_total
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemCharsetAbilities => {
                values
                    .file_system_charset_abilities
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemLayoutType => {
                values
                    .file_system_layout_type
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        xdr::serialize_vec(&mut attrs, val)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemLocations => {
                values
                    .file_system_locations
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemLocationsInfo => {
                values
                    .file_system_locations_info
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemStatus => {
                values
                    .file_system_status
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::Hidden => {
                values
                    .hidden
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::Homogeneous => {
                values
                    .homogeneous
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::LayoutAlignment => {
                values
                    .layout_alignment
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::LayoutBlockSize => {
                values
                    .layout_block_size
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::LayoutType => {
                values
                    .layout_type
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        xdr::serialize_vec(&mut attrs, val)
                    })
                    .transpose()?;
            }
            NfsAttribute::MaxFileSize => {
                values
                    .max_file_size
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::MaxLinks => {
                values
                    .max_links
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::MaxNameLength => {
                values
                    .max_name_length
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::MaxReadLength => {
                values
                    .max_read_length
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::MaxWriteLength => {
                values
                    .max_write_length
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::MimeType => {
                values
                    .mime_type
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        let data = val.as_bytes().to_vec();
                        data.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::Mode => {
                values
                    .mode
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::MountedOnFileId => {
                values
                    .mounted_on_file_id
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::NoTruncation => {
                values
                    .no_truncation
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::NumLinks => {
                values
                    .num_links
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::Owner => {
                values
                    .owner
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        let data = val.as_bytes().to_vec();
                        data.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::OwnerGroup => {
                values
                    .owner_group
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        let data = val.as_bytes().to_vec();
                        data.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::QuotaAvailableHard => {
                values
                    .quota_available_hard
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::QuotaAvailableSoft => {
                values
                    .quota_available_soft
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::QuotaUsed => {
                values
                    .quota_used
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::RawDevice => {
                values
                    .raw_device
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }

            NfsAttribute::SAcl => {
                values
                    .sacl
                    .as_ref()
                    .map(|(flag, aces)| -> std::io::Result<_> {
                        set_bit(attr, &mut mask);
                        flag.serialize(&mut attrs)?;
                        xdr::serialize_vec(&mut attrs, aces)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemSpaceAvailable => {
                values
                    .file_system_space_available
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemSpaceFree => {
                values
                    .file_system_space_free
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemSpaceTotal => {
                values
                    .file_system_space_total
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::FileSystemSpaceUsed => {
                values
                    .file_system_space_used
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::IsSystemFile => {
                values
                    .is_system_file
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::TimeAccess => {
                values
                    .time_access
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::TimeBackup => {
                values
                    .time_backup
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::TimeCreate => {
                values
                    .time_create
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::TimeDelta => {
                values
                    .time_delta
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::TimeMetadata => {
                values
                    .time_metadata
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::TimeModify => {
                values
                    .time_modify
                    .as_ref()
                    .map(|val| {
                        set_bit(attr, &mut mask);
                        val.serialize(&mut attrs)
                    })
                    .transpose()?;
            }
            NfsAttribute::Illegal
            | NfsAttribute::LayoutHint
            | NfsAttribute::MetadataServerSizeThreshold
            | NfsAttribute::ModeSetMasked
            | NfsAttribute::ReadAttributeError
            | NfsAttribute::GetEventRetention
            | NfsAttribute::SetEventRetention
            | NfsAttribute::GetRetention
            | NfsAttribute::SetRetention
            | NfsAttribute::HoldRetention
            | NfsAttribute::SetTimeAccess
            | NfsAttribute::SetTimeModify => {}
        }
    }

    Ok(FileAttrs {
        mask,
        attrs: attrs.into_inner(),
    })
}

fn is_bit_set(attr: NfsAttribute, attrs: &[u32]) -> bool {
    let num = attr.to_u32().unwrap();
    let elem = (num / 32) as usize;

    if elem >= attrs.len() {
        return false;
    }

    let bit_mask = 1u32 << (num % 32);

    (attrs[elem] & bit_mask) != 0
}

fn set_bit(attr: NfsAttribute, attrs: &mut Vec<u32>) {
    let num = attr.to_u32().unwrap();
    let elem = (num / 32) as usize;

    if elem >= attrs.len() {
        attrs.resize(elem + 1, 0);
    }

    let bit_mask = 1u32 << (num % 32);
    attrs[elem] |= bit_mask;
}
