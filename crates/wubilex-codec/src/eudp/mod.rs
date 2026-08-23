//! Microsoft Wubi EUDP binary decoding and canonical encoding.

mod decode;
mod encode;

pub use decode::decode;
pub use encode::encode;

pub(super) const MAGIC: &[u8; 8] = b"mschxudp";
pub(super) const HEADER_SIZE: usize = 64;
pub(super) const RECORD_HEADER_SIZE: usize = 16;

pub(super) const PHRASE_OFFSET_START_FIELD_OFFSET: usize = 16;
pub(super) const PHRASE_START_FIELD_OFFSET: usize = 20;
pub(super) const PHRASE_END_FIELD_OFFSET: usize = 24;
pub(super) const COUNT_FIELD_OFFSET: usize = 28;

pub(super) fn offset_u64(offset: usize) -> u64 {
    offset as u64
}
