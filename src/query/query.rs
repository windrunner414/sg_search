use crate::analyzer::char_filter::CharFilter;
use crate::analyzer::token_filter::TokenFilter;
use crate::analyzer::tokenizer::Tokenizer;
use crate::analyzer::Analyzer;
use crate::index::constants::{
    BIN_CODE_CONFIG, FST_FILENAME, TERM_FILENAME, TERM_FILE_MAGIC_NUMBER,
};
use crate::index::postings::PostingsBlock;
use crate::index::term_index;
use crate::query::{Error, Result};
use crate::store::skip_list;
use crate::store::{FileStore, ReadableStore};
use fst::IntoStreamer;
use memmap2::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::fs::File;
use std::ops::Range;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    /// the directory of index files
    pub index_dir: PathBuf,
}

impl Config {
    pub fn new(index_dir: PathBuf) -> Self {
        Self { index_dir }
    }

    /// return the full path of the index file (index_dir + filename)
    pub fn build_file_path(&self, filename: &str) -> PathBuf {
        let mut buf = self.index_dir.clone();
        buf.push(filename);
        buf
    }
}

#[derive(Debug)]
pub struct Query<C, T, I>
where
    C: CharFilter,
    T: TokenFilter,
    I: Tokenizer,
{
    analyzer: Analyzer<C, T, I>,
    // config: Config,
    fst: fst::Map<Mmap>,
    terms: FileStore,
    // doc_num: u64,
    // term_priority_calculator: TfIdfTermPriorityCalculator,
}

impl<C, T, I> Query<C, T, I>
where
    C: CharFilter,
    T: TokenFilter,
    I: Tokenizer,
{
    pub fn new(analyzer: Analyzer<C, T, I>, config: Config) -> Result<Self> {
        let fst_file = File::open(config.build_file_path(FST_FILENAME).to_str().unwrap())?;
        let mmap = unsafe { MmapOptions::new().map(&fst_file)? };
        let fst = fst::Map::new(mmap)?;

        let terms = FileStore::open(config.build_file_path(TERM_FILENAME).to_str().unwrap())?;
        let header: term_index::Header = terms.read(0, BIN_CODE_CONFIG)?;
        check_term_index(&header)?;

        // let doc_num = header.doc_num;
        //
        // let term_priority_calculator = TfIdfTermPriorityCalculator::new(doc_num);

        let query = Query {
            analyzer,
            // config,
            fst,
            terms,
            // doc_num,
            // term_priority_calculator,
        };

        Ok(query)
    }

    /// return all posting in this skip list
    #[inline]
    fn find_posting_list(&mut self, offset: u64) -> Result<PostingsBlock> {
        // TODO: do not read all postings
        let mut list = PostingsBlock { postings: vec![] };
        let skip_list: skip_list::Reader<FileStore, PostingsBlock> =
            skip_list::Reader::new(&self.terms, offset);
        for block in skip_list.iter()? {
            let mut block = block?;
            list.postings.append(&mut block.postings);
        }
        Ok(list)
    }

    /// return all posting that contains the specific pattern
    #[inline]
    fn query_term_postings<A: fst::Automaton>(
        &mut self,
        word: &str,
        aut_builder: &impl Fn(&str) -> Option<A>,
    ) -> Result<Option<PostingsBlock>> {
        // find the offsets in term index of `word`
        let term_offsets = match aut_builder(word) {
            None => self
                .fst
                .get(word)
                .map_or_else(Vec::new, |i| vec![(word.to_string(), i)]),
            Some(aut) => self.fst.search(aut).into_stream().into_str_vec()?,
        };

        // if we find the exact `word` in fst, return it
        // otherwise, use another word that is close to it
        let mut other: Option<(String, u64)> = None;
        for index in term_offsets.into_iter() {
            if index.0.as_str() == word {
                return Ok(Some(self.find_posting_list(index.1)?));
            } else {
                other = Some(index);
            }
        }

        other.map_or_else(
            || Ok(None),
            |index| Ok(Some(self.find_posting_list(index.1)?)),
        )
    }

    /// Score and sort all search results and return the results in `range`
    pub fn query<A: fst::Automaton>(
        &mut self,
        sentence: &str,
        aut_builder: &impl Fn(&str) -> Option<A>,
        range: Range<usize>,
    ) -> Result<Vec<u64>> {
        let (query_terms, _) = self.analyzer.analyze(sentence)?;
        let mut postings = HashMap::<u64, u64>::new();

        for term in query_terms.keys() {
            match self.query_term_postings(term.as_str(), aut_builder)? {
                None => (),
                Some(v) => {
                    for posting in v.postings {
                        postings
                            .entry(posting.external_id)
                            .and_modify(|score| *score += posting.tf as u64)
                            .or_insert(posting.tf as u64);
                    }
                }
            }
        }

        let mut postings_vec: Vec<(u64, u64)> = postings.into_iter().collect();
        postings_vec.sort_by_key(|k| k.1);

        Ok(postings_vec
            .iter()
            .skip(range.start)
            .take(range.end - range.start)
            .map(|p| p.0)
            .collect())
    }
}

/// check if this is a valid term index file
fn check_term_index(header: &term_index::Header) -> Result<()> {
    if header.magic != TERM_FILE_MAGIC_NUMBER {
        Err(Error::Incompatible)
    } else {
        Ok(())
    }
}
