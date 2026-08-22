//! Microsoft Wubi `.lex` binary decoding and canonical encoding.

mod decode;
mod encode;

pub use decode::decode;
pub use encode::encode;

pub(super) const MAGIC: &[u8; 8] = b"imscwubi";
pub(super) const HEADER_SIZE: usize = 64;
pub(super) const ALPHA_COUNT: usize = 26;
pub(super) const INDEX_SIZE: usize = ALPHA_COUNT * 4;
pub(super) const CANONICAL_INDEX_OFFSET: usize = HEADER_SIZE;
pub(super) const CANONICAL_TABLE_OFFSET: usize = HEADER_SIZE + INDEX_SIZE;

pub(super) const INDEX_OFFSET_FIELD_OFFSET: usize = 12;
pub(super) const TABLE_OFFSET_FIELD_OFFSET: usize = 16;
pub(super) const FILE_SIZE_FIELD_OFFSET: usize = 20;

pub(super) fn offset_u64(offset: usize) -> u64 {
    offset as u64
}
