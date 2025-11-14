//! Provides utilities for handling optional updates to values in a type-safe way.
//! The `Patch` enum represents either an update with a new value or an explicit ignore
//! instruction, making it clearer than using `Option` for update operations.

/// Represents a potential update to a value, either providing a new value or explicitly
/// indicating that the value should be ignored/unchanged.
///
/// This is useful in PATCH-style updates where some fields should be updated while others
/// remain unchanged. Unlike `Option`, `Patch` makes the intent to ignore a value explicit.
#[derive(Default, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Patch<T> {
    /// Explicitly indicates that the existing value should remain unchanged
    #[default]
    Ignore,
    /// Contains a value that should replace the existing value
    Update(T),
}

impl<T> Patch<T> {
    /// Creates a new `Patch` variant that will update to the given value
    pub const fn update(value: T) -> Self {
        Self::Update(value)
    }

    /// Creates a new `Patch::Ignore` variant indicating no update should occur
    pub const fn ignore() -> Self {
        Self::Ignore
    }

    pub const fn is_ignore(&self) -> bool {
        matches!(self, Self::Ignore)
    }

    /// Returns a new `Patch` that references the inner value without taking ownership
    pub const fn as_ref(&self) -> Patch<&T> {
        match self {
            Self::Update(value) => Patch::Update(value),
            Self::Ignore => Patch::Ignore,
        }
    }

    /// Converts the `Patch` into an `Option` that borrows the inner value
    pub const fn as_option_ref(&self) -> Option<&T> {
        match self {
            Self::Update(value) => Some(value),
            Self::Ignore => None,
        }
    }

    /// Converts the `Patch` into an `Option` containing a clone of the inner value
    pub fn as_option(&self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Self::Update(value) => Some(value.clone()),
            Self::Ignore => None,
        }
    }

    /// Converts the `Patch` into an `Option`, consuming self
    pub fn into_option(self) -> Option<T> {
        match self {
            Self::Update(value) => Some(value),
            Self::Ignore => None,
        }
    }
}

impl<T> From<Patch<T>> for Option<T> {
    fn from(value: Patch<T>) -> Self {
        value.into_option()
    }
}

impl<T> From<Option<T>> for Patch<T> {
    fn from(value: Option<T>) -> Self {
        value.map_or_else(|| Self::Ignore, |value| Self::Update(value))
    }
}

impl<T> PartialEq<Option<T>> for Patch<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Option<T>) -> bool {
        match (self, other) {
            (Self::Update(value), Some(other)) => value == other,
            (Self::Ignore, None) => true,
            _ => false,
        }
    }
}

#[cfg(feature = "serde")]
mod serde {
    use super::Patch;
    use serde::{Deserialize, Serialize};

    impl<T> Serialize for Patch<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::Update(v) => serializer.serialize_some(v),
                Self::Ignore => serializer.serialize_none(),
            }
        }
    }

    impl<'de, T> Deserialize<'de> for Patch<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            // Delegate to Option<T> and map back into Patch<T>
            let opt = Option::<T>::deserialize(deserializer)?;
            Ok(opt.map_or_else(|| Self::Ignore, |v| Self::Update(v)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let update = Patch::update(42);
        let ignore: Patch<i32> = Patch::ignore();

        assert!(matches!(update, Patch::Update(42)));
        assert!(matches!(ignore, Patch::Ignore));
    }

    #[test]
    fn test_option_conversions() {
        let update: Patch<i32> = Patch::update(42);
        let ignore: Patch<i32> = Patch::ignore();

        assert_eq!(Option::from(update), Some(42));
        assert_eq!(Option::from(ignore), None::<i32>);

        assert_eq!(Patch::from(Some(42)), Patch::Update(42));
        assert_eq!(Patch::from(None::<i32>), Patch::Ignore);
    }

    #[test]
    fn test_reference_operations() {
        let update = Patch::update(42);
        let ignore: Patch<i32> = Patch::ignore();

        assert_eq!(update.as_ref(), Patch::Update(&42));
        assert_eq!(ignore.as_ref(), Patch::Ignore);

        assert_eq!(update.as_option_ref(), Some(&42));
        assert_eq!(ignore.as_option_ref(), None);
    }

    #[test]
    #[allow(clippy::default_trait_access)]
    fn test_default() {
        let patch: Patch<i32> = Default::default();
        assert!(matches!(patch, Patch::Ignore));
    }

    #[test]
    fn test_equality() {
        let update = Patch::update(42);
        let ignore: Patch<i32> = Patch::ignore();

        assert_eq!(update, Some(42));
        assert_ne!(update, None);
        assert_eq!(ignore, None);
        assert_ne!(ignore, Some(42));
    }
}
