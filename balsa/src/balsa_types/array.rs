use std::ops::{Deref, DerefMut};

use super::{BalsaType, BalsaValue};

/// An array of BalsaValues.
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    vec: Vec<BalsaValue>,
    type_: BalsaType,
}

impl Deref for Array {
    type Target = Vec<BalsaValue>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl Array {
    /// Returns the type of the Array elements.
    pub fn get_type(&self) -> BalsaType {
        self.type_.clone()
    }
}
