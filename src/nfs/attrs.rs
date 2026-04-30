use std::io::Cursor;

use anyhow::Result;

use crate::nfs::api::{
    NfsAttribute, NfsAttributeValue, NfsExpirationPolicy, NfsHandleType,
};
use crate::nfs::constants;
use crate::nfs::types::{FileAttrs, NfsFileType};
use crate::xdr::XdrSerialize as _;

pub fn protocol_to_api(attrs: &[u32]) -> Vec<NfsAttribute> {
    let mut ret = Vec::new();

    for attr in NfsAttribute::all() {
        if is_bit_set(attr, attrs) {
            ret.push(attr);
        }
    }

    ret
}

pub fn values_to_protocol(
    mut vals: Vec<NfsAttributeValue>,
) -> Result<FileAttrs> {
    // First, sort the return values by order of the attribute number
    // because that's the order they have to be serialized in.
    vals.sort_by_key(|val| attr_to_protocol(attr_value_to_attr(val)));

    let mut bitmap = Vec::new();
    let mut writer = Cursor::new(Vec::new());

    for val in vals {
        bitmap = set_bit(attr_value_to_attr(&val), bitmap);

        match val {
            NfsAttributeValue::SupportedAttributes(attrs)
            | NfsAttributeValue::ExclusiveCreateAttributes(attrs) => {
                let mut sa_bitset = Vec::new();
                for attr in attrs {
                    sa_bitset = set_bit(attr, sa_bitset);
                }
                sa_bitset.serialize(&mut writer)?;
            }
            NfsAttributeValue::HandleType(typ) => {
                let proto = match typ {
                    NfsHandleType::Regular => NfsFileType::Regular,
                    NfsHandleType::Directory => NfsFileType::Directory,
                    NfsHandleType::Block => NfsFileType::Block,
                    NfsHandleType::Character => NfsFileType::Character,
                    NfsHandleType::Link => NfsFileType::Link,
                    NfsHandleType::Socket => NfsFileType::Socket,
                    NfsHandleType::Fifo => NfsFileType::Fifo,
                    NfsHandleType::AttributeDirectory => {
                        NfsFileType::AttrDirectory
                    }
                    NfsHandleType::NamedAttribute => NfsFileType::NamedAttr,
                };
                proto.serialize(&mut writer)?;
            }
            NfsAttributeValue::ExpirationPolicy(policy) => {
                let policy = match policy {
                    NfsExpirationPolicy::Persistent => constants::FH_PERSISTENT,
                    NfsExpirationPolicy::Volatile => constants::FH_VOLATILE_ANY,
                    NfsExpirationPolicy::VolatileExceptWhenOpen => {
                        constants::FH_VOLATILE_ANY
                            | constants::FH_NOEXPIRE_WITH_OPEN
                    }
                };
                policy.serialize(&mut writer)?;
            }
            NfsAttributeValue::Changed(val) | NfsAttributeValue::Size(val) => {
                val.serialize(&mut writer)?;
            }
            NfsAttributeValue::LinkSupport(val)
            | NfsAttributeValue::SymlinkSupport(val)
            | NfsAttributeValue::NamedAttributes(val)
            | NfsAttributeValue::UniqueHandles(val) => {
                val.serialize(&mut writer)?;
            }
            NfsAttributeValue::FileSystemId(val) => {
                val.major.serialize(&mut writer)?;
                val.minor.serialize(&mut writer)?;
            }
            NfsAttributeValue::LeaseTime(val) => {
                val.serialize(&mut writer)?;
            }
            NfsAttributeValue::FileHandle(fh) => {
                fh.serialize(&mut writer)?;
            }
        }
    }

    Ok(FileAttrs {
        mask: bitmap,
        attrs: writer.into_inner(),
    })
}

fn attr_to_protocol(attr: NfsAttribute) -> u32 {
    match attr {
        NfsAttribute::SupportedAttributes => 0,
        NfsAttribute::HandleType => 1,
        NfsAttribute::ExpirationPolicy => 2,
        NfsAttribute::Changed => 3,
        NfsAttribute::Size => 4,
        NfsAttribute::LinkSupport => 5,
        NfsAttribute::SymlinkSupport => 6,
        NfsAttribute::NamedAttributes => 7,
        NfsAttribute::FileSystemId => 8,
        NfsAttribute::UniqueHandles => 9,
        NfsAttribute::LeaseTime => 10,
        NfsAttribute::ReadAttributeError => 11,
        NfsAttribute::FileHandle => 19,
        NfsAttribute::ExclusiveCreateAttributes => 75,
    }
}

fn attr_value_to_attr(val: &NfsAttributeValue) -> NfsAttribute {
    match val {
        NfsAttributeValue::SupportedAttributes(_) => {
            NfsAttribute::SupportedAttributes
        }
        NfsAttributeValue::HandleType(_) => NfsAttribute::HandleType,
        NfsAttributeValue::ExpirationPolicy(_) => {
            NfsAttribute::ExpirationPolicy
        }
        NfsAttributeValue::Changed(_) => NfsAttribute::Changed,
        NfsAttributeValue::Size(_) => NfsAttribute::Size,
        NfsAttributeValue::LinkSupport(_) => NfsAttribute::LinkSupport,
        NfsAttributeValue::SymlinkSupport(_) => NfsAttribute::SymlinkSupport,
        NfsAttributeValue::NamedAttributes(_) => NfsAttribute::NamedAttributes,
        NfsAttributeValue::FileSystemId(_) => NfsAttribute::FileSystemId,
        NfsAttributeValue::UniqueHandles(_) => NfsAttribute::UniqueHandles,
        NfsAttributeValue::LeaseTime(_) => NfsAttribute::LeaseTime,
        NfsAttributeValue::FileHandle(_) => NfsAttribute::FileHandle,
        NfsAttributeValue::ExclusiveCreateAttributes(_) => {
            NfsAttribute::ExclusiveCreateAttributes
        }
    }
}

fn is_bit_set(attr: NfsAttribute, attrs: &[u32]) -> bool {
    let num = attr_to_protocol(attr);
    let elem = (num / 32) as usize;

    if elem >= attrs.len() {
        return false;
    }

    let bit_mask = 1u32 << (num % 32);

    (attrs[elem] & bit_mask) != 0
}

fn set_bit(attr: NfsAttribute, mut attrs: Vec<u32>) -> Vec<u32> {
    let num = attr_to_protocol(attr);
    let elem = (num / 32) as usize;

    if elem >= attrs.len() {
        attrs.resize(elem + 1, 0);
    }

    let bit_mask = 1u32 << (num % 32);
    attrs[elem] |= bit_mask;

    attrs
}
