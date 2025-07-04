use std::io::{Read, Seek, Write};

use crate::mp4box::*;
use crate::mp4box::{tfdt::TfdtBox, tfhd::TfhdBox, trun::TrunBox};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrafBox {
    pub tfhd: TfhdBox,
    pub tfdt: Option<TfdtBox>,
    pub trun: Option<TrunBox>,
}

impl TrafBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TrafBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.tfhd.box_size();
        if let Some(ref tfdt) = self.tfdt {
            size += tfdt.box_size();
        }
        if let Some(ref trun) = self.trun {
            size += trun.box_size();
        }
        size
    }
}

impl Mp4Box for TrafBox {
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

impl<R: Read + Seek> ReadBox<&mut R> for TrafBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tfhd = None;
        let mut tfdt = None;
        let mut trun = None;

        let mut current = reader.stream_position()?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;
            if s > size {
                return Err(Error::InvalidData(
                    "traf box contains a box with a larger size than it",
                ));
            }

            match name {
                BoxType::TfhdBox => {
                    tfhd = Some(TfhdBox::read_box(reader, s)?);
                }
                BoxType::TfdtBox => {
                    tfdt = Some(TfdtBox::read_box(reader, s)?);
                }
                BoxType::TrunBox => {
                    trun = Some(TrunBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.stream_position()?;
        }

        if tfhd.is_none() {
            return Err(Error::BoxNotFound(BoxType::TfhdBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(TrafBox {
            tfhd: tfhd.unwrap(),
            tfdt,
            trun,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TrafBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.tfhd.write_box(writer)?;
        if let Some(ref tfdt) = self.tfdt {
            tfdt.write_box(writer)?;
        }
        if let Some(ref trun) = self.trun {
            trun.write_box(writer)?;
        }

        Ok(size)
    }
}
