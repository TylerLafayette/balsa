use std::fmt::Display;
use std::string::ToString;

use crate::balsa_types::{BalsaType, BalsaValue};

/// Represents all Balsa errors.
#[derive(Debug, Clone, PartialEq)]
pub enum BalsaError {
    /// Represents a failure that occurred during template compilation, before being rendered.
    CompileError(BalsaCompileError),
}

/// Represents an error in compiling a file.
#[derive(Debug, Clone, PartialEq)]
pub enum BalsaCompileError {
    /// A failure occurred while trying to parse and tokenize the raw template.
    TemplateParseFail(TemplateErrorContext<ParseFail>),
    /// A failure occurred while attempting to cast a value from one type to another.
    InvalidTypeCast(TemplateErrorContext<InvalidTypeCast>),
}

/// Wraps an error and provides file context.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateErrorContext<T>
where
    T: Display,
{
    /// The character position within the raw template at which the failure occurred.
    pub pos: usize,
    /// The wrapped error that occurred.
    pub error: T,
}

/// Represents an error occurred while attempting to parse and tokenize the raw template.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseFail {
    Generic,
}

/// Represents an invalid or failed attempt to cast [`BalsaValue`] `value` from [`BalsaType`] `from` to [`BalsaType`] `to`.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidTypeCast {
    /// The value of the attempted type cast.
    pub value: BalsaValue,
    /// The origin type from which the value was trying to be casted.
    pub from: BalsaType,
    /// The destination type to which the value was trying to be casted.
    pub to: BalsaType,
}

impl Display for BalsaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalsaError::CompileError(e) => write!(f, "compile error: {}", e),
        }
    }
}

impl Display for BalsaCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalsaCompileError::TemplateParseFail(t) => t.fmt(f),
            BalsaCompileError::InvalidTypeCast(t) => t.fmt(f),
        }
    }
}

impl<T> Display for TemplateErrorContext<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at position {}", self.error, self.pos)
    }
}

impl Display for ParseFail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parser failed")
    }
}

impl Display for InvalidTypeCast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to cast value `{}` of type `{}` to type `{}`",
            self.value, self.from, self.to
        )
    }
}
