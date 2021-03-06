mod dictionary;
pub(crate) use dictionary::Dictionary;

mod array;
pub(crate) use array::Array;

use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

/// Represents a reference to a variable or key by name without any preceding characters like `$`.
pub(crate) type BalsaIdentifier = String;

/// Represents a low-level parsed expression in a Balsa template.
///
/// Should only be used for error-checking.
#[derive(Debug, Clone, PartialEq)]
pub enum BalsaExpression {
    Identifier(BalsaIdentifier),
    Type(BalsaType),
    Value(BalsaValue),
}

/// Represents a typed value in a Balsa template.
#[derive(Debug, Clone, PartialEq)]
pub enum BalsaValue {
    /// A basic string.
    String(String),
    /// Can be either a hex code or an RGB value.
    Color(String),
    /// A 64-bit integer.
    Integer(i64),
    /// A 64-bit float.
    Float(f64),
    /// An array of values.
    Array(Array),
    /// A dictionary of values indexed by a String.
    Dictionary(Dictionary),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub struct RecursiveBalsaType(Box<BalsaType>);

impl Deref for RecursiveBalsaType {
    type Target = BalsaType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents a type in a Balsa template.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub enum BalsaType {
    /// A basic string.
    String,
    /// Can be either a hex code or an RGB value.
    Color,
    /// A 64-bit integer.
    Integer,
    /// A 64-bit float.
    Float,
    /// An array of the specified type.
    Array(RecursiveBalsaType),
    /// A String-indexed dictionary of the specified type.
    Dictionary(RecursiveBalsaType),
}

impl BalsaExpression {
    /// Attempt to unwrap a [`BalsaExpression`] as an identifier.
    pub(crate) fn as_identifier(&self) -> Option<String> {
        if let Self::Identifier(s) = self {
            Some(s.clone())
        } else {
            None
        }
    }

    /// Attempt to unwrap a [`BalsaExpression`] as a type.
    pub(crate) fn as_type(&self) -> Option<BalsaType> {
        if let Self::Type(t) = self {
            Some(t.clone())
        } else {
            None
        }
    }

    /// Attempt to unwrap a [`BalsaExpression`] as a value.
    pub(crate) fn as_value(&self) -> Option<BalsaValue> {
        if let Self::Value(v) = self {
            Some(v.clone())
        } else {
            None
        }
    }
}

impl BalsaValue {
    /// Gets the [`BalsaType`] of a [`BalsaValue`].
    pub(crate) fn get_type(&self) -> BalsaType {
        match self {
            BalsaValue::String(_) => BalsaType::String,
            BalsaValue::Color(_) => BalsaType::Color,
            BalsaValue::Integer(_) => BalsaType::Integer,
            BalsaValue::Float(_) => BalsaType::Float,
            BalsaValue::Array(_) => todo!(),
            BalsaValue::Dictionary(_) => todo!(),
        }
    }

    /// Checks if a [`BalsaValue`] is the provided [`BalsaType`] `type_`.
    pub(crate) fn is_type(&self, type_: BalsaType) -> bool {
        self.get_type() == type_
    }
}

impl Display for BalsaExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalsaExpression::Identifier(i) => write!(f, "{}", i),
            BalsaExpression::Type(t) => t.fmt(f),
            BalsaExpression::Value(v) => v.fmt(f),
        }
    }
}

impl Display for BalsaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BalsaValue::String(s) => write!(f, r#""{}""#, s),
            BalsaValue::Color(c) => write!(f, r#"{}"#, c),
            BalsaValue::Integer(i) => write!(f, r#"{:?}"#, i),
            BalsaValue::Float(f_) => write!(f, r#"{}"#, f_),
            BalsaValue::Array(_) => todo!(),
            BalsaValue::Dictionary(_) => todo!(),
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
            BalsaType::Array(_) => todo!(),
            BalsaType::Dictionary(_) => todo!(),
        }
    }
}
