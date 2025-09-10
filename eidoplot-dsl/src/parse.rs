use std::fmt;

use crate::ast;
use crate::DiagTrait;
use crate::lex::{self, Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub enum Error {
    Lex(lex::Error),
    UnexpectedEndOfInput(Span),
    UnexpectedToken(Token, Option<String>),
}

impl Error {
}

impl From<lex::Error> for Error {
    fn from(e: lex::Error) -> Self {
        Error::Lex(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lex(err) => err.fmt(f),
            Error::UnexpectedEndOfInput(_) => {
                write!(f, "Unexpected end of input")
            }
            Error::UnexpectedToken(tok, expected) => {
                write!(f, "Unexpected token: {:?}", tok.kind)?;
                if let Some(expected) = expected {
                    write!(f, " (expected {})", expected)?;
                }
                Ok(())
            }
        }
    }
}

impl DiagTrait for Error {
    fn span(&self) -> Span {
        match self {
            Error::Lex(err) => err.span(),
            Error::UnexpectedEndOfInput(span) => *span,
            Error::UnexpectedToken(tok, _) => tok.span,
        }
    }

    fn message(&self) -> String {
        format!("{}", self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn parse<I>(input: I) -> Result<Vec<ast::Prop>>
where
    I: Iterator<Item = char> + Clone,
{
    let tokens = lex::tokenize(input);
    let mut parser = Parser::new(tokens);
    parser.parse_prop_list()
}

pub struct Parser<T> {
    tokens: T,
    last_span: Span,
}

impl<T> Parser<T> {
    pub fn new(tokens: T) -> Self {
        Self {
            tokens,
            last_span: Span::default(),
        }
    }
}

impl<T> Parser<T>
where
    T: Iterator<Item = lex::Result<Token>> + Clone,
{
    fn parse_prop_list(&mut self) -> Result<Vec<ast::Prop>> {
        let mut props = Vec::new();
        loop {
            self.ignore_com_eol();
            let Some(prop) = self.parse_prop()? else {
                break;
            };
            props.push(prop);
        }
        Ok(props)
    }

    fn parse_prop(&mut self) -> Result<Option<ast::Prop>> {
        self.ignore_opt_sp();
        let Some(tok) = self.first_token()? else {
            return Ok(None);
        };

        let name = match tok.kind {
            TokenKind::KebabCaseIdent(s) => {
                self.bump_token();
                ast::Ident {
                    name: s,
                    span: tok.span,
                }
            }
            _ => return Ok(None),
        };

        self.ignore_opt_sp();

        // Check for optional colon and value
        let value = match self.first_token()? {
            Some(Token {
                kind: TokenKind::Colon,
                ..
            }) => {
                self.bump_token();
                self.ignore_opt_sp();
                Some(self.parse_prop_value()?)
            }
            _ => None,
        };

        self.ignore_opt_sp();
        // com-eol is handled by parse_prop_list

        Ok(Some(ast::Prop { name, value }))
    }

    fn parse_prop_value(&mut self) -> Result<ast::Value> {
        let tok = self.expect_next_token()?;

        match tok.kind {
            TokenKind::StrLit(s) => {
                let (span, val) = self.parse_str_concatenation(tok.span, s)?;
                let scalar = ast::Scalar {
                    span,
                    kind: ast::ScalarKind::Str(val),
                };
                self.parse_scalar_or_seq(scalar)
            }
            TokenKind::IntLit(val) => {
                let scalar = ast::Scalar {
                    span: tok.span,
                    kind: ast::ScalarKind::Int(val),
                };
                self.parse_scalar_or_seq(scalar)
            }
            TokenKind::FloatLit(val) => {
                let scalar = ast::Scalar {
                    span: tok.span,
                    kind: ast::ScalarKind::Float(val),
                };
                self.parse_scalar_or_seq(scalar)
            }
            TokenKind::PascalCaseIdent(name) => {
                // both struct and enums can start with a pascal case identifier
                Ok(self.parse_struct_or_enum(ast::Ident {
                    span: tok.span,
                    name,
                })?)
            }
            TokenKind::OpenBrace => Ok(ast::Value::Struct(self.parse_struct(tok.span, None)?)),
            TokenKind::OpenBracket => Ok(ast::Value::Array(self.parse_array(tok.span)?)),
            _ => Err(Error::UnexpectedToken(tok, Some("value".to_string()))),
        }
    }

    fn parse_scalar_or_seq(&mut self, starter: ast::Scalar) -> Result<ast::Value> {
        self.ignore_opt_sp();
        match self.first_token()? {
            Some(Token {
                kind: TokenKind::Comma,
                ..
            }) => {
                self.bump_token();
            }
            _ => return Ok(ast::Value::Scalar(starter)),
        }

        let mut res_span = starter.span;
        let mut res_scalars = vec![starter];

        loop {
            self.ignore_opt_sp();

            match self.expect_first_token()? {
                Token {
                    kind: TokenKind::StrLit(val),
                    span,
                } => {
                    self.bump_token();
                    let (span, val) = self.parse_str_concatenation(span, val)?;
                    let scalar = ast::Scalar {
                        span,
                        kind: ast::ScalarKind::Str(val),
                    };
                    res_scalars.push(scalar);
                    res_span.1 = span.1;
                }
                Token {
                    kind: TokenKind::IntLit(val),
                    span,
                    ..
                } => {
                    self.bump_token();
                    let scalar = ast::Scalar {
                        span,
                        kind: ast::ScalarKind::Int(val),
                    };
                    res_scalars.push(scalar);
                    res_span.1 = span.1;
                }
                Token {
                    kind: TokenKind::FloatLit(val),
                    span,
                    ..
                } => {
                    self.bump_token();
                    let scalar = ast::Scalar {
                        span,
                        kind: ast::ScalarKind::Float(val),
                    };
                    res_scalars.push(scalar);
                    res_span.1 = span.1;
                }
                Token {
                    kind: TokenKind::PascalCaseIdent(name),
                    span,
                } => {
                    self.bump_token();
                    let scalar = ast::Scalar {
                        span,
                        kind: ast::ScalarKind::Enum(name),
                    };
                    res_scalars.push(scalar);
                    res_span.1 = span.1;
                }
                _ => (),
            }

            self.ignore_opt_sp();
            match self.first_token()? {
                Some(Token {
                    kind: TokenKind::Comma,
                    span,
                }) => {
                    self.bump_token();
                    res_span.1 = span.1;
                }
                _ => break,
            }
        }

        Ok(ast::Value::Seq(ast::Seq {
            span: res_span,
            scalars: res_scalars,
        }))
    }

    fn parse_str_concatenation(
        &mut self,
        start_span: Span,
        starter: String,
    ) -> Result<(Span, String)> {
        let mut res_str = starter;
        let mut res_span = start_span;

        loop {
            self.ignore_com_eol();
            match self.first_token()? {
                Some(Token {
                    kind: TokenKind::StrLit(s),
                    span,
                }) => {
                    res_span.1 = span.1;
                    res_str.push_str(&s);
                    self.bump_token();
                }
                _ => break,
            }
        }

        Ok((res_span, res_str))
    }

    fn parse_struct_or_enum(&mut self, ident: ast::Ident) -> Result<ast::Value> {
        self.ignore_opt_sp();
        match self.first_token()? {
            Some(Token {
                kind: TokenKind::OpenBrace,
                ..
            }) => {
                self.bump_token();
                Ok(ast::Value::Struct(
                    self.parse_struct(ident.span, Some(ident))?,
                ))
            }
            _ => Ok(ast::Value::Scalar(ast::Scalar {
                span: ident.span,
                kind: ast::ScalarKind::Enum(ident.name),
            })),
        }
    }

    fn parse_struct(&mut self, start_span: Span, typ: Option<ast::Ident>) -> Result<ast::Struct> {
        let props = self.parse_prop_list()?;
        match self.expect_next_token()? {
            Token {
                span,
                kind: TokenKind::CloseBrace,
            } => Ok(ast::Struct {
                span: (start_span.0, span.1),
                typ,
                props,
            }),
            tok => Err(Error::UnexpectedToken(tok, Some("}".to_string()))),
        }
    }

    fn parse_array(&mut self, start_span: Span) -> Result<ast::Array> {
        self.ignore_com_eol();
        let array_kind = match self.expect_next_token()? {
            Token {
                kind: TokenKind::StrLit(val),
                span,
            } => {
                let (_, val) = self.parse_str_concatenation(span, val)?;
                self.parse_str_sequence(vec![val])?
            }
            Token {
                kind: TokenKind::IntLit(val),
                ..
            } => self.parse_int_or_float_sequence(vec![val])?,
            Token {
                kind: TokenKind::FloatLit(val),
                ..
            } => self.parse_float_sequence(vec![val])?,
            _ => ast::ArrayKind::Empty,
        };
        self.ignore_com_eol();
        let span = self.expect_token(TokenKind::CloseBracket)?;
        Ok(ast::Array {
            span: (start_span.0, span.1),
            kind: array_kind,
        })
    }

    fn parse_int_or_float_sequence(&mut self, starter: Vec<i64>) -> Result<ast::ArrayKind> {
        let mut vec = starter;
        loop {
            self.ignore_com_eol();
            match self.expect_first_token()? {
                Token {
                    kind: TokenKind::Comma,
                    ..
                } => {
                    self.bump_token();
                }
                _ => break,
            }
            self.ignore_com_eol();
            match self.expect_first_token()? {
                Token {
                    kind: TokenKind::IntLit(val),
                    ..
                } => {
                    self.bump_token();
                    vec.push(val);
                }
                Token {
                    kind: TokenKind::FloatLit(val),
                    ..
                } => {
                    self.bump_token();
                    let mut fvec: Vec<f64> = vec.into_iter().map(|v| v as f64).collect();
                    fvec.push(val);
                    return self.parse_float_sequence(fvec);
                }
                _ => (),
            }
        }
        return Ok(ast::ArrayKind::Int(vec));
    }

    fn parse_float_sequence(&mut self, starter: Vec<f64>) -> Result<ast::ArrayKind> {
        let mut vec = starter;
        loop {
            self.ignore_com_eol();
            match self.expect_first_token()? {
                Token {
                    kind: TokenKind::Comma,
                    ..
                } => {
                    self.bump_token();
                }
                _ => break,
            }
            self.ignore_com_eol();
            match self.expect_first_token()? {
                Token {
                    kind: TokenKind::IntLit(val),
                    ..
                } => {
                    self.bump_token();
                    vec.push(val as f64);
                }
                Token {
                    kind: TokenKind::FloatLit(val),
                    ..
                } => {
                    self.bump_token();
                    vec.push(val);
                }
                _ => (),
            }
        }
        return Ok(ast::ArrayKind::Float(vec));
    }

    fn parse_str_sequence(&mut self, starter: Vec<String>) -> Result<ast::ArrayKind> {
        let mut vec = starter;
        loop {
            self.ignore_com_eol();
            match self.expect_first_token()? {
                Token {
                    kind: TokenKind::Comma,
                    ..
                } => {
                    self.bump_token();
                }
                _ => break,
            }
            self.ignore_com_eol();
            match self.expect_first_token()? {
                Token {
                    kind: TokenKind::StrLit(val),
                    span,
                } => {
                    self.bump_token();
                    let (_, val) = self.parse_str_concatenation(span, val)?;
                    vec.push(val);
                }
                _ => (),
            }
        }
        return Ok(ast::ArrayKind::Str(vec));
    }

    fn ignore_opt_sp(&mut self) {
        loop {
            match self.first_token() {
                Ok(Some(Token {
                    kind: TokenKind::Space,
                    ..
                })) => self.bump_token(),
                _ => break,
            }
        }
    }

    fn ignore_com_eol(&mut self) {
        loop {
            self.ignore_opt_sp();
            match self.first_token() {
                Ok(Some(Token {
                    kind: TokenKind::Eol,
                    ..
                })) => self.bump_token(),
                Ok(Some(Token {
                    kind: TokenKind::Comment,
                    ..
                })) => self.bump_token(),
                _ => break,
            }
        }
    }
}

impl<T> Parser<T>
where
    T: Iterator<Item = lex::Result<Token>> + Clone,
{
    fn next_token(&mut self) -> lex::Result<Option<Token>> {
        let tok = self.tokens.next().transpose()?;
        if let Some(tok) = &tok {
            self.last_span = tok.span;
        }
        Ok(tok)
    }

    fn expect_next_token(&mut self) -> Result<Token> {
        let Some(tok) = self.next_token()? else {
            return Err(Error::UnexpectedEndOfInput(self.last_span));
        };
        Ok(tok)
    }
    fn first_token(&self) -> lex::Result<Option<Token>> {
        self.tokens.clone().next().transpose()
    }

    fn bump_token(&mut self) {
        self.next_token().unwrap();
    }

    fn expect_first_token(&mut self) -> Result<Token> {
        let Some(tok) = self.first_token()? else {
            return Err(Error::UnexpectedEndOfInput(self.last_span));
        };
        Ok(tok)
    }

    fn expect_token(&mut self, tok_kind: TokenKind) -> Result<Span> {
        let tok = self.expect_next_token()?;
        if tok.kind != tok_kind {
            Err(Error::UnexpectedToken(tok, None))
        } else {
            Ok(tok.span)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let dsl = "";
        let props = parse(dsl.chars()).unwrap();
        assert!(props.is_empty());
    }

    #[test]
    fn test_empty_prop() {
        let dsl = "foo";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props,
            &[ast::Prop {
                name: ast::Ident {
                    name: "foo".to_string(),
                    span: (0, 3,),
                },
                value: None,
            }]
        );
    }

    #[test]
    fn test_int() {
        let dsl = "foo: 1234";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props,
            &[ast::Prop {
                name: ast::Ident {
                    name: "foo".to_string(),
                    span: (0, 3,),
                },
                value: Some(ast::Value::Scalar(ast::Scalar {
                    span: (5, 9,),
                    kind: ast::ScalarKind::Int(1234),
                })),
            }]
        );
    }

    #[test]
    fn test_float() {
        let dsl = "foo: 12.34";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props,
            &[ast::Prop {
                name: ast::Ident {
                    name: "foo".to_string(),
                    span: (0, 3,),
                },
                value: Some(ast::Value::Scalar(ast::Scalar {
                    span: (5, 10,),
                    kind: ast::ScalarKind::Float(12.34),
                })),
            }]
        );
    }

    #[test]
    fn test_str() {
        let dsl = "foo: \"string\"";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props,
            &[ast::Prop {
                name: ast::Ident {
                    name: "foo".to_string(),
                    span: (0, 3,),
                },
                value: Some(ast::Value::Scalar(ast::Scalar {
                    span: (5, 13,),
                    kind: ast::ScalarKind::Str("string".into()),
                })),
            }]
        );
    }

    #[test]
    fn test_str_concatenation() {
        let dsl = r#"foo: "a" "b" "c""#;
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props[0].value,
            Some(ast::Value::Scalar(ast::Scalar {
                span: (5, 16),
                kind: ast::ScalarKind::Str("abc".into()),
            }))
        );
    }

    #[test]
    fn test_enum_value() {
        let dsl = "foo: Bar";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props,
            &[ast::Prop {
                name: ast::Ident {
                    name: "foo".to_string(),
                    span: (0, 3),
                },
                value: Some(ast::Value::Scalar(ast::Scalar {
                    span: (5, 8),
                    kind: ast::ScalarKind::Enum("Bar".into()),
                })),
            }]
        );
    }

    #[test]
    fn test_seq_of_scalars() {
        let dsl = "foo: 1, 2, 3";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props[0].value,
            Some(ast::Value::Seq(ast::Seq {
                span: (5, 12),
                scalars: vec![
                    ast::Scalar {
                        span: (5, 6),
                        kind: ast::ScalarKind::Int(1),
                    },
                    ast::Scalar {
                        span: (8, 9),
                        kind: ast::ScalarKind::Int(2),
                    },
                    ast::Scalar {
                        span: (11, 12),
                        kind: ast::ScalarKind::Int(3),
                    },
                ]
            }))
        );
    }

    #[test]
    fn test_array_of_ints() {
        let dsl = "foo: [1, 2, 3]";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props[0].value,
            Some(ast::Value::Array(ast::Array {
                span: (5, 14),
                kind: ast::ArrayKind::Int(vec![1, 2, 3]),
            }))
        );
    }

    #[test]
    fn test_array_of_floats() {
        let dsl = "foo: [1.1, 2.2, 3.3]";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props[0].value,
            Some(ast::Value::Array(ast::Array {
                span: (5, 20,),
                kind: ast::ArrayKind::Float(vec![1.1, 2.2, 3.3]),
            }))
        );
    }

    #[test]
    fn test_array_of_strings() {
        let dsl = r#"foo: ["a", "b", "c"]"#;
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props[0].value,
            Some(ast::Value::Array(ast::Array {
                span: (5, 20,),
                kind: ast::ArrayKind::Str(vec!["a".into(), "b".into(), "c".into()]),
            }))
        );
    }

    #[test]
    fn test_struct_with_type() {
        let dsl = "foo: Bar { baz: 1 }";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props[0].value,
            Some(ast::Value::Struct(ast::Struct {
                span: (5, 19),
                typ: Some(ast::Ident {
                    name: "Bar".into(),
                    span: (5, 8),
                }),
                props: vec![ast::Prop {
                    name: ast::Ident {
                        name: "baz".into(),
                        span: (11, 14),
                    },
                    value: Some(ast::Value::Scalar(ast::Scalar {
                        span: (16, 17),
                        kind: ast::ScalarKind::Int(1),
                    })),
                }]
            }))
        );
    }

    #[test]
    fn test_struct_without_type() {
        let dsl = "foo: { bar: 2 }";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props[0].value,
            Some(ast::Value::Struct(ast::Struct {
                span: (5, 15),
                typ: None,
                props: vec![ast::Prop {
                    name: ast::Ident {
                        name: "bar".into(),
                        span: (7, 10),
                    },
                    value: Some(ast::Value::Scalar(ast::Scalar {
                        span: (12, 13),
                        kind: ast::ScalarKind::Int(2),
                    })),
                }]
            }))
        );
    }

    #[test]
    fn test_empty_struct() {
        let dsl = "foo: { }";
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(
            props[0].value,
            Some(ast::Value::Struct(ast::Struct {
                span: (5, 8),
                typ: None,
                props: vec![],
            }))
        );
    }

    #[test]
    fn test_comments_and_eol() {
        let dsl = r#"
// comment
foo: 1

bar: 2 // another comment
"#;
        let props = parse(dsl.chars()).unwrap();
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].name.name, "foo");
        assert_eq!(props[1].name.name, "bar");
    }
}

#[cfg(test)]
mod fail_tests {
    use super::*;

    #[test]
    fn test_unterminated_string() {
        let dsl = "foo: \"bar\nbaz";
        let res = parse(dsl.chars());
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(
            err,
            Error::Lex(lex::Error::UnterminatedString { .. })
        ));
        assert!(err.to_string().contains("Unterminated string"));
    }

    #[test]
    fn test_unterminated_struct() {
        let dsl = "foo: {\n bar: 1\n";
        let res = parse(dsl.chars());
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, Error::UnexpectedEndOfInput(_)));
        assert!(err.to_string().contains("Unexpected end of input"));
    }
}
