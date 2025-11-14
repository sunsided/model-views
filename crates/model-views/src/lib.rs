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
