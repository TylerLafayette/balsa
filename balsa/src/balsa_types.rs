use std::fmt::Display;

/// Represents a reference to a variable or key by name without any preceding characters like `$`.
pub(crate) type BalsaIdentifier = String;

/// Represents a typed value in a Balsa template.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) enum BalsaExpression {
    Identifier(BalsaIdentifier),
    Type(BalsaType),
    Value(BalsaValue),
}

/// Represents a typed value in a Balsa template.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum BalsaValue {
    /// A basic string.
    String(String),
    /// Can be either a hex code or an RGB value.
    Color(String),
    /// A 64-bit integer.
    Integer(i64),
    /// A 64-bit float.
    Float(f64),
}

/// Represents a type in a Balsa template.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum BalsaType {
    /// A basic string.
    String,
    /// Can be either a hex code or an RGB value.
    Color,
    /// A 64-bit integer.
    Integer,
    /// A 64-bit float.
    Float,
}

impl BalsaValue {
    /// Gets the [`BalsaType`] of a [`BalsaValue`].
    pub(crate) fn get_type(&self) -> BalsaType {
        match self {
            BalsaValue::String(_) => BalsaType::String,
            BalsaValue::Color(_) => BalsaType::Color,
            BalsaValue::Integer(_) => BalsaType::Integer,
            BalsaValue::Float(_) => BalsaType::Float,
        }
    }

    /// Checks if a [`BalsaValue`] is the provided [`BalsaType`] `type_`.
    pub(crate) fn is_type(&self, type_: BalsaType) -> bool {
        self.get_type() == type_
    }
}

impl Display for BalsaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalsaValue::String(s) => write!(f, r#""{}""#, s),
            BalsaValue::Color(c) => write!(f, r#"{}"#, c),
            BalsaValue::Integer(i) => write!(f, r#"{:?}"#, i),
            BalsaValue::Float(f_) => write!(f, r#"{}"#, f_),
        }
    }
}

impl Display for BalsaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BalsaType::String => write!(f, "string"),
            BalsaType::Color => write!(f, "color"),
            BalsaType::Integer => write!(f, "int"),
            BalsaType::Float => write!(f, "float"),
        }
    }
}
