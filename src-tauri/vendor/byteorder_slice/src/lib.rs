#![allow(clippy::cast_lossless)]
pub mod option;
pub mod result;

pub use byteorder;
pub use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, NetworkEndian};
