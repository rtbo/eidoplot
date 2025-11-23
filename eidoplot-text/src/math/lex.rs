use std::iter::FusedIterator;

use super::{MathError, Pos, Span};
use crate::input::Cursor;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    CtlSeq(String),
    CtlSym(char),
    Hat,
    Underscore,
    BraceOpen,
    BraceClose,
    BraketOpen,
    BraketClose,
    /// Ordinary letter
    Letter(char),
    /// Ordinary symbol (number or symbol)
    Symbol(char),
    Plus,
    Hyphen,
    Asterisk,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token(pub Span, pub TokenKind);

pub fn tokenize<I>(chars: I) -> Tokenizer<I::IntoIter>
where
    I: IntoIterator<Item = char>,
{
    Tokenizer::new(Cursor::new(chars.into_iter()))
}

#[derive(Debug, Clone)]
pub struct Tokenizer<I> {
    cursor: Cursor<I>,
}

impl<I> Tokenizer<I> {
    pub fn new(cursor: Cursor<I>) -> Tokenizer<I> {
        Tokenizer { cursor }
    }
}

impl<I> Iterator for Tokenizer<I>
where
    I: Iterator<Item = char> + Clone,
{
    type Item = Result<Token, MathError>;

    fn next(&mut self) -> Option<Result<Token, MathError>> {
        let pos = self.cursor.pos();
        let kind = match self.next_token_kind(pos) {
            Ok(Some(kind)) => kind,
            Ok(None) => return None,
            Err(err) => return Some(Err(err)),
        };
        let end = self.cursor.pos();
        Some(Ok(Token((pos, end), kind)))
    }
}

impl<I> FusedIterator for Tokenizer<I> where I: FusedIterator<Item = char> + Clone {}

impl<I> Tokenizer<I>
where
    I: Iterator<Item = char> + Clone,
{
    fn next_token_kind(&mut self, pos: Pos) -> Result<Option<TokenKind>, MathError> {
        let Some(c) = self.cursor.next() else {
            return Ok(None);
        };

        match c {
            '\\' => {
                let ctl_seq = self.parse_ctl_seq(pos);
                ctl_seq
            }
            '^' => Ok(Some(TokenKind::Hat)),
            '_' => Ok(Some(TokenKind::Underscore)),
            _ => todo!(),
        }
    }

    fn parse_ctl_seq(&mut self, pos: Pos) -> Result<Option<TokenKind>, MathError> {
        todo!()
    }
}
