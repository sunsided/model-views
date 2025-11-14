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
