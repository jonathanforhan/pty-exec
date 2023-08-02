use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct PtyError(pub String);

impl fmt::Display for PtyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pseudo Terminal Error: {}", self.0)
    }
}

impl Error for PtyError {}
