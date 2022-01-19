/// Represents a type in a Balsa template.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub(crate) enum BalsaType {
    /// A basic string.
    String(String),
    /// Can be either a hex code or an RGB value.
    Color(String),
    /// A 64-bit integer.
    Integer(i64),
    /// A 64-bit float.
    Float(f64),
}
