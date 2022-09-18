/// .nrm, store norms of each documents by doc id
/// fixed-size (u8 * field count), order by field name in dictionary order
pub struct Norms {
    pub norms: Vec<u8>
}