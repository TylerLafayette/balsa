use std::{fmt::Display, ops::Deref};

use crate::{
    balsa_types::{BalsaExpression, BalsaType, BalsaValue},
    Balsa,
};

/// Represents all Balsa errors.
#[derive(Debug, Clone, PartialEq)]
pub enum BalsaError {
    /// Represents a failure that occurred during template compilation, before being rendered.
    CompileError(BalsaCompileError),
    /// Represents a failure that occurred while rendering a template.
    RenderError(BalsaRenderError),
}

/// Represents an error in compiling a file.
#[derive(Debug, Clone, PartialEq)]
pub enum BalsaCompileError {
    /// A failure occurred while trying to parse and tokenize the raw template.
    TemplateParseFail(TemplateErrorContext<TemplateParseFail>),
    /// A failure occurred while attempting to cast a value from one type to another.
    InvalidTypeCast(TemplateErrorContext<InvalidTypeCast>),
    /// A provided type expression was malformed or didn't match a valid type.
    InvalidTypeExpression(TemplateErrorContext<InvalidTypeExpression>),
    /// The variant of the provided expression was invalid.
    ///
    /// e.g. Identifier passed instead of value
    InvalidExpression(TemplateErrorContext<InvalidExpression>),
    /// An invalid identifier was provided for a parameter block.
    InvalidIdentifierForParameterBlock(TemplateErrorContext<InvalidIdentifierForParameterBlock>),
    /// An invalid identifier was provided for a variable in a declaration block.
    InvalidIdentifierForDeclarationBlock(
        TemplateErrorContext<InvalidIdentifierForDeclarationBlock>,
    ),
    /// Unexpected parameter was provided to a parameter block.
    InvalidParameter(TemplateErrorContext<InvalidParameter>),
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
pub enum TemplateParseFail {
    /// Represents a generic parser fail.
    // TODO: more descriptive variants
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

/// Represents an type expression which does not match a valid type.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidTypeExpression {
    /// The parsed type expression.
    pub expression: BalsaExpression,
}

/// The variant of the provided expression was invalid.
///
/// e.g. Identifier passed instead of value
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidExpression {
    /// The parsed type expression.
    expression: BalsaExpression,
}

/// Represents an invalid identifier provided in a parameter block.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidIdentifierForParameterBlock {
    /// The parsed type expression.
    pub expression: BalsaExpression,
}

/// Represents an invalid identifier provided for a variable in a declaration block.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidIdentifierForDeclarationBlock {
    /// The parsed type expression.
    pub expression: BalsaExpression,
}

/// Represents an invalid parameter provided in a block.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidParameter {
    /// The name of the invalid parameter.
    pub parameter_name: String,
}

/// Represents an error in compiling a file.
#[derive(Debug, Clone, PartialEq)]
pub enum BalsaRenderError {
    /// A parameter was expected and no default value was provided.
    MissingParameter(MissingParameter),
    /// A parameter's value could not be casted to the specified type.
    InvalidParameterType(InvalidParameterType),
}

/// A parameter was expected and no default value was provided.
#[derive(Debug, Clone, PartialEq)]
pub struct MissingParameter {
    /// The name of the missing parameter.
    pub parameter_name: String,
}

/// A parameter's value could not be casted to the specified type.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidParameterType {
    /// The name of the parameter.
    pub parameter_name: String,
    /// The value that the parameter was set to.
    pub received_value: BalsaValue,
    /// The type of the provided parameter value.
    pub received_type: BalsaType,
    /// The expected type for the parameter.
    pub expected_type: BalsaType,
}

impl Display for BalsaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalsaError::CompileError(e) => write!(f, "compile error: {}", e),
            BalsaError::RenderError(e) => write!(f, "render error: {}", e),
        }
    }
}

impl Display for BalsaCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TemplateParseFail(e) => e.fmt(f),
            Self::InvalidTypeCast(e) => e.fmt(f),
            Self::InvalidTypeExpression(e) => e.fmt(f),
            Self::InvalidExpression(e) => e.fmt(f),
            Self::InvalidIdentifierForParameterBlock(e) => e.fmt(f),
            Self::InvalidIdentifierForDeclarationBlock(e) => e.fmt(f),
            Self::InvalidParameter(e) => e.fmt(f),
        }
    }
}

// Allow [`TemplateErrorContext`]s to be deref'd to their wrapped error types.
impl<T> Deref for TemplateErrorContext<T>
where
    T: Display,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.error
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

impl Display for TemplateParseFail {
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

impl Display for InvalidTypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid type expression `{}` does not match any known types",
            self.expression
        )
    }
}

impl Display for InvalidExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expression `{}` is an unexpected variant",
            self.expression
        )
    }
}

impl Display for InvalidIdentifierForParameterBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid identifier `{}` provided in parameter block",
            self.expression
        )
    }
}

impl Display for InvalidIdentifierForDeclarationBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid identifier `{}` provided in declaration block",
            self.expression
        )
    }
}

impl Display for InvalidParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid or unknown parameter `{}` provided",
            self.parameter_name
        )
    }
}

impl Display for BalsaRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingParameter(e) => e.fmt(f),
            Self::InvalidParameterType(e) => e.fmt(f),
        }
    }
}

impl Display for MissingParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected parameter `{}` but no parameter was found and no default value was provided",
            self.parameter_name
        )
    }
}

impl Display for InvalidParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "parameter `{}` but no parameter was found and no default value was provided",
            self.parameter_name
        )
    }
}
// Error constructor functions.
impl BalsaError {
    /// Creates a [`BalsaError::CompileError`] with the provided [`BalsaCompileError`].
    pub(crate) fn new_compile_error(error: BalsaCompileError) -> Self {
        Self::CompileError(error)
    }

    /// Creates a new [`BalsaError::CompileError`] which wraps a [`CompileError::TemlateParseFail`]
    /// which wraps a [`ParseFail::Generic`].
    pub(crate) fn generic_template_parse_fail(pos: usize) -> Self {
        Self::new_compile_error(BalsaCompileError::TemplateParseFail(
            Self::template_context(pos, TemplateParseFail::Generic),
        ))
    }

    /// Creates a new [`BalsaError::CompileError`] which wraps a [`CompileError::InvalidTypeCast`]
    /// which wraps a [`InvalidTypeCast`] with the provided arguments.
    pub(crate) fn invalid_type_cast(
        pos: usize,
        value: BalsaValue,
        from_type: BalsaType,
        to_type: BalsaType,
    ) -> Self {
        Self::new_compile_error(BalsaCompileError::InvalidTypeCast(Self::template_context(
            pos,
            InvalidTypeCast {
                value,
                from: from_type,
                to: to_type,
            },
        )))
    }

    /// Creates a new [`BalsaError::CompileError`] which wraps a
    /// [`CompileError::InvalidTypeExpression`] which wraps a [`InvalidTypeExpression`] with the
    /// provided expression.
    pub(crate) fn invalid_type_expression(pos: usize, expression: BalsaExpression) -> Self {
        Self::new_compile_error(BalsaCompileError::InvalidTypeExpression(
            Self::template_context(pos, InvalidTypeExpression { expression }),
        ))
    }

    /// Creates a new [`BalsaError::CompileError`] which wraps a
    /// [`CompileError::InvalidExpression`] which wraps a [`InvalidExpression`] with the
    /// provided expression.
    pub(crate) fn invalid_expression(pos: usize, expression: BalsaExpression) -> Self {
        Self::new_compile_error(BalsaCompileError::InvalidExpression(
            Self::template_context(pos, InvalidExpression { expression }),
        ))
    }

    /// Creates a new [`BalsaError::CompileError`] which wraps a
    /// [`CompileError::InvalidIdentifierForParameterBlock`] which wraps a
    /// [`InvalidIdentifierForParameterBlock`] with the provided arguments.
    pub(crate) fn invalid_identifier_in_parameter_block(
        pos: usize,
        expression: BalsaExpression,
    ) -> Self {
        Self::new_compile_error(BalsaCompileError::InvalidIdentifierForParameterBlock(
            Self::template_context(pos, InvalidIdentifierForParameterBlock { expression }),
        ))
    }

    /// Creates a new [`BalsaError::CompileError`] which wraps a
    /// [`CompileError::InvalidIdentifierForDeclarationBlock`] which wraps a
    /// [`InvalidIdentifierForDeclarationBlock`] with the provided arguments.
    pub(crate) fn invalid_identifier_in_declaration_block(
        pos: usize,
        expression: BalsaExpression,
    ) -> Self {
        Self::new_compile_error(BalsaCompileError::InvalidIdentifierForDeclarationBlock(
            Self::template_context(pos, InvalidIdentifierForDeclarationBlock { expression }),
        ))
    }

    /// Creates a new [`BalsaError::CompileError`] which wraps a
    /// [`CompileError::InvalidParameter`] which wraps a [`InvalidParameter`] with the provided
    /// parameter name.
    pub(crate) fn invalid_parameter(pos: usize, parameter_name: String) -> Self {
        Self::new_compile_error(BalsaCompileError::InvalidParameter(Self::template_context(
            pos,
            InvalidParameter { parameter_name },
        )))
    }

    pub(crate) fn new_render_error(error: BalsaRenderError) -> Self {
        Self::RenderError(error)
    }

    /// Creates a new [`BalsaError::RenderError`] which wraps a
    /// [`RenderError::MissingParameter`] which wraps a [`MissingParameter`] with the provided
    /// parameter name.
    pub(crate) fn missing_parameter(parameter_name: String) -> Self {
        Self::new_render_error(BalsaRenderError::MissingParameter(MissingParameter {
            parameter_name,
        }))
    }

    /// Creates a new [`BalsaError::RenderError`] which wraps a
    /// [`RenderError::InvalidParameterType`] which wraps a [`InvalidParameterType`] with the provided
    /// parameter name, parameter_value.
    pub(crate) fn invalid_parameter_type(
        parameter_name: String,
        received_value: BalsaValue,
        received_type: BalsaType,
        expected_type: BalsaType,
    ) -> Self {
        Self::new_render_error(BalsaRenderError::InvalidParameterType(
            InvalidParameterType {
                parameter_name,
                received_value,
                received_type,
                expected_type,
            },
        ))
    }

    /// Makes a [`TemplateErrorContext<T>`] with the provided `pos` and `error` of type `T`.
    fn template_context<T: Display>(pos: usize, error: T) -> TemplateErrorContext<T> {
        TemplateErrorContext { pos, error }
    }
}
