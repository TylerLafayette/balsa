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
/// Renderer for compiled Balsa templates.
pub(crate) mod balsa_renderer;
/// Type casting for Balsa types.
pub(crate) mod balsa_type_cast;
/// Types supported in Balsa templates.
pub(crate) mod balsa_types;
/// Error types for Balsa compilation.
pub mod errors;
pub use errors::BalsaError;
/// Name constants for parameters.
pub(crate) mod parameter_names;

use std::{
    fmt,
    fs::{self, File},
    marker::PhantomData,
    path::PathBuf,
};

use balsa_compiler::CompiledTemplate;
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

/// [`AsParameters`] trait and parameter builder methods.
mod parameters_builder;
pub use parameters_builder::{AsParameters, BalsaParameters};

/// The top-level unit struct used for initializing a Balsa builder.
#[derive(Debug)]
pub struct Balsa;

/// A trait for loading a raw template document as a String.
trait TemplateSource: fmt::Debug {
    fn read_template(&self) -> BalsaResult<String>;
}

/// Loads raw template from a file.
#[derive(Debug)]
struct FileSource {
    path: PathBuf,
}

impl TemplateSource for FileSource {
    fn read_template(&self) -> BalsaResult<String> {
        fs::read_to_string(&self.path).map_err(BalsaError::read_template_error)
    }
}

/// Loads raw template from a string.
#[derive(Debug, Clone)]
struct StringSource {
    raw_template: String,
}

impl TemplateSource for StringSource {
    fn read_template(&self) -> BalsaResult<String> {
        Ok(self.raw_template.clone())
    }
}

/// A struct for building a Balsa template from a static HTML document.
#[derive(Debug)]
pub struct BalsaBuilder {
    template_source: Box<dyn TemplateSource>,
}

/// A compiled template that can be rendered with the specified `T`.
pub trait BalsaTemplate<T>: Sync + Send {
    /// Renders the template with the specified `params` argument.
    fn render_html_string(&self, params: &T) -> BalsaResult<String>;
}

/// A compiled template that can be rendered with any type implementing [`AsParameters`].
///
/// Can be built with any object that implements [`AsParameters`].
#[derive(Debug, Clone)]
pub struct Template {
    raw_template: String, // TODO: more memory-efficient way of loading raw templates
    compiled_template: CompiledTemplate,
}

/// A compiled template that is pinned to the parameters type `T`. This is meant to provide a sort
/// of statically-typed feel to the template and add extra information.
///
/// Can only be built using an object of type `T`.
#[derive(Debug, Clone)]
pub struct TypedTemplate<T: AsParameters> {
    template: Template,
    _type: PhantomData<T>,
}

impl<T: AsParameters> BalsaTemplate<T> for Template {
    fn render_html_string(&self, params: &T) -> BalsaResult<String> {
        let renderer = balsa_renderer::Renderer::new(&self.raw_template, &self.compiled_template);
        let params = params.as_parameters();

        renderer.render_with_parameters(&params)
    }
}

impl<T: AsParameters + Sync + Send> BalsaTemplate<T> for TypedTemplate<T> {
    fn render_html_string(&self, params: &T) -> BalsaResult<String> {
        self.template.render_html_string(params)
    }
}

impl BalsaBuilder {
    /// Parses and compiles the template, returning a [`Template`] on success which takes any type
    /// implementing [`AsParameters`] as parameters for rendering.
    pub fn build(&self) -> BalsaResult<Template> {
        let raw_template = self.template_source.read_template()?;
        let tokens = balsa_parser::BalsaParser::parse(raw_template.clone())?;
        let compiled_template = balsa_compiler::Compiler::compile_from_tokens(&tokens)?;

        Ok(Template {
            raw_template,
            compiled_template,
        })
    }
    /// Parses and compiles the template, returning a [`TypedTemplate<T>`] on success which
    /// requires the specified type (which must implement [`AsParameters`]) as parameters for
    /// rendering.
    pub fn build_struct<T: AsParameters>(&self) -> BalsaResult<TypedTemplate<T>> {
        Ok(TypedTemplate {
            template: self.build()?,
            _type: PhantomData::default(),
        })
    }
}

impl Balsa {
    /// Creates a new [`BalsaBuilder`] from a file using the provided path.
    pub fn from_file<P: AsRef<PathBuf>>(path: P) -> BalsaBuilder {
        BalsaBuilder {
            template_source: Box::new(FileSource {
                path: path.as_ref().clone(),
            }),
        }
    }
    /// Creates a new [`BalsaBuilder`] from the provided template as a string.
    pub fn from_string(raw_template: impl Into<String>) -> BalsaBuilder {
        BalsaBuilder {
            template_source: Box::new(StringSource {
                raw_template: raw_template.into(),
            }),
        }
    }
}
