### Group Name
windrunner

### Group member names and NetIDs
Yuxiang Liu, yl131

### Project Introduction
This is a search engine that supports full-text searches with the goal of quickly searching tens of millions of data.

### Technical Overview
We will use reverse index to achieve this goal.  
The reverse index is composed of FST (Finite State Transducer) and Skip List, and the FST index will be loaded into memory.  

#### Checkpoint 1
completing some basic data structure.

#### Checkpoint 2
finishing index building and querying.

#### Challenges
For a query, we may find a large number of search results, how do we quickly score and sort them?
