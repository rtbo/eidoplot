use std::iter::FusedIterator;

use super::{MathError, Pos, Span};
use crate::input::Cursor;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    CtlWord(String),
    CtlSym(char),
    Power,
    Indice,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    ParOpen,
    ParClose,
    /// Binary operator (those that take space on both sides)
    BinOp(char),
    /// Relational operator (eg. <, >, =, :)
    RelOp(char),
    /// Punctuation marks
    PuncMark(char),
    /// Ordinary letter
    Letter(char),
    /// Ordinary symbol (number or symbol, including '/')
    Symbol(char),
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
        while let Some(c) = self.cursor.first() {
            if c.is_whitespace() {
                self.cursor.next();
            } else {
                break;
            }
        }

        let Some(c) = self.cursor.next() else {
            return Ok(None);
        };

        match c {
            '\\' => self.parse_ctl_seq(pos),
            '^' => Ok(Some(TokenKind::Power)),
            '_' => Ok(Some(TokenKind::Indice)),
            '{' => Ok(Some(TokenKind::BraceOpen)),
            '}' => Ok(Some(TokenKind::BraceClose)),
            '[' => Ok(Some(TokenKind::BracketOpen)),
            ']' => Ok(Some(TokenKind::BracketClose)),
            '(' => Ok(Some(TokenKind::ParOpen)),
            ')' => Ok(Some(TokenKind::ParClose)),
            '+' | '-' | '*' => Ok(Some(TokenKind::BinOp(c))),
            '=' | '<' | '>' | ':' => Ok(Some(TokenKind::RelOp(c))),
            ',' | ';' => Ok(Some(TokenKind::PuncMark(c))),
            'A'..='Z' | 'a'..='z' => Ok(Some(TokenKind::Letter(c))),
            '0'..='9' | '!' | '?' | '.' | '|' | '/' | '`' | '@' | '"' => {
                Ok(Some(TokenKind::Symbol(c)))
            }
            ' ' => self.next_token_kind(pos),
            _ => todo!(),
        }
    }

    fn parse_ctl_seq(&mut self, pos: Pos) -> Result<Option<TokenKind>, MathError> {
        let Some(c) = self.cursor.next() else {
            return Err(MathError::UnexpectedEndOfStr(pos));
        };
        if c.is_ascii_alphabetic() {
            let mut ctl_word = String::new();
            ctl_word.push(c);
            while let Some(c) = self.cursor.first() {
                if c.is_ascii_alphabetic() {
                    self.cursor.next();
                    ctl_word.push(c);
                } else {
                    break;
                }
            }
            Ok(Some(TokenKind::CtlWord(ctl_word)))
        } else {
            Ok(Some(TokenKind::CtlSym(c)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_kinds(expr: &str) -> Vec<TokenKind> {
        tokenize(expr.chars()).map(|t| t.unwrap().1).collect()
    }

    #[test]
    fn test_ctl_seq() {
        let toks = token_kinds("\\word\\other2\\yetother \\,");
        assert_eq!(
            &toks,
            &[
                TokenKind::CtlWord("word".to_string()),
                TokenKind::CtlWord("other".to_string()),
                TokenKind::Symbol('2'),
                TokenKind::CtlWord("yetother".to_string()),
                TokenKind::CtlSym(','),
            ]
        );
    }

    #[test]
    fn test_power_indice() {
        let toks = token_kinds("x_0^2");
        assert_eq!(
            &toks,
            &[
                TokenKind::Letter('x'),
                TokenKind::Indice,
                TokenKind::Symbol('0'),
                TokenKind::Power,
                TokenKind::Symbol('2'),
            ]
        );
    }

    #[test]
    fn test_binop() {
        let toks = token_kinds("a +b");
        assert_eq!(
            &toks,
            &[
                TokenKind::Letter('a'),
                TokenKind::BinOp('+'),
                TokenKind::Letter('b')
            ]
        );
    }

    #[test]
    fn test_complex_expr() {
        // Katex demo expression
        let expr = r#"\relax f(x) = \int_{-\infty}^\infty
                            \hat f(\xi)\,e^{2 \pi i \xi x}\,d\xi"#;
        let toks = token_kinds(expr);
        assert_eq!(
            &toks,
            &[
                TokenKind::CtlWord("relax".to_string()),
                TokenKind::Letter('f'),
                TokenKind::ParOpen,
                TokenKind::Letter('x'),
                TokenKind::ParClose,
                TokenKind::RelOp('='),
                TokenKind::CtlWord("int".to_string()),
                TokenKind::Indice,
                TokenKind::BraceOpen,
                TokenKind::BinOp('-'),
                TokenKind::CtlWord("infty".to_string()),
                TokenKind::BraceClose,
                TokenKind::Power,
                TokenKind::CtlWord("infty".to_string()),
                TokenKind::CtlWord("hat".to_string()),
                TokenKind::Letter('f'),
                TokenKind::ParOpen,
                TokenKind::CtlWord("xi".to_string()),
                TokenKind::ParClose,
                TokenKind::CtlSym(','),
                TokenKind::Letter('e'),
                TokenKind::Power,
                TokenKind::BraceOpen,
                TokenKind::Symbol('2'),
                TokenKind::CtlWord("pi".to_string()),
                TokenKind::Letter('i'),
                TokenKind::CtlWord("xi".to_string()),
                TokenKind::Letter('x'),
                TokenKind::BraceClose,
                TokenKind::CtlSym(','),
                TokenKind::Letter('d'),
                TokenKind::CtlWord("xi".to_string()),
            ]
        )
    }
}
