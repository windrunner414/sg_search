use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    /// the directory of index files
    pub output_dir: PathBuf,
}

impl Config {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    pub fn build_file_path(&self, filename: &str) -> PathBuf {
        let mut buf = self.output_dir.clone();
        buf.push(filename);
        buf
    }
}
