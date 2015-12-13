
use std::fmt;

/// An Error type.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Malformed,
    MissingTable,
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
        }
    }
}
