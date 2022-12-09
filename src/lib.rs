extern crate core;

pub mod analyzer;
pub mod index;
pub mod query;
mod store;

#[cfg(test)]
mod tests {
    use crate::analyzer::char_filter::CJKDocCharFilter;
    use crate::analyzer::token_filter::StopWordTokenFilter;
    use crate::analyzer::tokenizer::JiebaTokenizer;
    use crate::analyzer::Analyzer;
    use crate::index;
    use crate::index::document::Document;
    use crate::query;
    use fst::automaton::Levenshtein;
    use std::fs;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read};
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

    const TEST_INDEX_DIR: &str = "./test_output/index";
    const STOP_WORD_DICT: &str = "./dict/stop_words.txt";
    const TEST_DOCUMENTS_DIR: &str = "./test_documents";

    fn clear_file() {
        let path = Path::new(TEST_INDEX_DIR);
        fs::remove_dir_all(path).unwrap_or(());
        fs::create_dir_all(path).unwrap();
    }

    #[test]
    fn build_index() {
        clear_file();

        let mut stop_word_dict = File::open(STOP_WORD_DICT).unwrap();
        let analyzer = Analyzer::new(
            CJKDocCharFilter::new(),
            StopWordTokenFilter::new(&mut stop_word_dict).unwrap(),
            JiebaTokenizer::new(),
        );
        let mut index_writer = index::writer::Writer::new(
            analyzer,
            index::writer::config::Config::new(PathBuf::from(TEST_INDEX_DIR)),
        )
        .unwrap();

        for entry in fs::read_dir(TEST_DOCUMENTS_DIR).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let id = u64::from_str(entry.file_name().to_str().unwrap()).unwrap();
            let mut content = String::new();
            File::open(path)
                .unwrap()
                .read_to_string(&mut content)
                .unwrap();
            let document = Document {
                external_id: id,
                content: &content,
            };
            index_writer.add_doc(&document).unwrap();
        }
        index_writer.finish().unwrap();
    }

    #[test]
    fn query() {
        let mut stop_word_dict = File::open(STOP_WORD_DICT).unwrap();
        let analyzer = Analyzer::new(
            CJKDocCharFilter::new(),
            StopWordTokenFilter::new(&mut stop_word_dict).unwrap(),
            JiebaTokenizer::new(),
        );
        let mut query =
            query::Query::new(analyzer, query::Config::new(PathBuf::from(TEST_INDEX_DIR))).unwrap();
        let result = query
            .query(
                "Canada",
                &|w| Levenshtein::new(w, if w.chars().count() > 4 { 1 } else { 0 }).ok(),
                0..3,
            )
            .unwrap();
        for id in result {
            let mut doc_file = PathBuf::from(TEST_DOCUMENTS_DIR);
            doc_file.push(id.to_string());
            let first_line = BufReader::new(File::open(doc_file).unwrap())
                .lines()
                .next()
                .unwrap()
                .unwrap();
            println!("id: {}, first_line: {}", id, first_line);
            println!();
        }
    }
}
