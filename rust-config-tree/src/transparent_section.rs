//! Transparent array section wrappers for split-friendly configuration.
//!
//! Sections marked with `x-tree-transparent-array` serialize as YAML arrays in
//! single-file configs and as body-only arrays in split section files.

use confique::serde::{Deserialize, Deserializer, Serialize, Serializer};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use std::borrow::Cow;

/// Split-friendly nested section that transparently serializes as an array.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArraySection<T> {
    /// Inner items loaded from the transparent section body.
    pub items: Vec<T>,
}

impl<T> ArraySection<T> {
    /// Returns inner items as a slice.
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    /// Returns the number of inner items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether this section contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> From<ArraySection<T>> for Vec<T> {
    fn from(section: ArraySection<T>) -> Self {
        section.items
    }
}

impl<T: JsonSchema> JsonSchema for ArraySection<T> {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("ArraySection")
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        Vec::<T>::json_schema(generator)
    }
}

impl<T: Serialize> Serialize for ArraySection<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.items.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for ArraySection<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            items: Vec::<T>::deserialize(deserializer)?,
        })
    }
}

/// Generates a transparent array section wrapper with confique support.
///
/// # Examples
///
/// ```ignore
/// transparent_array_section! {
///     /// Child declarations loaded from YAML.
///     pub struct ChildrenConfigSection {
///         #[config(default = [{ "name": "worker" }])]
///         pub items: Vec<ChildDeclaration>,
///     }
/// }
/// ```
#[macro_export]
macro_rules! transparent_array_section {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $(#[$items_meta:meta])*
            $items_vis:vis items: Vec<$item:ty> $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        #[derive(Debug, Clone, PartialEq, confique::Config)]
        $vis struct $name {
            $(#[$items_meta])*
            $items_vis items: Vec<$item>,
        }

        impl Default for $name {
            fn default() -> Self {
                Self { items: Vec::new() }
            }
        }

        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($name))
            }

            fn json_schema(
                generator: &mut schemars::SchemaGenerator,
            ) -> schemars::Schema {
                Vec::<$item>::json_schema(generator)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: confique::serde::Serializer,
            {
                self.items.serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: confique::serde::Deserializer<'de>,
            {
                Ok(Self {
                    items: Vec::<$item>::deserialize(deserializer)?,
                })
            }
        }

        impl $name {
            /// Returns inner items as a slice.
            pub fn as_slice(&self) -> &[$item] {
                &self.items
            }

            /// Returns the number of inner items.
            pub fn len(&self) -> usize {
                self.items.len()
            }

            /// Returns whether this section contains no items.
            pub fn is_empty(&self) -> bool {
                self.items.is_empty()
            }
        }

        impl From<$name> for Vec<$item> {
            fn from(section: $name) -> Self {
                section.items
            }
        }
    };
}
