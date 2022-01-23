use std::{collections::HashMap, hash::Hash};

/// Converts a vector of two-element tuples to a
/// HashMap where the key is the type of the first element
/// and the value is the type of the second element.
pub(crate) fn tuple_vec_to_map<K, V>(tuples: Vec<(K, V)>) -> HashMap<K, V>
where
    K: Hash + Eq,
{
    let mut map = HashMap::new();

    for (key, value) in tuples {
        map.insert(key, value);
    }

    map
}
