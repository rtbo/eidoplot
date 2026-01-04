//! # plotive-dsl
//!
//! This crate provides a general-purpose and recursive DSL that describes
//! dictionaries, lists, sequences, or scalars such as numbers and strings.
//!
//! It is similar in spirit to JSON, YAML or TOML.
//! It enforces however a few conventions for naming and structuring data that
//! allow clearer and more concise descriptions.
//!
//! It is used by the [plotive](https://crates.io/crates/plotive) crate
//! to describe plots.
//!
//! Here is a simple example of a DSL document as used in plotive:
//! ```dsl
//! figure: {
//!     title: "Subplots"
//!     space: 10
//!     subplots: 2, 1
//!     plot: {
//!         subplot: 1, 1
//!         x-axis: shared("x"), Grid
//!         y-axis: "y1", Ticks
//!         series: Line {
//!             x-data: "x1"
//!             y-data: "y1"
//!         }
//!     }
//!     plot: {
//!         subplot: 2, 1
//!         x-axis: "x", PiMultipleTicks, Grid, id("x-axis")
//!         y-axis: "y2", Ticks
//!         series: Line {
//!             x-data: "x2"
//!             y-data: "y2"
//!         }
//!     }
//! }
//! ```
//!
//! Plotive DSL documents are parsed into an abstract syntax tree (AST)
//! defined in the [`ast`] module.
//! The AST can then be parsed by applications.
pub mod ast;
#[cfg(feature = "diag")]
mod diag;
mod input;
mod lex;
mod parse;

#[cfg(feature = "diag")]
pub use diag::{DiagReport, DiagResult, DiagTrait, Diagnostic, Source};
pub use input::Pos;
pub use lex::Span;
pub use parse::{Error, parse};
