use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};
use std::mem::size_of;

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StssBox {
    pub version: u8,
    pub flags: u32,
    pub entries: Vec<u32>,
}

impl StssBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::StssBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + (4 * self.entries.len() as u64)
    }
}

impl Mp4Box for StssBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn summary(&self) -> Result<String> {
        let s = format!("entries={}", self.entries.len());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for StssBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let header_size = HEADER_SIZE + HEADER_EXT_SIZE;
        let other_size = size_of::<u32>(); // entry_count
        let entry_size = size_of::<u32>(); // sample_number
        let entry_count = reader.read_u32::<BigEndian>()?;
        if u64::from(entry_count)
            > size
                .saturating_sub(header_size)
                .saturating_sub(other_size as u64)
                / entry_size as u64
        {
            return Err(Error::InvalidData(
                "stss entry_count indicates more entries than could fit in the box",
            ));
        }
        let mut entries = Vec::with_capacity(entry_count as usize);
        for _i in 0..entry_count {
            let sample_number = reader.read_u32::<BigEndian>()?;
            entries.push(sample_number);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(StssBox {
            version,
            flags,
            entries,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for StssBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.entries.len() as u32)?;
        for sample_number in self.entries.iter() {
            writer.write_u32::<BigEndian>(*sample_number)?;
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
    fn test_stss() {
        let src_box = StssBox {
            version: 0,
            flags: 0,
            entries: vec![1, 61, 121, 181, 241, 301, 361, 421, 481],
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::StssBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = StssBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
