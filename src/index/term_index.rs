use bincode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub struct Header {
    pub magic: u64,
    pub doc_num: u64,
}
