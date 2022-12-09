#[derive(Debug)]
pub struct Document<'a> {
    /// External document ID, the search result will be it
    pub external_id: u64,
    /// document content
    pub content: &'a str,
}
