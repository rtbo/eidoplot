use std::{fmt, str::FromStr};

use eidoplot_color::Color;

use crate::{
    RichTextBuilder, font,
    rich::{TextOptProps, TextProps},
};

/// Position into an input stream
pub type Pos = usize;

/// Byte span into an input stream
/// (first pos, one past last pos)
pub type Span = (Pos, Pos);

#[derive(Debug, Clone)]
pub enum ParseRichTextError {
    UnmatchedTag(Span),
    UnterminatedTag(Span),
    InvalidEscSequence(Span, char),
    UnexpectedEndOfStr(Pos),
    UnknownClass(Span, String),
    BadPropValue(Span, String, String),
}

impl ParseRichTextError {
    pub fn span(&self) -> Span {
        match self {
            ParseRichTextError::UnmatchedTag(span) => *span,
            ParseRichTextError::UnterminatedTag(span) => *span,
            ParseRichTextError::InvalidEscSequence(span, _) => *span,
            ParseRichTextError::UnexpectedEndOfStr(pos) => (*pos, *pos),
            ParseRichTextError::UnknownClass(span, _) => *span,
            ParseRichTextError::BadPropValue(span, _, _) => *span,
        }
    }
}

impl fmt::Display for ParseRichTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseRichTextError::UnmatchedTag(..) => {
                write!(f, "unmatched tag",)
            }
            ParseRichTextError::UnterminatedTag(..) => {
                write!(f, "unterminated tag",)
            }
            ParseRichTextError::InvalidEscSequence(_, c) => {
                write!(f, "invalid escape sequence: \\{}", c)
            }
            ParseRichTextError::UnexpectedEndOfStr(..) => {
                write!(f, "unexpected end of string")
            }
            ParseRichTextError::UnknownClass(_, class) => {
                write!(f, "unknown class or property: '{}'", class)
            }
            ParseRichTextError::BadPropValue(_, prop, value) => {
                write!(f, "bad value '{}' for property '{}'", value, prop)
            }
        }
    }
}

impl std::error::Error for ParseRichTextError {}

#[derive(Debug, Clone)]
pub struct ParsedRichText<C> {
    pub text: String,
    pub prop_spans: Vec<(Pos, Pos, TextOptProps<C>)>,
}

impl<C> ParsedRichText<C>
where
    C: Color + PartialEq,
{
    pub fn into_builder(self, root_props: TextProps<C>) -> RichTextBuilder<C> {
        let mut builder = RichTextBuilder::new(self.text, root_props);
        for (start, end, props) in self.prop_spans {
            builder.add_span(start, end, props);
        }
        builder
    }
}

pub fn parse_rich_text<C>(fmt: &str) -> Result<ParsedRichText<C>, ParseRichTextError>
where
    C: Color + FromStr,
{
    let parser = RichTextParser::new(fmt);
    parser.parse()
}

pub fn parse_rich_text_with_classes<C>(
    fmt: &str,
    user_classes: &[(String, TextOptProps<C>)],
) -> Result<ParsedRichText<C>, ParseRichTextError>
where
    C: Color + FromStr,
{
    let parser = RichTextParser::new_with_classes(fmt, user_classes);
    parser.parse()
}

#[derive(Debug, Clone)]
struct RichTextParser<'a, C> {
    fmt: &'a str,
    user_classes: &'a [(String, TextOptProps<C>)],
}

impl<'a, C> RichTextParser<'a, C>
where
    C: Color + FromStr,
{
    pub fn new(fmt: &'a str) -> Self {
        Self {
            fmt,
            user_classes: &[],
        }
    }

    pub fn new_with_classes(fmt: &'a str, user_classes: &'a [(String, TextOptProps<C>)]) -> Self {
        Self { fmt, user_classes }
    }

    pub fn parse(&self) -> Result<ParsedRichText<C>, ParseRichTextError> {
        let tokens = lex::tokenize(self.fmt.chars());
        let mut prop_stack = Vec::new();

        let mut text = String::new();
        let mut prop_spans = Vec::new();
        for token in tokens {
            let token = token?;
            match token.1 {
                lex::TokenKind::StrLit(s) => {
                    if text.is_empty() {
                        text = s;
                    } else {
                        text.push_str(&s);
                    }
                }
                lex::TokenKind::OpenTag(opening_tag) => {
                    let props = self.opening_tag_to_props(token.0, &opening_tag)?;
                    let close_tag = opening_tag
                        .0
                        .into_iter()
                        .map(|prop| prop.prop)
                        .collect::<Vec<_>>();
                    prop_stack.push((text.len(), token.0, close_tag, props));
                }
                lex::TokenKind::CloseTag(closing_tag) => {
                    for idx in (0..prop_stack.len()).rev() {
                        if closing_tag.0.iter().all(|c| prop_stack[idx].2.contains(c)) {
                            let start_pos = prop_stack[idx].0;
                            let end_pos = text.len();
                            let props = prop_stack.remove(idx).3;
                            prop_spans.push((start_pos, end_pos, props));
                            break;
                        }
                    }
                }
            }
        }

        if !prop_stack.is_empty() {
            return Err(ParseRichTextError::UnmatchedTag(prop_stack[0].1));
        }

        Ok(ParsedRichText { text, prop_spans })
    }

    fn merge_props(base: TextOptProps<C>, overlay: &TextOptProps<C>) -> TextOptProps<C> {
        TextOptProps {
            font_family: overlay.font_family.clone().or_else(|| base.font_family),
            font_weight: overlay.font_weight.or(base.font_weight),
            font_width: overlay.font_width.or(base.font_width),
            font_style: overlay.font_style.or(base.font_style),
            font_size: overlay.font_size.or(base.font_size),
            fill: overlay.fill.or(base.fill),
            stroke: overlay.stroke.or(base.stroke),
            underline: overlay.underline.or(base.underline),
            strikeout: overlay.strikeout.or(base.strikeout),
        }
    }

    fn opening_tag_to_props(
        &self,
        span: Span,
        tag: &lex::OpeningTag,
    ) -> Result<TextOptProps<C>, ParseRichTextError> {
        let mut props = TextOptProps::default();
        for prop in &tag.0 {
            // if no value, it is a class, or boolean prop.
            // we first check for user classes, if no match,
            // we apply built-in classes,
            if let Some(value) = &prop.value {
                match prop.prop.as_str() {
                    "font-size" | "size" | "sz" => {
                        if let Ok(size) = value.parse::<f32>() {
                            props.font_size = Some(size);
                        }
                    }
                    "font-family" | "font" | "family" | "ff" => {
                        props.font_family =
                            Some(font::parse_font_families(value).map_err(|_| {
                                ParseRichTextError::BadPropValue(
                                    span,
                                    prop.prop.clone(),
                                    value.clone(),
                                )
                            })?);
                    }
                    "font-weight" | "weight" | "fw" => {
                        let weight: font::Weight = value.parse().map_err(|_| {
                            ParseRichTextError::BadPropValue(span, prop.prop.clone(), value.clone())
                        })?;
                        props.font_weight = Some(weight);
                    }
                    "font-style" | "style" | "fs" => {
                        let style: font::Style = value.parse().map_err(|_| {
                            ParseRichTextError::BadPropValue(span, prop.prop.clone(), value.clone())
                        })?;
                        props.font_style = Some(style);
                    }
                    "font-width" | "width" | "font-stretch" | "stretch" => {
                        let width: font::Width = value.parse().map_err(|_| {
                            ParseRichTextError::BadPropValue(span, prop.prop.clone(), value.clone())
                        })?;
                        props.font_width = Some(width);
                    }
                    "color" | "fill" => {
                        let color: C = value.parse().map_err(|_| {
                            ParseRichTextError::BadPropValue(span, prop.prop.clone(), value.clone())
                        })?;
                        props.fill = Some(color);
                    }
                    "outline" | "stroke" => {
                        let color: C = value.parse().map_err(|_| {
                            ParseRichTextError::BadPropValue(span, prop.prop.clone(), value.clone())
                        })?;
                        props.fill = Some(color);
                    }
                    _ => {
                        return Err(ParseRichTextError::UnknownClass(span, prop.prop.clone()));
                    }
                }
                continue;
            }
            if prop.value.is_none() {
                let mut has_user = false;
                for (class_name, class_props) in self.user_classes {
                    if class_name == &prop.prop {
                        props = Self::merge_props(props, class_props);
                        has_user = true;
                    }
                }

                if has_user {
                    continue;
                }

                match prop.prop.as_str() {
                    // font weight
                    "thin" => {
                        props.font_weight = Some(font::Weight::THIN);
                    }
                    "extra-light" => {
                        props.font_weight = Some(font::Weight::EXTRA_LIGHT);
                    }
                    "light" => {
                        props.font_weight = Some(font::Weight::LIGHT);
                    }
                    "medium" => {
                        props.font_weight = Some(font::Weight::MEDIUM);
                    }
                    "semi-bold" => {
                        props.font_weight = Some(font::Weight::SEMIBOLD);
                    }
                    "bold" => {
                        props.font_weight = Some(font::Weight::BOLD);
                    }
                    "extra-bold" | "extrabold" => {
                        props.font_weight = Some(font::Weight::EXTRA_BOLD);
                    }
                    "black" => {
                        props.font_weight = Some(font::Weight::BLACK);
                    }

                    // font style
                    "italic" => {
                        props.font_style = Some(font::Style::Italic);
                    }
                    "oblique" => {
                        props.font_style = Some(font::Style::Oblique);
                    }

                    // font width
                    "ultra-condensed" => {
                        props.font_width = Some(font::Width::UltraCondensed);
                    }
                    "extra-condensed" => {
                        props.font_width = Some(font::Width::ExtraCondensed);
                    }
                    "condensed" => {
                        props.font_width = Some(font::Width::Condensed);
                    }
                    "semi-condensed" => {
                        props.font_width = Some(font::Width::SemiCondensed);
                    }
                    "semi-expanded" => {
                        props.font_width = Some(font::Width::SemiExpanded);
                    }
                    "expanded" => {
                        props.font_width = Some(font::Width::Expanded);
                    }
                    "extra-expanded" => {
                        props.font_width = Some(font::Width::ExtraExpanded);
                    }
                    "ultra-expanded" => {
                        props.font_width = Some(font::Width::UltraExpanded);
                    }

                    // for normal, we set them all
                    "normal" => {
                        props.font_weight = Some(font::Weight::NORMAL);
                        props.font_style = Some(font::Style::Normal);
                        props.font_width = Some(font::Width::Normal);
                    }

                    "underline" => {
                        props.underline = Some(true);
                    }
                    "strikeout" => {
                        props.strikeout = Some(true);
                    }

                    other => {
                        // still no match, we check for a fill color
                        let color: C = other.parse().map_err(|_| {
                            ParseRichTextError::UnknownClass(span, other.to_string())
                        })?;
                        props.fill = Some(color);
                    }
                }
            }
        }
        Ok(props)
    }
}

mod input {
    use super::Pos;

    /// A cursor over an input stream of characters.
    /// It keeps track of the current position in the stream.
    #[derive(Debug, Clone)]
    pub struct Cursor<I> {
        // input iterator
        input: I,
        // current position in the stream
        pos: Pos,
    }

    impl<I> Cursor<I> {
        pub fn new(input: I) -> Self {
            Self {
                input,
                pos: Pos::default(),
            }
        }

        pub fn pos(&self) -> Pos {
            self.pos
        }
    }

    impl<I> Cursor<I>
    where
        I: Iterator<Item = char> + Clone,
    {
        pub fn first(&self) -> Option<char> {
            self.input.clone().next()
        }
    }

    impl<I> Iterator for Cursor<I>
    where
        I: Iterator<Item = char>,
    {
        type Item = char;

        fn next(&mut self) -> Option<Self::Item> {
            let next = self.input.next();
            if let Some(c) = next {
                self.pos += c.len_utf8();
            }
            next
        }
    }

    impl<I> std::iter::FusedIterator for Cursor<I> where I: std::iter::FusedIterator<Item = char> {}
}

mod lex {
    use std::iter::FusedIterator;

    use super::input::Cursor;
    use super::{ParseRichTextError, Pos, Span};

    #[derive(Debug, Clone, PartialEq)]
    pub struct OpeningProp {
        pub prop: String,
        pub value: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct OpeningTag(pub Vec<OpeningProp>);

    pub type ClosingProp = String;

    #[derive(Debug, Clone, PartialEq)]
    pub struct ClosingTag(pub Vec<ClosingProp>);

    #[derive(Debug, Clone, PartialEq)]
    pub enum TokenKind {
        OpenTag(OpeningTag),
        CloseTag(ClosingTag),
        StrLit(String),
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
        type Item = Result<Token, ParseRichTextError>;

        fn next(&mut self) -> Option<Result<Token, ParseRichTextError>> {
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
        fn next_token_kind(
            &mut self,
            start_pos: Pos,
        ) -> Result<Option<TokenKind>, ParseRichTextError> {
            let Some(c) = self.cursor.first() else {
                return Ok(None);
            };
            match c {
                '[' => {
                    self.cursor.next();
                    let tag = self.parse_tag(start_pos)?;
                    Ok(Some(tag))
                }
                _ => {
                    let lit = self.parse_str_lit(start_pos)?;
                    Ok(Some(TokenKind::StrLit(lit)))
                }
            }
        }

        fn parse_esc_sequence(&mut self, start_pos: Pos) -> Result<char, ParseRichTextError> {
            let Some(c) = self.cursor.next() else {
                return Err(ParseRichTextError::UnexpectedEndOfStr(start_pos));
            };
            match c {
                '[' => Ok('['),
                '\\' => Ok('\\'),
                _ => Err(ParseRichTextError::InvalidEscSequence(
                    (start_pos, self.cursor.pos()),
                    c,
                )),
            }
        }

        fn parse_str_lit(
            &mut self,
            _start_pos: Pos,
        ) -> Result<String, ParseRichTextError> {
            let mut buf = String::new();
            loop {
                let pos = self.cursor.pos();
                match self.cursor.first() {
                    None => break,
                    Some('[') => break,
                    Some('\\') => {
                        self.cursor.next();
                        buf.push(self.parse_esc_sequence(pos)?);
                    }
                    Some(c) => {
                        self.cursor.next();
                        buf.push(c);
                    }
                }
            }
            Ok(buf)
        }

        fn parse_tag(&mut self, start_pos: Pos) -> Result<TokenKind, ParseRichTextError> {
            let Some(c) = self.cursor.first() else {
                return Err(ParseRichTextError::UnexpectedEndOfStr(start_pos));
            };
            if c == '/' {
                self.cursor.next();
                self.parse_closing_tag(start_pos)
            } else {
                self.parse_opening_tag(start_pos)
            }
        }

        fn parse_opening_tag(&mut self, start_pos: Pos) -> Result<TokenKind, ParseRichTextError> {
            let mut props = Vec::new();
            loop {
                let pos = self.cursor.pos();
                match self.cursor.first() {
                    None => return Err(ParseRichTextError::UnterminatedTag((start_pos, pos))),
                    Some(']') => {
                        self.cursor.next();
                        break;
                    }
                    Some(';') => {
                        self.cursor.next();
                    }
                    Some(_) => {
                        let class = self.parse_opening_prop(start_pos)?;
                        props.push(class);
                    }
                }
            }
            Ok(TokenKind::OpenTag(OpeningTag(props)))
        }

        fn parse_opening_prop(
            &mut self,
            start_pos: Pos,
        ) -> Result<OpeningProp, ParseRichTextError> {
            let mut prop_buf = String::new();
            let mut value_buf = String::new();
            let mut in_value = false;
            loop {
                let pos = self.cursor.pos();
                match self.cursor.first() {
                    None => return Err(ParseRichTextError::UnterminatedTag((start_pos, pos))),
                    Some(']') | Some(';') => {
                        break;
                    }
                    Some('=') if !in_value => {
                        in_value = true;
                        self.cursor.next();
                    }
                    Some(c) => {
                        self.cursor.next();
                        if in_value {
                            value_buf.push(c);
                        } else {
                            prop_buf.push(c);
                        }
                    }
                }
            }
            let prop = prop_buf.trim().to_string();
            let value = if value_buf.is_empty() {
                None
            } else {
                Some(value_buf.trim().to_string())
            };
            Ok(OpeningProp { prop, value })
        }

        fn parse_closing_tag(&mut self, _start_pos: Pos) -> Result<TokenKind, ParseRichTextError> {
            let mut classes = Vec::new();
            loop {
                let pos = self.cursor.pos();
                match self.cursor.first() {
                    None => return Err(ParseRichTextError::UnterminatedTag((pos, pos))),
                    Some(']') => {
                        self.cursor.next();
                        break;
                    }
                    Some(';') => {
                        self.cursor.next();
                    }
                    Some(_) => {
                        let class = self.parse_closing_prop(pos)?;
                        classes.push(class);
                    }
                }
            }
            Ok(TokenKind::CloseTag(ClosingTag(classes)))
        }

        fn parse_closing_prop(
            &mut self,
            start_pos: Pos,
        ) -> Result<ClosingProp, ParseRichTextError> {
            let mut class_buf = String::new();
            loop {
                let pos = self.cursor.pos();
                match self.cursor.first() {
                    None => return Err(ParseRichTextError::UnterminatedTag((start_pos, pos))),
                    Some(']') | Some(';') => {
                        break;
                    }
                    Some(c) => {
                        self.cursor.next();
                        class_buf.push(c);
                    }
                }
            }
            let class = class_buf.trim().to_string();
            Ok(class)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_tag() {
        let input = "Some [bold]bold[/bold] text";
        let tokens: Vec<_> = lex::tokenize(input.chars())
            .map(|res| res.unwrap().1)
            .collect();
        assert_eq!(
            &tokens,
            &[
                lex::TokenKind::StrLit("Some ".to_string()),
                lex::TokenKind::OpenTag(lex::OpeningTag(vec![lex::OpeningProp {
                    prop: "bold".to_string(),
                    value: None,
                }])),
                lex::TokenKind::StrLit("bold".to_string()),
                lex::TokenKind::CloseTag(lex::ClosingTag(vec!["bold".to_string()])),
                lex::TokenKind::StrLit(" text".to_string()),
            ]
        );
    }

    #[test]
    fn lex_escape() {
        let input = r#"Some \[bold]bold\[/bold] \\text"#;
        let tokens: Vec<_> = lex::tokenize(input.chars())
            .map(|res| res.unwrap().1)
            .collect();
        assert_eq!(
            &tokens,
            &[lex::TokenKind::StrLit(
                "Some [bold]bold[/bold] \\text".to_string()
            ),]
        );
    }

    #[test]
    fn lex_prop_tag() {
        let input = "Some [fs=12]small[/fs] text";
        let tokens: Vec<_> = lex::tokenize(input.chars())
            .map(|res| res.unwrap().1)
            .collect();
        assert_eq!(
            &tokens,
            &[
                lex::TokenKind::StrLit("Some ".to_string()),
                lex::TokenKind::OpenTag(lex::OpeningTag(vec![lex::OpeningProp {
                    prop: "fs".to_string(),
                    value: Some("12".to_string()),
                }])),
                lex::TokenKind::StrLit("small".to_string()),
                lex::TokenKind::CloseTag(lex::ClosingTag(vec!["fs".to_string()])),
                lex::TokenKind::StrLit(" text".to_string()),
            ]
        );
    }

    #[test]
    fn lex_multiple_prop_tag() {
        let input = "Some [fs=12;ff=Arial]small[/fs;ff] text";
        let tokens: Vec<_> = lex::tokenize(input.chars())
            .map(|res| res.unwrap().1)
            .collect();
        assert_eq!(
            &tokens,
            &[
                lex::TokenKind::StrLit("Some ".to_string()),
                lex::TokenKind::OpenTag(lex::OpeningTag(vec![
                    lex::OpeningProp {
                        prop: "fs".to_string(),
                        value: Some("12".to_string()),
                    },
                    lex::OpeningProp {
                        prop: "ff".to_string(),
                        value: Some("Arial".to_string()),
                    }
                ])),
                lex::TokenKind::StrLit("small".to_string()),
                lex::TokenKind::CloseTag(lex::ClosingTag(vec!["fs".to_string(), "ff".to_string()])),
                lex::TokenKind::StrLit(" text".to_string()),
            ]
        );
    }

    #[test]
    fn lex_escape_tag() {
        let input = "Freq. [italic]\\[Hz][/italic]";
        let tokens: Vec<_> = lex::tokenize(input.chars())
            .map(|res| res.unwrap().1)
            .collect();
        assert_eq!(
            &tokens,
            &[
                lex::TokenKind::StrLit("Freq. ".to_string()),
                lex::TokenKind::OpenTag(lex::OpeningTag(vec![lex::OpeningProp {
                    prop: "italic".to_string(),
                    value: None,
                }])),
                lex::TokenKind::StrLit("[Hz]".to_string()),
                lex::TokenKind::CloseTag(lex::ClosingTag(vec!["italic".to_string()])),
            ]
        );
    }
}
