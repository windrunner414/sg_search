use crate::analyzer;
use crate::store;

use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Analyze(analyzer::Error),
    Io(std::io::Error),
    Fst(fst::Error),
    Store(store::Error),
    Incompatible,
    OutOfRange,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Analyze(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
            Error::Fst(ref e) => Some(e),
            Error::Store(ref e) => Some(e),
            Error::Incompatible => None,
            Error::OutOfRange => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Analyze(ref e) => Display::fmt(&e, f),
            Error::Io(ref e) => Display::fmt(&e, f),
            Error::Fst(ref e) => Display::fmt(&e, f),
            Error::Store(ref e) => Display::fmt(&e, f),
            Error::Incompatible => write!(f, "incompatible data file"),
            Error::OutOfRange => write!(f, "out of range"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<analyzer::Error> for Error {
    fn from(e: analyzer::Error) -> Self {
        Error::Analyze(e)
    }
}

impl From<fst::Error> for Error {
    fn from(e: fst::Error) -> Self {
        Error::Fst(e)
    }
}

impl From<store::Error> for Error {
    fn from(e: store::Error) -> Self {
        Error::Store(e)
    }
}
