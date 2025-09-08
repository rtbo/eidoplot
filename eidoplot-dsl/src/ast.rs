use crate::lex::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub span: Span,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prop {
    pub name: Ident,
    pub value: Option<Value>,
}

impl Prop {
    pub fn span(&self) -> Span {
        let start_span = self.name.span;
        if let Some(value) = &self.value {
            (start_span.0, value.span().1)
        } else {
            start_span
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Scalar(Scalar),
    Seq(Seq),
    Array(Array),
    Struct(Struct),
}

impl Value {
    pub fn span(&self) -> Span {
        match self {
            Value::Scalar(scalar) => scalar.span,
            Value::Seq(seq) => seq.span,
            Value::Array(array) => array.span,
            Value::Struct(struct_) => struct_.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scalar {
    pub span: Span,
    pub kind: ScalarKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScalarKind {
    Enum(String),
    Str(String),
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Seq {
    pub span: Span,
    pub scalars: Vec<Scalar>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub span: Span,
    pub kind: ArrayKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayKind {
    Empty,
    Int(Vec<i64>),
    Float(Vec<f64>),
    Str(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub span: Span,
    pub typ: Option<Ident>,
    pub props: Vec<Prop>,
}

impl Struct {
    pub fn has_prop(&self, name: &str) -> bool {
        self.props.iter().any(|p| p.name.name == name)
    }

    pub fn prop(&self, name: &str) -> Option<&Prop> {
        self.props.iter().find(|p| p.name.name == name)
    }

    pub fn take_prop(&mut self, name: &str) -> Option<Prop> {
        if let Some(pos) = self.props.iter().position(|p| p.name.name == name) {
            Some(self.props.remove(pos))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::input::Pos;

    use super::*;

    #[test]
    fn test_prop_span() {

        let prop = Prop {
            name: Ident {
                span: (
                    Pos {
                        index: 0,
                        line: 1,
                        column: 1
                    },
                    Pos {
                        index: 3,
                        line: 1,
                        column: 4
                    },
                ),
                name: "foo".to_string(),
            },
            value: Some(Value::Array(Array {
                span: (
                    Pos {
                        index: 5,
                        line: 1,
                        column: 6
                    },
                    Pos {
                        index: 14,
                        line: 1,
                        column: 15
                    }
                ),
                kind: ArrayKind::Int(vec![1, 2, 3]),
            }))
        };

        assert_eq!(
            prop.span(),
            (
                Pos {
                    index: 0,
                    line: 1,
                    column: 1
                },
                Pos {
                    index: 14,
                    line: 1,
                    column: 15
                }
            )
        );
    }
}