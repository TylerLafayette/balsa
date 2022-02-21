use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use super::{BalsaType, BalsaValue};

/// A dictionary of String-indexed values.
#[derive(Debug, Clone)]
pub struct Dictionary {
    map: HashMap<String, BalsaValue>,
    type_: BalsaType,
}

impl Deref for Dictionary {
    type Target = HashMap<String, BalsaValue>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Dictionary {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl PartialEq for Dictionary {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (key, value) in &self.map {
            if let Some(other_value) = other.get(key) {
                if value != other_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl Dictionary {
    pub fn get_type(&self) -> BalsaType {
        self.type_.clone()
    }
}
