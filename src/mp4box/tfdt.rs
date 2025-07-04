use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TfdtBox {
    pub version: u8,
    pub flags: u32,
    pub base_media_decode_time: u64,
}

impl TfdtBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TfdtBox
    }

    pub fn get_size(&self) -> u64 {
        let mut sum = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            sum += 8;
        } else {
            sum += 4;
        }
        sum
    }
}

impl Mp4Box for TfdtBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn summary(&self) -> Result<String> {
        let s = format!("base_media_decode_time={}", self.base_media_decode_time);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TfdtBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let base_media_decode_time = if version == 1 {
            reader.read_u64::<BigEndian>()?
        } else if version == 0 {
            reader.read_u32::<BigEndian>()? as u64
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        };

        skip_bytes_to(reader, start + size)?;

        Ok(TfdtBox {
            version,
            flags,
            base_media_decode_time,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TfdtBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.base_media_decode_time)?;
        } else if self.version == 0 {
            writer.write_u32::<BigEndian>(self.base_media_decode_time as u32)?;
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_tfdt32() {
        let src_box = TfdtBox {
            version: 0,
            flags: 0,
            base_media_decode_time: 0,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TfdtBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TfdtBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_tfdt64() {
        let src_box = TfdtBox {
            version: 1,
            flags: 0,
            base_media_decode_time: 0,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::TfdtBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = TfdtBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
