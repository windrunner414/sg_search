use crate::store::PackedU64Array;

/// create a new block when the post number = POSTING_BLOCK_MAX_SIZE
pub const POSTING_BLOCK_MAX_SIZE: u8 = 128;

#[derive(Debug)]
pub struct PostingBlock {
    pub postings: PackedU64Array,
}
