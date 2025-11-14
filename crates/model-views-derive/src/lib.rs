//! Procedural macro for deriving view types from models.
//!
//! This crate provides the `#[derive(Views)]` macro that automatically generates
//! specialized view types for different access modes (Get, Create, Patch) from a
//! base model struct.
//!
//! # Overview
//!
//! The `Views` derive macro generates up to three view types for a model:
//!
//! - **`{Model}Get`**: A read-only view for retrieving data
//! - **`{Model}Create`**: A view for creating new instances
//! - **`{Model}Patch`**: A view for partial updates using the `Patch<T>` wrapper
//!
//! Each generated type only includes fields relevant to its access mode, based on
//! field-level attributes that specify visibility policies.
//!
//! # Field Policies
//!
//! Control field visibility in each view using these attributes:
//!
//! - `#[views(get = "policy")]`: Controls field visibility in the Get view
//!   - `"required"` (default): Field is always present
//!   - `"optional"`: Field is wrapped in `Option<T>`
//!   - `"forbidden"`: Field is excluded from this view
//!
//! - `#[views(create = "policy")]`: Controls field visibility in the Create view
//!   - `"required"` (default): Field must be provided
//!   - `"optional"`: Field is wrapped in `Option<T>`
//!   - `"forbidden"`: Field is excluded from this view
//!
//! - `#[views(patch = "policy")]`: Controls field visibility in the Patch view
//!   - `"patch"` (default): Field is wrapped in `Patch<T>`
//!   - `"optional"`: Field is wrapped in `Patch<Option<T>>`
//!   - `"forbidden"`: Field is excluded from this view
//!
//! # Container Attributes
//!
//! - `#[views(crate = "path")]`: Override the path to the `model_views` crate
//! - `#[views(serde)]`: Automatically derive `Serialize`/`Deserialize` for generated types
//!
//! # Example
//!
//! ```rust,ignore
//! #[derive(Views)]
//! #[views(serde)]
//! struct User {
//!     #[views(get = "required", create = "forbidden", patch = "forbidden")]
//!     id: i64,
//!     
//!     #[views(get = "required", create = "required", patch = "patch")]
//!     name: String,
//!     
//!     #[views(get = "optional", create = "optional", patch = "optional")]
//!     email: String,
//! }
//! ```
//!
//! This generates:
//! - `UserGet` with `id: i64`, `name: String`, `email: Option<Option<String>>`
//! - `UserCreate` with `name: String`, `email: Option<Option<String>>`
//! - `UserPatch` with `name: Patch<String>`, `email: Patch<Option<String>>`

#![allow(clippy::option_if_let_else)]

use darling::{FromDeriveInput, FromField, util::Ignored};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Type, parse_macro_input};

const BASE_CRATE: &str = "model_views";

#[derive(FromDeriveInput)]
#[darling(attributes(views))]
struct ViewsInput {
    ident: syn::Ident,
    vis: syn::Visibility,
    generics: syn::Generics,
    data: darling::ast::Data<Ignored, ViewsField>,
    /// Path (string) to base crate, e.g. "`model_views`"
    #[darling(default)]
    crate_: Option<String>,
    /// Whether to derive serde traits for the generated types
    #[darling(default)]
    serde: Option<bool>,
}

#[derive(FromField, Clone)]
#[darling(attributes(views))]
struct ViewsField {
    ident: Option<syn::Ident>,
    ty: Type,
    #[darling(default)]
    get: Option<String>,
    #[darling(default)]
    create: Option<String>,
    #[darling(default)]
    patch: Option<String>,
}

/// Derives view types for different access modes from a model struct.
///
/// This procedural macro generates up to three specialized view types based on the
/// annotated model:
///
/// - `{Model}Get`: For read/retrieval operations
/// - `{Model}Create`: For creation operations
/// - `{Model}Patch`: For update/modification operations
///
/// # Generated Types
///
/// For a struct named `User`, the macro generates:
/// - `UserGet` with appropriate `Serialize` derives (if serde enabled)
/// - `UserCreate` with appropriate `Deserialize` derives (if serde enabled)
/// - `UserPatch` with `Default` and `Deserialize` derives (if serde enabled)
///
/// Each generated type implements `View<ViewMode{Get,Create,Patch}>` for the original type,
/// allowing generic code to work with different view modes.
///
/// # Container Attributes
///
/// The `#[views(...)]` attribute on the struct itself accepts:
///
/// - `crate = "path"`: Override the path to the `model_views` crate. Useful when
///   re-exporting or when the crate is available under a different name.
///   
///   ```rust,ignore
///   #[derive(Views)]
///   #[views(crate = "my_models::views")]
///   struct User { /* ... */ }
///   ```
///
/// - `serde` or `serde = true`: Automatically derive `Serialize` for Get views and
///   `Deserialize` for Create and Patch views. Also adds `deny_unknown_fields` and
///   appropriate field-level serde attributes.
///   
///   ```rust,ignore
///   #[derive(Views)]
///   #[views(serde)]
///   struct User { /* ... */ }
///   ```
///
/// # Field Attributes
///
/// Each field can be independently configured for each view mode using `#[views(...)]`:
///
/// ## Get Mode (`get = "policy"`)
///
/// Controls how the field appears in the `{Model}Get` type:
/// - `"required"` (default): Field is always present with its view type
/// - `"optional"`: Field is wrapped in `Option<T>`
/// - `"forbidden"`: Field is excluded from the Get view
///
/// ## Create Mode (`create = "policy"`)
///
/// Controls how the field appears in the `{Model}Create` type:
/// - `"required"` (default): Field must be provided during creation
/// - `"optional"`: Field is wrapped in `Option<T>`, with serde's `default` and
///   `skip_serializing_if` attributes when serde is enabled
/// - `"forbidden"`: Field is excluded from the Create view
///
/// ## Patch Mode (`patch = "policy"`)
///
/// Controls how the field appears in the `{Model}Patch` type:
/// - `"patch"` (default): Field is wrapped in `Patch<T>`, allowing explicit ignore/update
/// - `"optional"`: Field is wrapped in `Patch<Option<T>>`
/// - `"forbidden"`: Field is excluded from the Patch view
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,ignore
/// use model_views::Views;
///
/// #[derive(Views)]
/// struct Article {
///     // ID only appears in Get view (auto-generated, can't be set)
///     #[views(get = "required", create = "forbidden", patch = "forbidden")]
///     id: u64,
///     
///     // Title is required everywhere
///     #[views(get = "required", create = "required", patch = "patch")]
///     title: String,
///     
///     // Published status can be patched
///     #[views(get = "required", create = "optional", patch = "patch")]
///     published: bool,
/// }
/// ```
///
/// This generates:
/// - `ArticleGet { id: u64, title: String, published: bool }`
/// - `ArticleCreate { title: String, published: Option<bool> }`
/// - `ArticlePatch { title: Patch<String>, published: Patch<bool> }`
///
/// ## With Serde Support
///
/// ```rust,ignore
/// #[derive(Views)]
/// #[views(serde)]
/// struct User {
///     #[views(get = "required", create = "forbidden", patch = "forbidden")]
///     id: u64,
///     #[views(get = "required", create = "required", patch = "patch")]
///     name: String,
/// }
/// ```
///
/// The generated types will have appropriate `Serialize`/`Deserialize` derives.
///
/// ## Generic Types
///
/// The macro supports generic parameters:
///
/// ```rust,ignore
/// #[derive(Views)]
/// struct Container<T> {
///     #[views(get = "required", create = "required", patch = "patch")]
///     value: T,
/// }
/// ```
///
/// # Panics
///
/// The macro will panic at compile time if:
/// - Applied to an enum or union (only structs with named fields are supported)
/// - An unknown policy value is used (e.g., `get = "invalid"`)
/// - The `crate` attribute contains an invalid path
///
/// # Implementation Details
///
/// - View types only include fields that have at least one non-forbidden policy
/// - If all fields are forbidden for a view mode, that view type is still generated
///   (as an empty struct)
/// - Generated types preserve the original struct's visibility and generic parameters
/// - Non-`#[views(...)]` attributes from the original struct are copied to generated types
/// - When serde is enabled, optional create fields get `#[serde(default, skip_serializing_if = "Option::is_none")]`
#[proc_macro_derive(Views, attributes(views, view))]
#[allow(clippy::missing_panics_doc,clippy::cognitive_complexity,clippy::too_many_lines)]
pub fn derive_views(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let meta = ViewsInput::from_derive_input(&input).expect("parse #[derive(Views)]");

    let crate_path: syn::Path = if let Some(s) = &meta.crate_ {
        syn::parse_str(s).expect("valid path in #[views(crate = \"...\")]")
    } else {
        syn::parse_str(BASE_CRATE).unwrap()
    };

    let with_serde = meta.serde.unwrap_or(false);

    let name = &meta.ident;
    let (impl_generics, ty_generics, where_clause) = meta.generics.split_for_impl();

    let create_ident = format_ident!("{name}Create");
    let read_ident = format_ident!("{name}Get");
    let patch_ident = format_ident!("{name}Patch");

    let mut create_fields = Vec::new();
    let mut read_fields = Vec::new();
    let mut patch_fields = Vec::new();

    // Track whether a given mode actually has any fields
    let mut has_get = false;
    let mut has_create = false;
    let mut has_patch = false;

    let mv_view = quote!(#crate_path::View);
    let mv_get = quote!(#crate_path::ViewModeGet);
    let mv_create = quote!(#crate_path::ViewModeCreate);
    let mv_patch = quote!(#crate_path::ViewModePatch);
    let mv_patch_t = quote!(#crate_path::Patch);

    if let darling::ast::Data::Struct(ds) = &meta.data {
        for f in &ds.fields {
            let ident = f.ident.clone().expect("named fields only");
            let fty = &f.ty;

            // policies with defaults
            let get_p = f.get.as_deref().unwrap_or("required");
            let crt_p = f.create.as_deref().unwrap_or("required");
            let patch_p = f.patch.as_deref().unwrap_or("patch");

            // ---- GET / READ ----
            match get_p {
                "required" => {
                    has_get = true;
                    read_fields.push(quote! { pub #ident: <#fty as #mv_view<#mv_get>>::Type, });
                }
                "optional" => {
                    has_get = true;
                    read_fields.push(quote! {
                        pub #ident: ::core::option::Option<<#fty as #mv_view<#mv_get>>::Type>,
                    });
                }
                "forbidden" => {}
                other => panic!("unknown get policy: {other}"),
            }

            // ---- CREATE ----
            match crt_p {
                "required" => {
                    has_create = true;
                    create_fields.push(quote! {
                        pub #ident: <#fty as #mv_view<#mv_create>>::Type,
                    });
                }
                "optional" => {
                    has_create = true;
                    if with_serde {
                        create_fields.push(quote! {
                            #[serde(default, skip_serializing_if = "Option::is_none")]
                        });
                    }
                    create_fields.push(quote! {
                        pub #ident: ::core::option::Option<<#fty as #mv_view<#mv_create>>::Type>,
                    });
                }
                "forbidden" => {}
                other => panic!("unknown create policy: {other}"),
            }

            // ---- PATCH ----
            match patch_p {
                "patch" => {
                    has_patch = true;
                    patch_fields.push(quote! {
                        pub #ident: #mv_patch_t<<#fty as #mv_view<#mv_patch>>::Type>,
                    });
                }
                "optional" => {
                    has_patch = true;
                    patch_fields.push(quote! {
                        pub #ident: #mv_patch_t<::core::option::Option<<#fty as #mv_view<#mv_patch>>::Type>>,
                    });
                }
                "forbidden" => {}
                other => panic!("unknown patch policy: {other}"),
            }
        }
    } else {
        panic!("#[derive(Views)] supports struct with named fields only");
    }

    // pull locals for quote!
    let vis = &meta.vis;
    let struct_attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("views"))
        .collect();
    let create_ident = &create_ident;
    let read_ident = &read_ident;
    let patch_ident = &patch_ident;

    let create_fields_ts = &create_fields;
    let read_fields_ts = &read_fields;
    let patch_fields_ts = &patch_fields;

    // Build items conditionally
    let mut items = Vec::<proc_macro2::TokenStream>::new();

    let serialize_attrs = if with_serde {
        quote! {
            #[derive(::serde::Serialize)]
            #[serde(deny_unknown_fields)]
        }
    } else {
        quote! {}
    };

    let deserialize_attrs = if with_serde {
        quote! {
            #[derive(::serde::Deserialize)]
            #[serde(deny_unknown_fields)]
        }
    } else {
        quote! {}
    };

    if has_create {
        items.push(quote! {
            #deserialize_attrs
            #(#struct_attrs)*
            #vis struct #create_ident #ty_generics
            #where_clause
            {
                #(#create_fields_ts)*
            }

            impl #impl_generics #mv_view<#mv_create> for #name #ty_generics #where_clause {
                type Type = #create_ident #ty_generics;
            }
        });
    }

    if has_get {
        items.push(quote! {
            #serialize_attrs
            #(#struct_attrs)*
            #vis struct #read_ident #ty_generics
            #where_clause
            {
                #(#read_fields_ts)*
            }

            impl #impl_generics #mv_view<#mv_get> for #name #ty_generics #where_clause {
                type Type = #read_ident #ty_generics;
            }
        });
    }

    if has_patch {
        items.push(quote! {
            #[derive(::core::default::Default)]
            #deserialize_attrs
            #(#struct_attrs)*
            #vis struct #patch_ident #ty_generics
            #where_clause
            {
                #(#patch_fields_ts)*
            }

            impl #impl_generics #mv_view<#mv_patch> for #name #ty_generics #where_clause {
                type Type = #patch_ident #ty_generics;
            }
        });
    }

    let out = quote! { #(#items)* };
    out.into()
}
