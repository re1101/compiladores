use std::{
    fmt,
    error::Error};

#[derive(Debug)]
pub enum CompilerError {
    OpenError(std::io::Error),
    RegexError(regex::Error),
    InvalidSyntax(u16),
    InvalidToken(String, u16),
    NoSuchVar(String, u16),
    MissmatchedTypes(String, u16),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::OpenError(err) => write!(f, "Captured Underlying Error: {}", err),
            CompilerError::RegexError(err) => write!(f, "Captured Underlying Regex Error: {}", err),
            CompilerError::InvalidSyntax(line) => write!(f, "Invalid syntax at line: {}", line),
            CompilerError::InvalidToken(token, line) => {
                write!(f, "Invalid token \"{}\" at line: {}", token, line)
            }
            CompilerError::NoSuchVar(token, line) => {
                write!(f, "Invalid Var \"{}\" at line: {}", token, line)
            }
            CompilerError::MissmatchedTypes(token, line) => {
                write!(f, "Missmatched Var \"{}\" at line: {}", token, line)
            }
        }
    }
}

impl Error for CompilerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CompilerError::OpenError(err) => Some(err),
            _ => None,
        }
    }
}