use crate::mp4box::BoxType;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    InvalidData(&'static str),
    BoxNotFound(BoxType),
    Box2NotFound(BoxType, BoxType),
    TrakNotFound(u32),
    BoxInTrakNotFound(u32, BoxType),
    BoxInTrafNotFound(u32, BoxType),
    BoxInStblNotFound(u32, BoxType),
    EntryInStblNotFound(u32, BoxType, u32),
    EntryInTrunNotFound(u32, BoxType, u32),
    UnsupportedBoxVersion(BoxType, u8),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "{}", err),
            Error::InvalidData(msg) => write!(f, "{}", msg),
            Error::BoxNotFound(box_type) => write!(f, "{} not found", box_type),
            Error::Box2NotFound(box_type1, box_type2) => {
                write!(f, "{} and {} not found", box_type1, box_type2)
            }
            Error::TrakNotFound(index) => write!(f, "trak[{}] not found", index),
            Error::BoxInTrakNotFound(index, box_type) => {
                write!(f, "trak[{}].{} not found", index, box_type)
            }
            Error::BoxInTrafNotFound(index, box_type) => {
                write!(f, "traf[{}].{} not found", index, box_type)
            }
            Error::BoxInStblNotFound(index, box_type) => {
                write!(f, "trak[{}].stbl.{} not found", index, box_type)
            }
            Error::EntryInStblNotFound(index, box_type, entry) => write!(
                f,
                "trak[{}].stbl.{}.entry[{}] not found",
                index, box_type, entry
            ),
            Error::EntryInTrunNotFound(index, box_type, entry) => write!(
                f,
                "traf[{}].trun.{}.entry[{}] not found",
                index, box_type, entry
            ),
            Error::UnsupportedBoxVersion(box_type, version) => {
                write!(f, "{} version {} is not supported", box_type, version)
            }
        }
    }
}

impl std::error::Error for Error {}
