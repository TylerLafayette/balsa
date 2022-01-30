//! Balsa is a delightfully simple HTML template engine designed to be used
//! in user interfaces such as a CMS where a user needs to be able to edit
//! the template's parameters. Balsa includes support for extra metadata,
//! such as friendly variable names, default values, and types.
//!
//! The Balsa API is based around the builder pattern. Templates are constructed,
//! built and then executed on a list of parameters, whether that be a map or a struct.
//!
//! # Example
//!
//! ```rust,no_run,ignore
//! use balsa::{Balsa, AsParameters, BalsaParameters};
//!
//! pub struct TemplateStruct {
//!     pub header_text: String,
//!     pub current_year: i32,
//! }
//!
//! impl AsParameters for TemplateStruct {
//!     fn as_parameters(&self) -> BalsaParameters {
//!         BalsaParameters::new()
//!             .string("headerText", self.header_text)
//!             .int("currentYear", self.current_year)
//!     }
//! }
//!
//! fn main() {
//!     let template =
//!         Balsa::from_file("template.html")
//!             .build_struct::<TemplateStruct>();
//!
//!     let output_html =
//!         template
//!             .render_html_string(&TemplateStruct {
//!                 header_text: "Hello world!",
//!                 current_year: 2022,
//!             });
//! }
//! ```

#![deny(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    rustdoc::broken_intra_doc_links
)]

/// Compiler for parsed Balsa templates.
pub(crate) mod balsa_compiler;
/// Parser for Balsa templates.
pub(crate) mod balsa_parser;
/// Type casting for Balsa types.
pub(crate) mod balsa_type_cast;
/// Types supported in Balsa templates.
pub(crate) mod balsa_types;
/// Error types for Balsa compilation.
pub mod errors;
/// Name constants for parameters.
pub(crate) mod parameter_names;
pub use balsa_types::{BalsaType, BalsaValue};

/// Internal type converters.
pub(crate) mod converters;
/// Internal types for the compiler, etc.
pub(crate) mod types;
/// Validators for color formats etc.
pub(crate) mod validators;
pub use types::BalsaResult;

/// Parser combinators
pub(crate) mod parser;

mod parameters_builder;
pub use parameters_builder::{AsParameters, BalsaParameters};

// use std::path::Path;

/// The top-level unit struct used for initializing a Balsa builder.
#[derive(Debug)]
pub struct Balsa;

/// A built template that accepts a specified type as its input
/// for compilation.
pub trait BalsaStaticTemplate<T>: Sync + Send {
    /// Compiles the template with the specified params.
    fn compile(&self, params: &T) -> String;
}

/// This trait is implemented by types that can be used to build a
/// Balsa template.
pub trait BalsaBuilder: Sync + Send + Sized {
    /// Builds a template that accepts a specified type that implements
    /// the [`AsParameters`] trait.
    fn build_struct<T: AsParameters + Sized>() -> dyn BalsaStaticTemplate<T>;
}

// impl Balsa {
//     pub fn from_file<P: AsRef<Path>>(path: P) -> impl BalsaBuilder {}
// }
