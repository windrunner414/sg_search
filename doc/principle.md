### search:
- split the search text into terms
- find the offset of posting list for each term from FST
- find the intersection of each posting list
- calculate the similarity with the searched text and sort

### build index
- add a document, store the norms
- split the texts of each field into terms
- add this document to the posting list of each term
- after finishing adding all documents, build FST