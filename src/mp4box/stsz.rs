use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use std::mem::size_of;

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StszBox {
    pub version: u8,
    pub flags: u32,
    pub sample_size: u32,
    pub sample_count: u32,
    pub sample_sizes: Vec<u32>,
}

impl StszBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::StszBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8 + (4 * self.sample_sizes.len() as u64)
    }
}

impl Mp4Box for StszBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "sample_size={} sample_count={} sample_sizes={}",
            self.sample_size,
            self.sample_count,
            self.sample_sizes.len()
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StszBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let header_size = HEADER_SIZE + HEADER_EXT_SIZE;
        let other_size = size_of::<u32>() + size_of::<u32>(); // sample_size + sample_count
        let sample_size = reader.read_u32::<BigEndian>()?;
        let stsz_item_size = if sample_size == 0 {
            size_of::<u32>() // entry_size
        } else {
            0
        };
        let sample_count = reader.read_u32::<BigEndian>()?;
        let mut sample_sizes = Vec::new();
        if sample_size == 0 {
            if u64::from(sample_count)
                > size
                    .saturating_sub(header_size)
                    .saturating_sub(other_size as u64)
                    / stsz_item_size as u64
            {
                return Err(Error::InvalidData(
                    "stsz sample_count indicates more values than could fit in the box",
                ));
            }
            sample_sizes.reserve(sample_count as usize);
            for _ in 0..sample_count {
                let sample_number = reader.read_u32::<BigEndian>()?;
                sample_sizes.push(sample_number);
            }
        }

        skip_bytes_to(reader, start + size)?;

        Ok(StszBox {
            version,
            flags,
            sample_size,
            sample_count,
            sample_sizes,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for StszBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.sample_size)?;
        writer.write_u32::<BigEndian>(self.sample_count)?;
        if self.sample_size == 0 {
            if self.sample_count != self.sample_sizes.len() as u32 {
                return Err(Error::InvalidData("sample count out of sync"));
            }
            for sample_number in self.sample_sizes.iter() {
                writer.write_u32::<BigEndian>(*sample_number)?;
            }
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
    fn test_stsz_same_size() {
        let src_box = StszBox {
            version: 0,
            flags: 0,
            sample_size: 1165,
            sample_count: 12,
            sample_sizes: vec![],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::StszBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = StszBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_stsz_many_sizes() {
        let src_box = StszBox {
            version: 0,
            flags: 0,
            sample_size: 0,
            sample_count: 9,
            sample_sizes: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::StszBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = StszBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
