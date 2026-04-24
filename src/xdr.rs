use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt as _, WriteBytesExt as _};

pub type XdrEndian = BigEndian;

pub trait XdrSerde: Sized {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()>;
    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self>;
}

impl XdrSerde for bool {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        if *self {
            dest.write_u32::<XdrEndian>(1)
        } else {
            dest.write_u32::<XdrEndian>(2)
        }
    }

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_u32::<XdrEndian>().map(|val| val > 0)
    }
}

impl XdrSerde for i32 {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_i32::<XdrEndian>(*self)
    }

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_i32::<XdrEndian>()
    }
}

impl XdrSerde for i64 {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_i64::<XdrEndian>(*self)
    }

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_i64::<XdrEndian>()
    }
}

impl XdrSerde for u32 {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_u32::<XdrEndian>(*self)
    }

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_u32::<XdrEndian>()
    }
}

impl XdrSerde for u64 {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_u64::<XdrEndian>(*self)
    }

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        src.read_u64::<XdrEndian>()
    }
}

impl<const N: usize> XdrSerde for [u8; N] {
    fn serialize<W: Write>(&self, dest: &mut W) -> std::io::Result<()> {
        dest.write_all(self)
    }

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let mut ret = [0u8; N];
        src.read_exact(&mut ret)?;
        Ok(ret)
    }
}

impl XdrSerde for Vec<u8> {
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

impl XdrSerde for Vec<u32> {
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

    fn deserialize<R: Read>(src: &mut R) -> std::io::Result<Self> {
        let length = u32::deserialize(src)? as usize;

        let mut ret = vec![0u32; length];
        for pos in &mut ret {
            *pos = u32::deserialize(src)?;
        }

        Ok(ret)
    }
}

#[macro_export]
macro_rules! serde_enum {
    ($t:ident) => {
        impl XdrSerde for $t {
            fn serialize<W: std::io::Write>(
                &self,
                dest: &mut W,
            ) -> std::io::Result<()> {
                use byteorder::WriteBytesExt;
                dest.write_u32::<$crate::xdr::XdrEndian>(*self as u32)
            }

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
