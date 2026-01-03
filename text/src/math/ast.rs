
pub struct Node {
    typ: NodeType,
    span: (usize, usize),
}

pub enum NodeType {
    Scalar(Scalar),
    Seq(Seq),
    Power(Power),
    Indice(Indice),
    Frac(Frac),
    Root(Root),
    Binary(Binary),
}

pub enum Scalar {
    Char(char),
    Str(String),
}

pub struct Seq {
    nodes: Vec<Node>,
}

pub enum AccentType {
    Hat,
    Check,
    Tilde,
    Acute,
    Grave,
    Dot,
    Ddot,
    Breve,
    Bar,
    Vec,
}

pub struct Accent {
    typ: AccentType,
    scalar: Scalar,
}

pub enum WideType {
    Bar,
    Vec,
    Hat,
    Tilde,
}

pub struct Wide {
    typ: WideType,
    node: Box<Node>,
}

pub struct Power {
    base: Box<Node>,
    exp: Box<Node>,
}

pub struct Indice {
    base: Box<Node>,
    index: Box<Node>,
}

pub struct Frac {
    numer: Box<Node>,
    denom: Box<Node>,
}

pub struct Root {
    rad: Box<Node>,
    mag: Option<Box<Node>>,
}

pub struct Binary {
    left: Box<Node>,
    op: Scalar,
    right: Box<Node>,
}
