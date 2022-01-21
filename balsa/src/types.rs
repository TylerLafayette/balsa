/// Represents a typed value in a Balsa template.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum BalsaValue {
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
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum BalsaType {
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
