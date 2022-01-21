use std::collections::HashMap;

use crate::types::BalsaValue;

/// A struct used for generating a hashmap of parameters using
/// the builder pattern.
#[derive(Debug, Default)]
pub struct BalsaParameters {
    parameters: HashMap<String, BalsaValue>,
}

impl BalsaParameters {
    /// Creates a new empty parameter list.
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    /// Appends a String value to the parameters list.
    pub fn string(&self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.insert(key, BalsaValue::String(value.into()))
    }

    /// Appends a hex code or RGB value to the parameters list.
    pub fn color(&self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.insert(key, BalsaValue::Color(value.into()))
    }

    /// Appends an integer value to the parameters list.
    pub fn int(&self, key: impl Into<String>, value: impl Into<i64>) -> Self {
        self.insert(key, BalsaValue::Integer(value.into()))
    }

    /// Appends a float value to the parameters list.
    pub fn float(&self, key: impl Into<String>, value: impl Into<f64>) -> Self {
        self.insert(key, BalsaValue::Float(value.into()))
    }

    /// Returns a new BalsaParameters with the provided
    /// key and value inserted into the parameters map.
    fn insert(&self, key: impl Into<String>, value: BalsaValue) -> Self {
        let mut parameters = self.parameters.clone();
        parameters.insert(key.into(), value);

        Self { parameters }
    }

    /// Gets a single value from the parameter list.
    pub(crate) fn get(&self, key: impl Into<String>) -> Option<BalsaValue> {
        self.parameters.get(&key.into()).map(|x| x.to_owned())
    }
}

/// This trait allows any data type to be converted into a source
/// of parameters for a Balsa template.
///
/// # Example
/// ```rust,no_run
/// # use balsa::*;
/// struct TemplateParams {
///     header_text: String,
///     red: String,
///     small_int: i32,
/// }
///
/// impl AsParameters for TemplateParams {
///     fn as_parameters(&self) -> BalsaParameters {
///         BalsaParameters::new()
///             .string("headerText", self.header_text.clone())
///             .color("red", self.red.clone())
///             .int("smallInt", self.small_int)
///     }
/// }
/// ```
pub trait AsParameters {
    /// Transforms the object into a parameter list.
    fn as_parameters(&self) -> BalsaParameters;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_parameters() {
        let params = BalsaParameters::new()
            .string("hello", "world")
            .color("red", "#ff0000")
            .int("currentYear", 2022)
            .float("floatyFloat", 20.23);

        assert_eq!(
            params.get("hello"),
            Some(BalsaValue::String("world".to_string())),
            "String parameter `hello` does not equal `world`"
        );

        assert_eq!(
            params.get("red"),
            Some(BalsaValue::Color("#ff0000".to_string())),
            "Color parameter `red` does not equal `#ff0000`"
        );

        assert_eq!(
            params.get("currentYear"),
            Some(BalsaValue::Integer(2022)),
            "Integer parameter `currentYear` does not equal `2022`"
        );

        assert_eq!(
            params.get("floatyFloat"),
            Some(BalsaValue::Float(20.23)),
            "Integer parameter `currentYear` does not equal `2022`"
        );
    }

    struct ParameterTestStruct {
        header_text: String,
        red: String,
        small_int: i32,
    }

    impl AsParameters for ParameterTestStruct {
        fn as_parameters(&self) -> BalsaParameters {
            BalsaParameters::new()
                .string("headerText", self.header_text.clone())
                .color("red", self.red.clone())
                .int("smallInt", self.small_int)
        }
    }

    #[test]
    fn struct_parameters() {
        let params = ParameterTestStruct {
            header_text: "Hello world!".to_string(),
            red: "#ff0000".to_string(),
            small_int: 123,
        };

        let balsa_params = params.as_parameters();

        assert_eq!(
            balsa_params.get("headerText"),
            Some(BalsaValue::String(params.header_text.clone())),
            "String parameter `headerText` does not equal `{}`",
            params.header_text
        );

        assert_eq!(
            balsa_params.get("red"),
            Some(BalsaValue::Color(params.red.clone())),
            "Color parameter `red` does not equal `{}`",
            params.red
        );

        assert_eq!(
            balsa_params.get("smallInt"),
            Some(BalsaValue::Integer(params.small_int.into())),
            "Integer parameter `smallInt` does not equal `{}`",
            params.small_int
        );
    }
}
