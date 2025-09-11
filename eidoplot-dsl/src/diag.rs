use core::fmt;

use miette::MietteSpanContents;

use crate::Span;
use crate::{parse, lex};

pub trait DiagTrait: fmt::Debug + fmt::Display {
    fn span(&self) -> Span;
    fn message(&self) -> String;
    fn help(&self) -> Option<String> {
        None
    }
}

impl DiagTrait for lex::Error {
    fn span(&self) -> Span {
        match self {
            lex::Error::UnexpectedChar { pos, .. } => (*pos, *pos + 1),
            lex::Error::UnexpectedEndOfFile(pos) => (*pos, *pos),
            lex::Error::UnterminatedString { span, .. } => *span,
            lex::Error::InvalidEscSequence(span, _) => *span,
            lex::Error::InvalidNumber(span, _) => *span,
            lex::Error::InvalidKebabIdent(span, _) => *span,
            lex::Error::InvalidPascalIdent(span, _) => *span,
        }
    }

    fn message(&self) -> String {
        format!("{}", self)
    }
}

impl DiagTrait for parse::Error {
    fn span(&self) -> Span {
        match self {
            parse::Error::Lex(err) => err.span(),
            parse::Error::UnexpectedEndOfInput(span) => *span,
            parse::Error::UnexpectedToken(tok, _) => tok.span,
        }
    }

    fn message(&self) -> String {
        format!("{}", self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Source<'a> {
    pub name: Option<&'a str>,
    pub src: &'a str,
}

impl miette::SourceCode for Source<'_> {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        let start = span.offset();
        let end = start + span.len();

        if start > self.src.len() || end > self.src.len() || start > end {
            return Err(miette::MietteError::OutOfBounds);
        }

        let content = <str as miette::SourceCode>::read_span(
            &self.src,
            span,
            context_lines_before,
            context_lines_after,
        )?;
        if let Some(name) = self.name {
            let content = MietteSpanContents::new_named(
                name.to_string(),
                content.data(),
                *content.span(),
                content.line(),
                content.column(),
                content.line_count(),
            )
            .with_language("edsl");
            Ok(Box::new(content))
        } else {
            Ok(content)
        }
    }
}

#[derive(Debug)]
pub struct Diagnostic<'a> {
    diag: Box<dyn DiagTrait>,
    source: Source<'a>,
}

impl<'a> Diagnostic<'a> {
    pub fn new(diag: Box<dyn DiagTrait>, source: Source<'a>) -> Self {
        Self { diag, source }
    }
}

impl fmt::Display for Diagnostic<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diag.message())?;
        if let Some(help) = self.diag.help() {
            write!(f, "\nHelp: {}", help)?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostic<'_> {}

unsafe impl Send for Diagnostic<'_> {}
unsafe impl Sync for Diagnostic<'_> {}

impl miette::Diagnostic for Diagnostic<'_> {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        None
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.diag
            .help()
            .map(|h| Box::new(h) as Box<dyn fmt::Display>)
    }

    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        let (start, end) = self.diag.span();
        let labeled_span =
            miette::LabeledSpan::new(Some(self.diag.message()), start.into(), end - start);
        Some(Box::new(std::iter::once(labeled_span)))
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source as &dyn miette::SourceCode)
    }
}
