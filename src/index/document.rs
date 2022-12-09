#[derive(Debug)]
pub struct Document<'a> {
    pub external_id: u64,
    pub content: &'a str,
}
