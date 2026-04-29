use std::io::{Read, Write};

use anyhow::{Result, anyhow};
use byteorder::{BigEndian, ReadBytesExt as _, WriteBytesExt as _};

pub type XdrEndian = BigEndian;

pub trait XdrSerialize {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()>;
}

pub trait XdrDeserialize: Sized {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self>;
}

impl XdrSerialize for bool {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        if *self {
            dest.write_u32::<XdrEndian>(1)
        } else {
            dest.write_u32::<XdrEndian>(2)
        }
    }
}

impl XdrDeserialize for bool {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_u32::<XdrEndian>().map(|val| val > 0)
    }
}

impl XdrSerialize for i32 {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_i32::<XdrEndian>(*self)
    }
}

impl XdrDeserialize for i32 {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_i32::<XdrEndian>()
    }
}

impl XdrSerialize for i64 {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_i64::<XdrEndian>(*self)
    }
}

impl XdrDeserialize for i64 {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_i64::<XdrEndian>()
    }
}

impl XdrSerialize for u32 {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_u32::<XdrEndian>(*self)
    }
}

impl XdrDeserialize for u32 {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_u32::<XdrEndian>()
    }
}

impl XdrSerialize for u64 {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_u64::<XdrEndian>(*self)
    }
}

impl XdrDeserialize for u64 {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_u64::<XdrEndian>()
    }
}

impl<const N: usize> XdrSerialize for [u8; N] {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_all(self)
    }
}

impl<const N: usize> XdrDeserialize for [u8; N] {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let mut ret = [0u8; N];
        src.read_exact(&mut ret)?;
        Ok(ret)
    }
}

pub type XdrOpaque = Vec<u8>;

impl XdrSerialize for Vec<u8> {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        assert!(self.len() < u32::MAX as usize, "Vec<u8> too long");

        #[expect(
            clippy::cast_possible_truncation,
            reason = "Just checked above."
        )]
        let length = self.len() as u32;
        length.serialize(dest)?;
        dest.write_all(self)?;

        let tail = (length % 4) as usize;
        if tail > 0 {
            let pad = 4 - tail;
            let zeros: [u8; 4] = [0, 0, 0, 0];
            dest.write_all(&zeros[..pad])?;
        }

        Ok(())
    }
}

impl XdrDeserialize for Vec<u8> {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let length = u32::deserialize(src)?;

        let mut ret = vec![0u8; length as usize];
        src.read_exact(&mut ret)?;

        let tail = (length % 4) as usize;
        if tail > 0 {
            let pad = 4 - tail;
            let mut zeroes: [u8; 4] = [0, 0, 0, 0];
            src.read_exact(&mut zeroes[..pad])?;
        }

        Ok(ret)
    }
}

impl XdrSerialize for Vec<Vec<u8>> {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        #[expect(clippy::cast_possible_truncation, reason = "Yep.")]
        (self.len() as u32).serialize(dest)?;
        for part in self {
            part.serialize(dest)?;
        }

        Ok(())
    }
}

impl XdrDeserialize for Vec<Vec<u8>> {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let length = u32::deserialize(src)?;
        let mut ret = Self::new();

        for _ in 0..length {
            ret.push(Vec::<u8>::deserialize(src)?);
        }

        Ok(ret)
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct MaxLenBytes<const N: usize>(Vec<u8>);

impl<const N: usize> MaxLenBytes<N> {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        if data.len() > N {
            return Err(anyhow!(
                "Invalid max length bytes buffer, size {} exceeds max length {N}",
                data.len()
            ));
        }

        Ok(Self(data))
    }
}

impl<const N: usize> From<MaxLenBytes<N>> for Vec<u8> {
    fn from(val: MaxLenBytes<N>) -> Self {
        val.0
    }
}

impl<const N: usize> XdrSerialize for MaxLenBytes<N> {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        if self.0.len() > N {
            return Err(std::io::Error::other(format!(
                "Error serializing max length bytes, size {} exceeds maximum size {N}",
                self.0.len()
            )));
        }

        #[expect(
            clippy::cast_possible_truncation,
            reason = "Just checked above."
        )]
        let length = self.0.len() as u32;
        length.serialize(dest)?;
        dest.write_all(&self.0)?;

        let tail = (length % 4) as usize;
        if tail > 0 {
            let pad = 4 - tail;
            let zeros: [u8; 4] = [0, 0, 0, 0];
            dest.write_all(&zeros[..pad])?;
        }

        Ok(())
    }
}

impl<const N: usize> XdrDeserialize for MaxLenBytes<N> {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let length = u32::deserialize(src)?;

        if length as usize > N {
            return Err(std::io::Error::other(format!(
                "Error deserializing max length bytes, size {length} exceeds maximum size {N}"
            )));
        }

        let mut ret = vec![0u8; length as usize];
        src.read_exact(&mut ret)?;

        let tail = (length % 4) as usize;
        if tail > 0 {
            let pad = 4 - tail;
            let mut zeroes: [u8; 4] = [0, 0, 0, 0];
            src.read_exact(&mut zeroes[..pad])?;

            assert_eq!(
                zeroes,
                [0u8, 0, 0, 0],
                "Invalid padding data included non-zero values."
            );
        }

        Ok(Self(ret))
    }
}

impl XdrSerialize for Vec<u32> {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        assert!(self.len() * 4 < u32::MAX as usize, "Vec<u32> too long.");

        #[expect(
            clippy::cast_possible_truncation,
            reason = "Just checked above."
        )]
        let length = self.len() as u32;
        length.serialize(dest)?;

        for i in self {
            i.serialize(dest)?;
        }

        Ok(())
    }
}

impl XdrDeserialize for Vec<u32> {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let length = u32::deserialize(src)? as usize;

        let mut ret = vec![0u32; length];
        for pos in &mut ret {
            *pos = u32::deserialize(src)?;
        }

        Ok(ret)
    }
}

pub struct OptionalData<T>(Vec<T>);

impl<T: XdrSerialize> XdrSerialize for OptionalData<T> {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        for val in &self.0 {
            true.serialize(dest)?;
            val.serialize(dest)?;
        }

        false.serialize(dest)
    }
}

impl<T: XdrDeserialize> XdrDeserialize for OptionalData<T> {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let mut ret = Vec::new();
        loop {
            let has_next = bool::deserialize(src)?;
            if !has_next {
                break;
            }

            ret.push(T::deserialize(src)?);
        }

        Ok(Self(ret))
    }
}

impl<T: XdrSerialize> XdrSerialize for Option<T> {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        if let Some(val) = self {
            1u32.serialize(dest)?;
            val.serialize(dest)?;
        } else {
            0u32.serialize(dest)?;
        }

        Ok(())
    }
}

impl<T: XdrDeserialize> XdrDeserialize for Option<T> {
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let length = u32::deserialize(src)?;

        if length > 1 {
            return Err(std::io::Error::other(format!(
                "Invalid length for Option<T>: {length}"
            )));
        }

        if length == 1 {
            Ok(Some(T::deserialize(src)?))
        } else {
            Ok(None)
        }
    }
}

#[macro_export]
macro_rules! serde_enum {
    ($t:ident) => {
        impl XdrSerialize for $t {
            fn serialize<W: std::io::Write>(
                &self,
                dest: &mut W,
            ) -> std::io::Result<()> {
                use byteorder::WriteBytesExt;
                dest.write_u32::<$crate::xdr::XdrEndian>(*self as u32)
            }
        }

        impl XdrDeserialize for $t {
            fn deserialize<R: std::io::Read>(
                src: &mut R,
            ) -> std::io::Result<Self> {
                let val = u32::deserialize(src)?;
                if let Some(var) =
                    num_traits::cast::FromPrimitive::from_u32(val)
                {
                    Ok(var)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid value for {}", stringify!($t)),
                    ))
                }
            }
        }
    };
}

pub(crate) use serde_enum;

/// Serialize a vector of things that implement `XdrSerialize`.
///
/// Do *NOT* use this for Vec<u8> because this function does not implement
/// padding bytes as required. Although, you can use this on Vec<Vec<u8>> just
/// fine.
pub fn serialize_vec<W: Write, T: XdrSerialize>(
    dest: &mut W,
    data: &[T],
) -> std::io::Result<()> {
    #[expect(clippy::cast_possible_truncation, reason = "Yep.")]
    (data.len() as u32).serialize(dest)?;
    for item in data {
        item.serialize(dest)?;
    }

    Ok(())
}

/// Deserialze a vector of things that implement `XdrDeserialize`.
///
/// See `serialize_vec`, don't use this for Vec<u8> values.
pub fn deserialize_vec<R: Read, T: XdrDeserialize>(
    src: &mut R,
) -> std::io::Result<Vec<T>> {
    let length = u32::deserialize(src)?;
    let mut ret = Vec::with_capacity(length as usize);

    for _ in 0..length {
        ret.push(T::deserialize(src)?);
    }

    Ok(ret)
}

/// Serialize a vector of vectors of things that implement `XdrSerialize`.
///
/// Don't use this when T = u8. Use the trait based versions. See `serialize_vec`
pub fn serialize_vec_vec<W: Write, T: XdrSerialize>(
    dest: &mut W,
    data: &[Vec<T>],
) -> std::io::Result<()> {
    #[expect(clippy::cast_possible_truncation, reason = "Yep.")]
    (data.len() as u32).serialize(dest)?;
    for items in data {
        serialize_vec(dest, items)?;
    }

    Ok(())
}

/// Deserialze a vector of vectors of things that implement `XdrDeserialize`.
///
/// See `serialize_vec`, don't use this for u8 values.
pub fn deserialize_vec_vec<R: Read, T: XdrDeserialize>(
    src: &mut R,
) -> std::io::Result<Vec<Vec<T>>> {
    let length = u32::deserialize(src)?;
    let mut ret = Vec::with_capacity(length as usize);

    for _ in 0..length {
        let items = deserialize_vec::<_, T>(src)?;
        ret.push(items);
    }

    Ok(ret)
}
