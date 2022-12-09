`git clone https://github.com/windrunner414/sg_search.git`

Since this project is a library, I provide some test cases to easily run it.  
Test documents is in the `test_documents` directory and the file name is the document id.

#### Build index
`cargo test --color=always --package sg_search --lib tests::build_index -- --show-output`
#### Search
Please build index before searching.  
You can modify the search keyword in lib.rs: `query.query("Your Keyword"...)"`  
`cargo test --color=always --package sg_search --lib tests::query -- --show-output`