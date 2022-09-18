use crate::store::PackedIdArray;

/// create a new block when the post number = POSTING_BLOCK_MAX_SIZE
pub const POSTING_BLOCK_MAX_SIZE: u8 = 128;

pub struct PostingBlock {
    pub postings: PackedIdArray,
}
