use super::{ast, lex, Span};
use super::MathResult;
use lex::{Token, TokenKind};


pub fn parse<I>(input: I) -> MathResult<ast::Node>
where
    I: Iterator<Item = char> + Clone,
{
    let tokens = lex::tokenize(input);
    let mut parser = Parser::new(tokens);
    parser.parse_node()
}

pub struct Parser<T> {
    tokens: T,
}

impl<T> Parser<T> {
    pub fn new(tokens: T) -> Self {
        Self {
            tokens,
        }
    }
}

impl<T> Parser<T>
where
    T: Iterator<Item = MathResult<Token>> + Clone,
{
    fn parse_node(&mut self) -> MathResult<ast::Node> {
        loop {
            let Some(tok) = self.tokens.next() else {

            }

        }
    }
}
