use std::io::{Read, Seek, Write};

use crate::mp4box::elst::ElstBox;
use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EdtsBox {
    pub elst: Option<ElstBox>,
}

impl EdtsBox {
    pub(crate) fn new() -> EdtsBox {
        Default::default()
    }

    pub fn get_type(&self) -> BoxType {
        BoxType::EdtsBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref elst) = self.elst {
            size += elst.box_size();
        }
        size
    }
}

impl Mp4Box for EdtsBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn summary(&self) -> Result<String> {
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for EdtsBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut edts = EdtsBox::new();

        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;
        if s > size {
            return Err(Error::InvalidData(
                "edts box contains a box with a larger size than it",
            ));
        }

        if let BoxType::ElstBox = name {
            let elst = ElstBox::read_box(reader, s)?;
            edts.elst = Some(elst);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(edts)
    }
}

impl<W: Write> WriteBox<&mut W> for EdtsBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        if let Some(ref elst) = self.elst {
            elst.write_box(writer)?;
        }

        Ok(size)
    }
}
