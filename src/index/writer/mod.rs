pub mod config;
mod error;

pub use crate::index::writer::error::{Error, Result};

use crate::analyzer::char_filter::CharFilter;
use crate::analyzer::token_filter::TokenFilter;
use crate::analyzer::tokenizer::Tokenizer;
use crate::analyzer::Analyzer;
use crate::index::constants::{
    BIN_CODE_CONFIG, FST_FILENAME, TERM_FILENAME, TERM_FILE_MAGIC_NUMBER,
};
use crate::index::document::Document;
use crate::index::postings::{PostingsBlock, PostingsInfo, POSTING_BLOCK_MAX_SIZE};
use crate::index::term_index;
use crate::index::writer::config::Config;
use crate::store::{skip_list as SkipList, FileStore, WriteableStore};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::BufWriter;

type TermDict = BTreeMap<String, TermData>;

#[derive(Debug)]
struct TermData {
    offset: u64,
    postings: Vec<PostingsInfo>,
    doc_num: u64,
}

impl TermData {
    #[inline]
    fn new() -> Self {
        Self {
            offset: 0,
            postings: Vec::with_capacity(POSTING_BLOCK_MAX_SIZE),
            doc_num: 0,
        }
    }
}

#[derive(Debug)]
pub struct Writer<C, T, I>
where
    C: CharFilter,
    T: TokenFilter,
    I: Tokenizer,
{
    analyzer: Analyzer<C, T, I>,
    // config: Config,
    dict: TermDict,
    doc_num: u64,
    term_file: FileStore,
    fst_file: File,
}

macro_rules! write_posting {
    ($store:expr, $data:expr) => {{
        if !$data.postings.is_empty() {
            let mut skip_list = if $data.offset > 0 {
                SkipList::Writer::new($store, $data.offset)
            } else {
                let s = SkipList::Writer::make_append($store)?;
                $data.offset = s.offset();
                s
            };
            skip_list.push_back(PostingsBlock {
                postings: $data.postings.drain(..).collect(),
            })?;
        }
    }};
}

impl<C, T, I> Writer<C, T, I>
where
    C: CharFilter,
    T: TokenFilter,
    I: Tokenizer,
{
    #[inline]
    pub fn new(analyzer: Analyzer<C, T, I>, config: Config) -> Result<Self> {
        let fst_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config.build_file_path(FST_FILENAME).to_str().unwrap())?;
        let term_file =
            FileStore::open(config.build_file_path(TERM_FILENAME).to_str().unwrap())?;

        let mut w = Writer {
            analyzer,
            // config,
            dict: TermDict::new(),
            doc_num: 0,
            term_file,
            fst_file,
        };
        w.write_header()?;
        Ok(w)
    }

    fn write_header(&mut self) -> Result<()> {
        let header = term_index::Header {
            magic: TERM_FILE_MAGIC_NUMBER,
            doc_num: self.doc_num,
        };
        self.term_file.write(header, 0, BIN_CODE_CONFIG)?;
        Ok(())
    }

    pub fn add_doc(&mut self, doc: &Document) -> Result<()> {
        self.doc_num += 1;

        let (tokens, token_num) = self.analyzer.analyze(doc.content)?;
        for (term, num) in tokens {
            let tf = ((num as f64 / token_num as f64) * u16::MAX as f64) as u16;
            self.add_term(&term, tf, doc, self.doc_num)?;
        }

        Ok(())
    }

    #[inline]
    fn add_term(&mut self, term: &str, tf: u16, doc: &Document, doc_id: u64) -> Result<()> {
        let posting = PostingsInfo {
            id: doc_id,
            external_id: doc.external_id,
            tf,
        };
        match self.dict.get_mut(term) {
            None => {
                let mut d = TermData::new();
                d.postings.push(posting);
                d.doc_num += 1;
                self.dict.insert(term.to_string(), d);
            }
            Some(d) => {
                d.postings.push(posting);
                d.doc_num += 1;
                if d.postings.len() >= POSTING_BLOCK_MAX_SIZE {
                    write_posting!(&mut self.term_file, d);
                }
            }
        }

        Ok(())
    }

    // TODO: store doc num for each term
    pub fn finish(&mut self) -> Result<()> {
        let mut fst_builder = fst::raw::Builder::new(BufWriter::new(&self.fst_file))?;

        for (term, data) in &mut self.dict {
            write_posting!(&mut self.term_file, data);
            fst_builder.insert(term, data.offset)?;
        }

        fst_builder.finish()?;
        self.write_header()?;

        Ok(())
    }
}
