//! Type-safe view types for different access modes on data models.
//!
//! This crate provides a trait-based system for generating specialized view types
//! for different operations on data models (Get/Read, Create, Patch/Update). Each
//! view type only includes the fields relevant to that operation, enforcing API
//! contracts at compile time.
//!
//! # Core Concepts
//!
//! ## View Modes
//!
//! The crate defines three access modes for models:
//!
//! - **[`ViewModeGet`]**: For read operations, retrieving existing data
//! - **[`ViewModeCreate`]**: For create operations, accepting input to create new entities
//! - **[`ViewModePatch`]**: For update operations, allowing partial modifications
//!
//! ## The View Trait
//!
//! The [`View`] trait associates a type with its representation in a specific mode:
//!
//! ```rust,ignore
//! trait View<M: ViewMode> {
//!     type Type;
//! }
//! ```
//!
//! This allows the same model to have different representations depending on the operation.
//!
//! ## Patch Type
//!
//! The [`Patch<T>`] enum is central to update operations, providing an explicit way to
//! distinguish between "don't update this field" and "update this field to a value":
//!
//! - `Patch::Ignore`: Leave the field unchanged
//! - `Patch::Update(value)`: Update the field to the given value
//!
//! This is clearer than using `Option<T>` for updates, especially when dealing with
//! optional fields.
//!
//! # Usage
//!
//! ## Basic Example
//!
//! ```rust
//! use model_views::{Views, Patch};
//!
//! #[derive(Views)]
//! #[views(serde)]
//! struct User {
//!     // ID is returned when reading, but can't be set during create/update
//!     #[views(get = "required", create = "forbidden", patch = "forbidden")]
//!     id: u64,
//!     
//!     // Name is always required for all operations
//!     #[views(get = "required", create = "required", patch = "patch")]
//!     name: String,
//!     
//!     // Email is optional everywhere
//!     #[views(get = "optional", create = "optional", patch = "optional")]
//!     email: String,
//! }
//!
//! // This generates three types:
//!
//! // UserGet - for reading user data
//! let user_get = UserGet {
//!     id: 1,
//!     name: "Alice".to_string(),
//!     email: Some("alice@example.com".to_string()),
//! };
//!
//! // UserCreate - for creating new users
//! let user_create = UserCreate {
//!     name: "Bob".to_string(),
//!     email: Some("bob@example.com".to_string()),
//! };
//!
//! // UserPatch - for updating users
//! let user_patch = UserPatch {
//!     name: Patch::Update("Charlie".to_string()), // Update the name
//!     email: Patch::Ignore,                       // Don't change email
//! };
//! ```
//!
//! ## Nested Models
//!
//! Views work seamlessly with nested structures:
//!
//! ```rust
//! # use model_views::{Views, Patch};
//! # #[derive(Views)]
//! # #[views(serde)]
//! # struct User {
//! #     // ID is returned when reading, but can't be set during create/update
//! #     #[views(get = "required", create = "forbidden", patch = "forbidden")]
//! #     id: u64,
//! #
//! #     // Name is always required for all operations
//! #     #[views(get = "required", create = "required", patch = "patch")]
//! #     name: String,
//! #
//! #     // Email is optional everywhere
//! #     #[views(get = "optional", create = "optional", patch = "optional")]
//! #     email: String,
//! # }
//! #[derive(Views)]
//! struct Post {
//!     #[views(get = "required", create = "forbidden", patch = "forbidden")]
//!     id: u64,
//!     
//!     #[views(get = "required")]
//!     title: String,
//!     
//!     // Nested models are automatically handled
//!     #[views(get = "required", create = "forbidden", patch = "optional")]
//!     author: User,
//! }
//!
//! // PostPatch will have: author: Patch<Option<UserPatch>>
//! let post_patch = PostPatch {
//!     title: Patch::Update("New Title".to_string()),
//!     author: Patch::Update(Some(UserPatch {
//!         name: Patch::Update("New Author Name".to_string()),
//!         email: Patch::Ignore,
//!     })),
//! };
//! ```
//!
//! ## Field Policies
//!
//! Each field can be configured independently for each view mode:
//!
//! - `get = "required"`: Field is always present in Get view
//! - `get = "optional"`: Field is `Option<T>` in Get view  
//! - `get = "forbidden"`: Field is excluded from Get view
//!
//! - `create = "required"`: Field must be provided when creating
//! - `create = "optional"`: Field is `Option<T>` in Create view
//! - `create = "forbidden"`: Field cannot be set during creation
//!
//! - `patch = "patch"`: Field is `Patch<T>` in Patch view
//! - `patch = "optional"`: Field is `Patch<Option<T>>` in Patch view
//! - `patch = "forbidden"`: Field cannot be modified via patches
//!
//! # Features
//!
//! - **`derive`** (default): Enables the `#[derive(Views)]` procedural macro
//! - **`serde`**: Adds `Serialize`/`Deserialize` support for `Patch<T>`
//! - **`uuid`**: Implements `View` for `uuid::Uuid`
//! - **`chrono`**: Implements `View` for `chrono::DateTime<Utc>`
//!
//! # Benefits
//!
//! - **Type Safety**: Different operations use different types, catching errors at compile time
//! - **API Clarity**: View types clearly document which fields are required/optional for each operation
//! - **Reduced Boilerplate**: Automatically generates DTOs (Data Transfer Objects) from models
//! - **Explicit Updates**: `Patch<T>` makes update intent clear, avoiding ambiguity with `Option<T>`

#![forbid(unsafe_code)]

mod patch;

pub use patch::*;

#[cfg(feature = "derive")]
pub use model_views_derive::Views;

pub trait View<M: ViewMode> {
    type Type;
}

/// Access mode for a model.
pub trait ViewMode {}

/// Read access for a model.
pub struct ViewModeGet;
impl ViewMode for ViewModeGet {}

/// Create access for a model.
pub struct ViewModeCreate;
impl ViewMode for ViewModeCreate {}

/// Update/Write access for a model.
pub struct ViewModePatch;
impl ViewMode for ViewModePatch {}

// Trivials just map to themselves for any mode
macro_rules! trivial_view {
    ($($t:ty),* $(,)?) => {$(
        impl<M: $crate::ViewMode> $crate::View<M> for $t { type Type = $t; }
    )*}
}

trivial_view!(
    bool,
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    i128,
    u128,
    f32,
    f64,
    String,
    &'static str
);

#[cfg(feature = "uuid")]
trivial_view!(uuid::Uuid);

#[cfg(feature = "chrono")]
trivial_view!(chrono::DateTime<chrono::Utc>);
