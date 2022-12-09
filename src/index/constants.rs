pub const TERM_FILENAME: &str = "index.trm";
pub const FST_FILENAME: &str = "index.fst";
pub const TERM_FILE_MAGIC_NUMBER: u64 = 4294872394;
// TODO: magic number for fst file

pub const BIN_CODE_CONFIG: bincode::config::Configuration<
    bincode::config::LittleEndian,
    bincode::config::Fixint,
    bincode::config::SkipFixedArrayLength,
> = bincode::config::standard()
    .with_little_endian()
    .with_fixed_int_encoding()
    .skip_fixed_array_length();
