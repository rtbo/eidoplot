use core::fmt;

mod ast;
mod lex;
mod parse;

pub use parse::parse;

/// Position into an input stream
pub type Pos = usize;

/// Byte span into an input stream
/// (first pos, one past last pos)
pub type Span = (Pos, Pos);

#[derive(Debug, Clone)]
pub enum MathError {
    UnexpectedEndOfStr(Pos),
    UnknownToken(Span, char),
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MathError::UnexpectedEndOfStr(..) => write!(f, "unexpected end of string"),
            MathError::UnknownToken(_, c) => write!(f, "unknown token: '{}'", c),
        }
    }
}

impl std::error::Error for MathError {}

pub type MathResult<T> = Result<T, MathError>;
