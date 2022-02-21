//! Contains the implementation for the [`BalsaValue`] method [`BalsaValue.try_cast`] which allows
//! Balsa values to be casted from one [`BalsaType`] to another.

use crate::{
    balsa_types::{BalsaType, BalsaValue},
    errors::InvalidTypeCast,
    validators::is_valid_color,
};

impl BalsaValue {
    /// Attempts to cast the [`BalsaValue`] from its [`BalsaType`] to the `target` [`BalsaType`].
    ///
    /// On success, a new [`BalsaValue`] is returned, otherwise an [`InvalidTypeCast`] error will
    /// be returned.
    pub(crate) fn try_cast(&self, target_type: BalsaType) -> Result<BalsaValue, InvalidTypeCast> {
        let err = Err(InvalidTypeCast {
            value: self.clone(),
            from: self.get_type(),
            to: target_type.clone(),
        });

        match self {
            BalsaValue::String(value) => match &target_type {
                BalsaType::String => Ok(self.clone()),
                BalsaType::Color => {
                    // Strings can be casted to colors only if they are valid.
                    if is_valid_color(value) {
                        Ok(BalsaValue::Color(value.clone()))
                    } else {
                        err
                    }
                }
                _ => err,
            },
            BalsaValue::Color(value) => match &target_type {
                BalsaType::String => Ok(BalsaValue::String(value.clone())),
                BalsaType::Color => Ok(self.clone()),
                _ => err,
            },
            BalsaValue::Integer(value) => match &target_type {
                BalsaType::Integer => Ok(self.clone()),
                BalsaType::Float => {
                    if let Ok(rounded) = i32::try_from(*value) {
                        Ok(BalsaValue::Float(rounded.into()))
                    } else {
                        err
                    }
                }
                _ => err,
            },
            BalsaValue::Float(_value) => match &target_type {
                BalsaType::Float => Ok(self.clone()),
                _ => err,
            },
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balsa_integer_to_float_cast() {
        let integer = BalsaValue::Integer(80000);

        integer.try_cast(BalsaType::Float).expect(&format!(
            "`BalsaValue::try_cast` should correctly cast value `{}` from type `{}` to type `{}`",
            integer,
            integer.get_type(),
            BalsaType::Float
        ));
    }
}
