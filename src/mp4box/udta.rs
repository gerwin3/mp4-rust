use std::io::{Read, Seek};

use crate::mp4box::meta::MetaBox;
use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UdtaBox {
    pub meta: Option<MetaBox>,
}

impl UdtaBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::UdtaBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(meta) = &self.meta {
            size += meta.box_size();
        }
        size
    }
}

impl Mp4Box for UdtaBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn summary(&self) -> Result<String> {
        Ok(String::new())
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for UdtaBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut meta = None;

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "udta box contains a box with a larger size than it",
                ));
            }

            match name {
                BoxType::MetaBox => {
                    meta = Some(MetaBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        skip_bytes_to(reader, start + size)?;

        Ok(UdtaBox { meta })
    }
}

impl<W: Write> WriteBox<&mut W> for UdtaBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(meta) = &self.meta {
            meta.write_box(writer)?;
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
    fn test_udta_empty() {
        let src_box = UdtaBox { meta: None };

        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::UdtaBox);
        assert_eq!(header.size, src_box.box_size());

        let dst_box = UdtaBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(dst_box, src_box);
    }

    #[test]
    fn test_udta() {
        let src_box = UdtaBox {
            meta: Some(MetaBox::default()),
        };

        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::UdtaBox);
        assert_eq!(header.size, src_box.box_size());

        let dst_box = UdtaBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(dst_box, src_box);
    }
}
