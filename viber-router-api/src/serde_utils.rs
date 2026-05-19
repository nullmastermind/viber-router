use serde::{Deserialize, Deserializer};

/// Deserialize JSON `null` as `Some(None)` and a missing field as `None`,
/// so PATCH-style endpoints can distinguish "clear to NULL" from "leave unchanged".
///
/// Use with `#[serde(default, deserialize_with = "crate::serde_utils::double_option")]`
/// on fields typed `Option<Option<T>>`.
pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(de).map(Some)
}
