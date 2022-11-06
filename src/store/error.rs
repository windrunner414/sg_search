use crate::analyzer;

use bincode::error::{DecodeError as BinCodeDecodeError, EncodeError as BinCodeEncodeError};
use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Analyze(analyzer::Error),
    IO(std::io::Error),
    FST(fst::Error),
    Encoding(EncodingError),
    Incompatible,
    OutOfRange,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Analyze(ref e) => Display::fmt(&e, f),
            Error::IO(ref e) => Display::fmt(&e, f),
            Error::FST(ref e) => Display::fmt(&e, f),
            Error::Encoding(ref e) => Display::fmt(&e, f),
            Error::Incompatible => write!(f, "incompatible data file"),
            Error::OutOfRange => write!(f, "out of range"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Analyze(ref e) => Some(e),
            Error::IO(ref e) => Some(e),
            Error::FST(ref e) => Some(e),
            Error::Encoding(ref e) => Some(e),
            Error::Incompatible => None,
            Error::OutOfRange => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<analyzer::Error> for Error {
    fn from(e: analyzer::Error) -> Self {
        Error::Analyze(e)
    }
}

impl From<fst::Error> for Error {
    fn from(e: fst::Error) -> Self {
        Error::FST(e)
    }
}

impl From<EncodingError> for Error {
    fn from(e: EncodingError) -> Self {
        Error::Encoding(e)
    }
}

impl From<BinCodeEncodeError> for Error {
    fn from(e: BinCodeEncodeError) -> Self {
        Error::Encoding(e.into())
    }
}

impl From<BinCodeDecodeError> for Error {
    fn from(e: BinCodeDecodeError) -> Self {
        Error::Encoding(e.into())
    }
}

#[derive(Debug)]
pub enum EncodingError {
    Encode(BinCodeEncodeError),
    Decode(BinCodeDecodeError),
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            EncodingError::Encode(ref e) => Display::fmt(&e, f),
            EncodingError::Decode(ref e) => Display::fmt(&e, f),
        }
    }
}

impl std::error::Error for EncodingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            EncodingError::Encode(ref e) => Some(e),
            EncodingError::Decode(ref e) => Some(e),
        }
    }
}

impl From<BinCodeEncodeError> for EncodingError {
    fn from(e: BinCodeEncodeError) -> Self {
        EncodingError::Encode(e)
    }
}

impl From<BinCodeDecodeError> for EncodingError {
    fn from(e: BinCodeDecodeError) -> Self {
        EncodingError::Decode(e)
    }
}
