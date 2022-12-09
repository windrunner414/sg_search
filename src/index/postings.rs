use bincode::{Decode, Encode};

/// create a new block when the post number = POSTING_BLOCK_MAX_SIZE
pub const POSTING_BLOCK_MAX_SIZE: usize = 128;

#[derive(Debug, Encode, Decode)]
pub struct PostingsInfo {
    /// Self-incrementing ID
    pub id: u64,
    /// See document.rs
    pub external_id: u64,
    /// term frequency
    pub tf: u16,
}

#[derive(Debug, Encode, Decode)]
pub struct PostingsBlock {
    pub postings: Vec<PostingsInfo>,
}
