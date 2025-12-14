use std::fmt;
use std::iter::FusedIterator;

use crate::input::{Cursor, Pos};

/// Byte span into an input stream
/// (first pos, one past last pos)
pub type Span = (Pos, Pos);

/// A lexical error
#[derive(Debug, Clone)]
pub enum Error {
    UnexpectedChar {
        pos: Pos,
        expected: char,
        found: char,
    },
    UnexpectedEndOfFile(Pos),
    UnterminatedString {
        span: Span,
        help: String,
    },
    InvalidEscSequence(Span, char),
    InvalidNumber(Span, String),
    InvalidKebabIdent(Span, String),
    InvalidPascalIdent(Span, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedChar {
                expected, found, ..
            } => write!(
                f,
                "Unexpected character: expected '{}', found '{}'",
                expected, found
            ),
            Error::UnexpectedEndOfFile(..) => write!(f, "Unexpected end of file"),
            Error::UnterminatedString { help, .. } => {
                write!(f, "Unterminated string. {}", help)
            }
            Error::InvalidEscSequence(_, c) => {
                write!(f, "Invalid escape sequence: \\{}", c)
            }
            Error::InvalidNumber(_, s) => write!(f, "Invalid number: {}", s),
            Error::InvalidKebabIdent(_, s) => {
                write!(f, "Invalid kebab-case identifier {}", s)
            }
            Error::InvalidPascalIdent(_, s) => write!(f, "Invalid pascal-case identifier {}", s,),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    KebabCaseIdent(String),
    PascalCaseIdent(String),
    OpenPar,
    ClosePar,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Colon,
    Comma,
    StrLit(String),
    IntLit(i64),
    FloatLit(f64),
    Space,
    Comment,
    Eol,
}

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
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Result<Token>> {
        let pos = self.cursor.pos();
        let kind = match self.next_token_kind(pos) {
            Ok(Some(kind)) => kind,
            Ok(None) => return None,
            Err(err) => return Some(Err(err)),
        };
        let end = self.cursor.pos();
        Some(Ok(Token {
            kind,
            span: (pos, end),
        }))
    }
}

impl<I> FusedIterator for Tokenizer<I> where I: FusedIterator<Item = char> + Clone {}

const STR_BREAK_HELP: &str = concat!(
    "To include a newline in a string, use \\n. ",
    "To breakdown long strings over multiple lines, concatenate them."
);

impl<I> Tokenizer<I>
where
    I: Iterator<Item = char> + Clone,
{
    fn next_char(&mut self) -> Result<(Pos, char)> {
        let pos = self.cursor.pos();
        let Some(c) = self.cursor.next() else {
            return Err(Error::UnexpectedEndOfFile(pos));
        };
        Ok((pos, c))
    }

    fn expect_next(&mut self, c: char) -> Result<()> {
        let (pos, next) = self.next_char()?;
        if next != c {
            Err(Error::UnexpectedChar {
                pos,
                expected: c,
                found: next,
            })
        } else {
            Ok(())
        }
    }

    fn next_token_kind(&mut self, start_pos: Pos) -> Result<Option<TokenKind>> {
        let Some(c) = self.cursor.next() else {
            return Ok(None);
        };
        match c {
            '\n' => Ok(Some(TokenKind::Eol)),
            '\r' => {
                self.expect_next('\n')?;
                Ok(Some(TokenKind::Eol))
            }
            '(' => Ok(Some(TokenKind::OpenPar)),
            ')' => Ok(Some(TokenKind::ClosePar)),
            '[' => Ok(Some(TokenKind::OpenBracket)),
            ']' => Ok(Some(TokenKind::CloseBracket)),
            '{' => Ok(Some(TokenKind::OpenBrace)),
            '}' => Ok(Some(TokenKind::CloseBrace)),
            ':' => Ok(Some(TokenKind::Colon)),
            ',' => Ok(Some(TokenKind::Comma)),
            '"' => {
                let buf = self.parse_string(start_pos)?;
                Ok(Some(TokenKind::StrLit(buf)))
            }
            '-' | '+' | '0'..='9' => {
                let kind = self.parse_number(start_pos, c)?;
                Ok(Some(kind))
            }
            'a'..='z' => {
                let buf = self.parse_kebab_case_ident(start_pos, c)?;
                Ok(Some(TokenKind::KebabCaseIdent(buf)))
            }
            'A'..='Z' => {
                let buf = self.parse_pascal_case_ident(start_pos, c)?;
                Ok(Some(TokenKind::PascalCaseIdent(buf)))
            }
            '/' => {
                self.expect_next('/')?;
                loop {
                    match self.cursor.next() {
                        None => break,
                        Some('\n') => break,
                        Some('\r') => {
                            self.expect_next('\n')?;
                            break;
                        }
                        Some(_) => (),
                    }
                }
                Ok(Some(TokenKind::Comment))
            }
            c if c.is_ascii_whitespace() => {
                loop {
                    let c = self.cursor.first();
                    match c {
                        Some(c) if c.is_ascii_whitespace() => {
                            self.cursor.next();
                        }
                        _ => break,
                    }
                }
                Ok(Some(TokenKind::Space))
            }
            _ => Ok(None),
        }
    }

    fn parse_esc_sequence(&mut self, start_pos: Pos) -> Result<char> {
        let Some(c) = self.cursor.next() else {
            return Err(Error::UnexpectedEndOfFile(start_pos));
        };
        match c {
            '\\' => Ok('\\'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            _ => Err(Error::InvalidEscSequence((start_pos, self.cursor.pos()), c)),
        }
    }

    fn parse_string(&mut self, start_pos: Pos) -> Result<String> {
        let mut buf = String::new();
        loop {
            let pos = self.cursor.pos();
            match self.cursor.next() {
                None => return Err(Error::UnexpectedEndOfFile(pos)),
                Some('"') => break,
                Some('\n') => {
                    return Err(Error::UnterminatedString {
                        span: (start_pos, self.cursor.pos()),
                        help: STR_BREAK_HELP.to_string(),
                    });
                }
                Some('\r') => {
                    self.expect_next('\n')?;
                    return Err(Error::UnterminatedString {
                        span: (start_pos, self.cursor.pos()),
                        help: STR_BREAK_HELP.to_string(),
                    });
                }
                Some('\\') => {
                    buf.push(self.parse_esc_sequence(pos)?);
                }
                Some(c) => buf.push(c),
            }
        }
        Ok(buf)
    }

    fn parse_number(&mut self, start_pos: Pos, first: char) -> Result<TokenKind> {
        let mut s = String::from(first);
        let mut was_e = false;
        loop {
            let c = self.cursor.first();
            match c {
                Some(c @ ('0'..='9' | '.')) => {
                    self.cursor.next();
                    s.push(c);
                    was_e = false;
                }
                Some(c @ ('e' | 'E')) => {
                    self.cursor.next();
                    s.push(c);
                    was_e = true;
                }
                Some(c @ ('+' | '-')) if was_e => {
                    self.cursor.next();
                    s.push(c);
                    was_e = false;
                }
                _ => break,
            }
        }

        match s.parse::<i64>() {
            Ok(n) => return Ok(TokenKind::IntLit(n)),
            _ => {}
        }

        match s.parse::<f64>() {
            Ok(n) => return Ok(TokenKind::FloatLit(n)),
            _ => {}
        }

        Err(Error::InvalidNumber((start_pos, self.cursor.pos()), s))
    }

    fn parse_kebab_case_ident(&mut self, start_pos: Pos, first: char) -> Result<String> {
        let mut buf = String::from(first);
        let mut last_was_hyphen = false;
        let mut invalid = false;
        loop {
            let c = self.cursor.first();
            match c {
                Some(c @ ('a'..='z')) => {
                    self.cursor.next();
                    buf.push(c);
                    last_was_hyphen = false;
                }
                Some(c @ ('0'..='9')) => {
                    self.cursor.next();
                    buf.push(c);
                    last_was_hyphen = false;
                }
                Some(c @ ('A'..='Z')) => {
                    self.cursor.next();
                    buf.push(c);
                    invalid = true;
                    last_was_hyphen = false;
                }
                Some(c @ '-') => {
                    if last_was_hyphen {
                        invalid = true;
                    }
                    self.cursor.next();
                    buf.push(c);
                    last_was_hyphen = true;
                }
                Some(c @ '_') => {
                    invalid = true;
                    self.cursor.next();
                    buf.push(c);
                    last_was_hyphen = false;
                }
                _ => break,
            }
        }
        if invalid {
            return Err(Error::InvalidKebabIdent(
                (start_pos, self.cursor.pos()),
                buf,
            ));
        }
        Ok(buf)
    }

    fn parse_pascal_case_ident(&mut self, start_pos: Pos, first: char) -> Result<String> {
        let mut buf = String::from(first);
        let mut invalid = false;
        loop {
            let c = self.cursor.first();
            match c {
                Some(c @ ('a'..='z')) => {
                    self.cursor.next();
                    buf.push(c);
                }
                Some(c @ ('A'..='Z')) => {
                    self.cursor.next();
                    buf.push(c);
                }
                Some(c @ ('0'..='9')) => {
                    self.cursor.next();
                    buf.push(c);
                }
                Some(c @ ('-' | '_')) => {
                    invalid = true;
                    self.cursor.next();
                    buf.push(c);
                }
                _ => break,
            }
        }
        if invalid {
            return Err(Error::InvalidPascalIdent(
                (start_pos, self.cursor.pos()),
                buf,
            ));
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize_str(s: &str) -> Vec<TokenKind> {
        tokenize(s.chars()).map(|r| r.unwrap().kind).collect()
    }

    #[test]
    fn test_kebab_case_ident() {
        let toks = tokenize_str("foo-bar baz42");
        assert_eq!(
            toks,
            vec![
                TokenKind::KebabCaseIdent("foo-bar".into()),
                TokenKind::Space,
                TokenKind::KebabCaseIdent("baz42".into())
            ]
        );
    }

    #[test]
    fn test_pascal_case_ident() {
        let toks = tokenize_str("Foo Bar42");
        assert_eq!(
            toks,
            vec![
                TokenKind::PascalCaseIdent("Foo".into()),
                TokenKind::Space,
                TokenKind::PascalCaseIdent("Bar42".into())
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let toks = tokenize_str("123 -42 3.14 +2.7e-3");
        assert_eq!(
            toks,
            vec![
                TokenKind::IntLit(123),
                TokenKind::Space,
                TokenKind::IntLit(-42),
                TokenKind::Space,
                TokenKind::FloatLit(3.14),
                TokenKind::Space,
                TokenKind::FloatLit(2.7e-3)
            ]
        );
    }

    #[test]
    fn test_string_literal_and_escape() {
        let toks = tokenize_str(r#""hello" "world\n" "foo\\bar""#);
        assert_eq!(
            toks,
            vec![
                TokenKind::StrLit("hello".into()),
                TokenKind::Space,
                TokenKind::StrLit("world\n".into()),
                TokenKind::Space,
                TokenKind::StrLit("foo\\bar".into())
            ]
        );
    }

    #[test]
    fn test_structural_tokens() {
        let toks = tokenize_str("{ } [ ] : ,");
        assert_eq!(
            toks,
            vec![
                TokenKind::OpenBrace,
                TokenKind::Space,
                TokenKind::CloseBrace,
                TokenKind::Space,
                TokenKind::OpenBracket,
                TokenKind::Space,
                TokenKind::CloseBracket,
                TokenKind::Space,
                TokenKind::Colon,
                TokenKind::Space,
                TokenKind::Comma
            ]
        );
    }

    #[test]
    fn test_comments_and_eol() {
        let toks = tokenize_str("// comment\nfoo\n//x\r\nbar");
        assert_eq!(
            toks,
            vec![
                TokenKind::Comment,
                TokenKind::KebabCaseIdent("foo".into()),
                TokenKind::Eol,
                TokenKind::Comment,
                TokenKind::KebabCaseIdent("bar".into())
            ]
        );
    }

    #[test]
    fn test_spaces_and_tabs() {
        let toks = tokenize_str("foo \t bar");
        assert_eq!(
            toks,
            vec![
                TokenKind::KebabCaseIdent("foo".into()),
                TokenKind::Space,
                TokenKind::KebabCaseIdent("bar".into())
            ]
        );
    }

    #[test]
    fn test_comment_without_eol() {
        let toks = tokenize_str("// bar");
        assert_eq!(toks, vec![TokenKind::Comment,]);
    }
}

#[cfg(test)]
mod fail_tests {
    use super::*;

    fn tokenize_str(s: &str) -> Result<Vec<Token>> {
        tokenize(s.chars()).collect()
    }

    #[test]
    fn test_malformed_comment() {
        let toks = tokenize_str("foo: 1\n / bar");
        assert!(toks.is_err());
        assert!(matches!(
            toks.unwrap_err(),
            Error::UnexpectedChar {
                pos: 9,
                expected: '/',
                found: ' '
            }
        ));
    }
}
