use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub struct Header {
    /// magic number
    pub magic: u64,
    /// total documents number
    pub doc_num: u64,
}
