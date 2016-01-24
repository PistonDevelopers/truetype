
use std::fmt;
use byteorder;

/// An Error type.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    Malformed,
    MissingTable,
    HHEAVersionIsNotSupported,
    HEADVersionIsNotSupported,
    MAXPVersionIsNotSupported,
    CMAPEncodingSubtableIsNotSupported,
    CMAPFormatIsNotSupported,
    UnknownLocationFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        f.write_str(self.description())
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Malformed => "malformed data",
            Error::MissingTable => "missing table",
            Error::HHEAVersionIsNotSupported => "hhea version is not supported",
            Error::HEADVersionIsNotSupported => "head version is not supported",
            Error::MAXPVersionIsNotSupported => "maxp version is not supported",
            Error::CMAPEncodingSubtableIsNotSupported => "cmap encoding subtable is not supported",
            Error::CMAPFormatIsNotSupported => "cmap format is not supported",
            Error::UnknownLocationFormat => "unknown index to glyph map format",
        }
    }
}

impl From<byteorder::Error> for Error {
    fn from(_: byteorder::Error) -> Self {
        Error::Malformed
    }
}
