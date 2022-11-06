#[derive(Debug)]
pub struct Document<'a> {
    external_id: u64,
    contents: Vec<&'a str>,
}
