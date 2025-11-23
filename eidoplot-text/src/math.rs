use core::fmt;

mod lex;

/// Position into an input stream
pub type Pos = usize;

/// Byte span into an input stream
/// (first pos, one past last pos)
pub type Span = (Pos, Pos);

#[derive(Debug, Clone)]
pub enum MathError {

}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for MathError {}
