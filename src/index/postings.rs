use bincode::{Decode, Encode};

/// create a new block when the post number = POSTING_BLOCK_MAX_SIZE
pub const POSTING_BLOCK_MAX_SIZE: usize = 128;

#[derive(Debug, Encode, Decode)]
pub struct PostingsInfo {
    pub id: u64,
    pub external_id: u64,
    pub tf: u16,
}

#[derive(Debug, Encode, Decode)]
pub struct PostingsBlock {
    pub postings: Vec<PostingsInfo>,
}
