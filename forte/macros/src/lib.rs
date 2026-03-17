//! # Integer Key Formatting for PK/SK
//!
//! When integer types are used as PK (Partition Key) or SK (Sort Key) fields,
//! they are zero-padded to their maximum decimal digit width so that
//! lexicographic (string) sort order matches numeric sort order.
//!
//! ## Unsigned types
//!
//! | Type          | Width | Example              |
//! |---------------|-------|----------------------|
//! | `u8`          | 3     | `042`                |
//! | `u16`         | 5     | `00042`              |
//! | `u32`         | 10    | `0000000042`         |
//! | `u64`/`usize` | 20   | `00000000000000000042` |
//!
//! ## Signed types (offset encoding)
//!
//! Signed integers are converted to unsigned by adding an offset equal to
//! `|T::MIN|` (i.e. `2^(bits-1)`), then zero-padded to the same width as the
//! corresponding unsigned type. This maps the full signed range onto `0..=U::MAX`
//! while preserving numeric order.
//!
//! | Type          | Offset           | Width | MIN → | 0 →  | MAX →  |
//! |---------------|------------------|-------|-------|------|--------|
//! | `i8`          | 128              | 3     | `000` | `128` | `255` |
//! | `i16`         | 32768            | 5     | `00000` | `32768` | `65535` |
//! | `i32`         | 2147483648       | 10    | `0000000000` | `2147483648` | `4294967295` |
//! | `i64`/`isize` | 9223372036854775808 | 20 | `00000000000000000000` | `09223372036854775808` | `18446744073709551615` |
//!
//! `usize` is always treated as `u64`, and `isize` as `i64`.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Fields, ItemStruct, LitStr};

fn format_placeholder(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "u8" | "i8" => return "{:03}".to_string(),
                "u16" | "i16" => return "{:05}".to_string(),
                "u32" | "i32" => return "{:010}".to_string(),
                "u64" | "i64" | "usize" | "isize" => return "{:020}".to_string(),
                _ => {}
            }
        }
    }
    "{}".to_string()
}

fn wrap_expr(ty: &syn::Type, expr: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "i8" => return quote! { (#expr as u8).wrapping_add(128u8) },
                "i16" => return quote! { (#expr as u16).wrapping_add(32768u16) },
                "i32" => return quote! { (#expr as u32).wrapping_add(2147483648u32) },
                "i64" | "isize" => return quote! { (#expr as u64).wrapping_add(9223372036854775808u64) },
                _ => {}
            }
        }
    }
    expr
}

#[proc_macro_attribute]
pub fn forte_doc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let vis = &input.vis;
    let get_name = format_ident!("{}Get", name);
    let query_name = format_ident!("{}Query", name);

    let fields = match &input.fields {
        Fields::Named(fields) => &fields.named,
        _ => panic!("forte_doc only supports named fields"),
    };

    let pk_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("pk")))
        .collect();

    let sk_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("sk")))
        .collect();

    let pk_field_names: Vec<_> = pk_fields.iter().map(|f| &f.ident).collect();
    let pk_field_types: Vec<_> = pk_fields.iter().map(|f| &f.ty).collect();
    let sk_field_names: Vec<_> = sk_fields.iter().map(|f| &f.ident).collect();
    let sk_field_types: Vec<_> = sk_fields.iter().map(|f| &f.ty).collect();

    let get_pk_fields: Vec<_> = pk_field_names
        .iter()
        .zip(pk_field_types.iter())
        .map(|(name, ty)| {
            let field_name = format_ident!("pk_{}", name.as_ref().unwrap());
            quote! { pub #field_name: #ty }
        })
        .collect();

    let get_sk_fields: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(name, ty)| {
            let field_name = format_ident!("sk_{}", name.as_ref().unwrap());
            quote! { pub #field_name: #ty }
        })
        .collect();

    let query_pk_fields: Vec<_> = pk_field_names
        .iter()
        .zip(pk_field_types.iter())
        .map(|(name, ty)| {
            let field_name = format_ident!("pk_{}", name.as_ref().unwrap());
            quote! { pub #field_name: #ty }
        })
        .collect();

    let query_sk_fields: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(name, ty)| {
            let field_name = format_ident!("sk_{}", name.as_ref().unwrap());
            quote! { pub #field_name: Option<#ty> }
        })
        .collect();

    let query_pk_str = if pk_fields.is_empty() {
        let name_str = name.to_string();
        quote! { #name_str.to_string() }
    } else {
        let name_str = name.to_string();
        let pk_format_parts: Vec<_> = pk_field_names
            .iter()
            .zip(pk_field_types.iter())
            .map(|(n, ty)| {
                let name_str = n.as_ref().unwrap().to_string();
                format!("{}={}", name_str, format_placeholder(ty))
            })
            .collect();
        let pk_format_string = format!("{}/{}", name_str, pk_format_parts.join("&"));
        let pk_format_args: Vec<_> = pk_field_names
            .iter()
            .zip(pk_field_types.iter())
            .map(|(n, ty)| {
                let field_name = format_ident!("pk_{}", n.as_ref().unwrap());
                wrap_expr(ty, quote! { self.#field_name })
            })
            .collect();
        quote! { format!(#pk_format_string, #(#pk_format_args),*) }
    };

    let query_sk_build: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(n, ty)| {
            let name_str = n.as_ref().unwrap().to_string();
            let field_name = format_ident!("sk_{}", n.as_ref().unwrap());
            let fmt = format!("{}={}", name_str, format_placeholder(ty));
            let val_expr = wrap_expr(ty, quote! { *v });
            quote! {
                if let Some(v) = &self.#field_name {
                    parts.push(format!(#fmt, #val_expr));
                } else {
                    break 'build;
                }
            }
        })
        .collect();

    let pk_str = if pk_fields.is_empty() {
        let name_str = name.to_string();
        quote! { #name_str.to_string() }
    } else {
        let name_str = name.to_string();
        let pk_format_parts: Vec<_> = pk_field_names
            .iter()
            .zip(pk_field_types.iter())
            .map(|(n, ty)| {
                let name_str = n.as_ref().unwrap().to_string();
                format!("{}={}", name_str, format_placeholder(ty))
            })
            .collect();
        let pk_format_string = format!("{}/{}", name_str, pk_format_parts.join("&"));
        let pk_format_args: Vec<_> = pk_field_names
            .iter()
            .zip(pk_field_types.iter())
            .map(|(n, ty)| {
                let field_name = format_ident!("pk_{}", n.as_ref().unwrap());
                wrap_expr(ty, quote! { self.#field_name })
            })
            .collect();
        quote! { format!(#pk_format_string, #(#pk_format_args),*) }
    };

    let sk_format_parts: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(n, ty)| {
            let name_str = n.as_ref().unwrap().to_string();
            format!("{}={}", name_str, format_placeholder(ty))
        })
        .collect();
    let sk_format_string = sk_format_parts.join("&");
    let sk_format_args: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(n, ty)| {
            let field_name = format_ident!("sk_{}", n.as_ref().unwrap());
            wrap_expr(ty, quote! { self.#field_name })
        })
        .collect();

    let put_pk_str = if pk_fields.is_empty() {
        let name_str = name.to_string();
        quote! { #name_str.to_string() }
    } else {
        let name_str = name.to_string();
        let pk_format_parts: Vec<_> = pk_field_names
            .iter()
            .zip(pk_field_types.iter())
            .map(|(n, ty)| {
                let name_str = n.as_ref().unwrap().to_string();
                format!("{}={}", name_str, format_placeholder(ty))
            })
            .collect();
        let pk_format_string = format!("{}/{}", name_str, pk_format_parts.join("&"));
        let pk_format_args: Vec<_> = pk_field_names
            .iter()
            .zip(pk_field_types.iter())
            .map(|(n, ty)| {
                let field_name = n.as_ref().unwrap();
                wrap_expr(ty, quote! { self.#field_name })
            })
            .collect();
        quote! { format!(#pk_format_string, #(#pk_format_args),*) }
    };

    let put_sk_format_args: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(n, ty)| {
            let field_name = n.as_ref().unwrap();
            wrap_expr(ty, quote! { self.#field_name })
        })
        .collect();

    let clean_fields: Vec<_> = fields
        .iter()
        .map(|f| {
            let mut f = f.clone();
            f.attrs.retain(|a| !a.path().is_ident("pk") && !a.path().is_ident("sk"));
            f
        })
        .collect();

    let expanded = quote! {
        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        #vis struct #name {
            #(#clean_fields,)*
        }

        impl #name {
            pub async fn put(&self) -> anyhow::Result<()> {
                let pk = #put_pk_str;
                let sk = format!(#sk_format_string, #(#put_sk_format_args),*);
                forte_db::turso()
                    .put(&pk, &sk, &serde_json::to_vec(self)?)
                    .await
            }

            pub async fn query_next(&self, limit: usize) -> anyhow::Result<Vec<Self>> {
                let pk = #put_pk_str;
                let sk = format!(#sk_format_string, #(#put_sk_format_args),*);
                Ok(forte_db::turso()
                    .query(&pk, Some(&sk), limit)
                    .await?
                    .into_iter()
                    .map(|(_sk, data)| serde_json::from_slice(&data))
                    .collect::<Result<Vec<_>, _>>()?)
            }
        }

        #vis struct #get_name {
            #(#get_pk_fields,)*
            #(#get_sk_fields,)*
        }

        impl #get_name {
            pub async fn send(self) -> anyhow::Result<Option<#name>> {
                let pk = #pk_str;
                let sk = format!(#sk_format_string, #(#sk_format_args),*);
                Ok(forte_db::turso()
                    .get(&pk, &sk)
                    .await?
                    .map(|data| serde_json::from_slice(&data))
                    .transpose()?)
            }
        }

        #vis struct #query_name {
            #(#query_pk_fields,)*
            #(#query_sk_fields,)*
        }

        impl #query_name {
            pub async fn send(self, limit: usize) -> anyhow::Result<Vec<#name>> {
                let pk = #query_pk_str;
                let after_sk: Option<String> = {
                    let mut parts: Vec<String> = Vec::new();
                    'build: {
                        #(#query_sk_build)*
                    }
                    if parts.is_empty() { None } else { Some(parts.join("&")) }
                };
                Ok(forte_db::turso()
                    .query(&pk, after_sk.as_deref(), limit)
                    .await?
                    .into_iter()
                    .map(|(_sk, data)| serde_json::from_slice(&data))
                    .collect::<Result<Vec<_>, _>>()?)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn doc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let pk_value = parse_macro_input!(attr as LitStr).value();
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let vis = &input.vis;
    let pk_name = format_ident!("{}Pk", name);
    let sk_name = format_ident!("{}Sk", name);

    let fields = match &input.fields {
        Fields::Named(fields) => &fields.named,
        _ => panic!("doc attribute only supports named fields"),
    };

    let sk_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("sk")))
        .collect();

    let sk_field_names: Vec<_> = sk_fields.iter().map(|f| &f.ident).collect();
    let sk_field_types: Vec<_> = sk_fields.iter().map(|f| &f.ty).collect();

    let sk_format_parts: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(n, ty)| {
            let name_str = n.as_ref().unwrap().to_string();
            format!("{}={}", name_str, format_placeholder(ty))
        })
        .collect();
    let sk_format_string = sk_format_parts.join(",");

    let sk_format_args: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(n, ty)| {
            let ident = n.as_ref().unwrap();
            wrap_expr(ty, quote! { sk.#ident })
        })
        .collect();

    let clean_fields: Vec<_> = fields
        .iter()
        .map(|f| {
            let mut f = f.clone();
            f.attrs.retain(|a| !a.path().is_ident("sk"));
            f
        })
        .collect();

    let expanded = quote! {
        #vis struct #pk_name;

        #vis struct #sk_name {
            #(pub #sk_field_names: #sk_field_types,)*
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        #vis struct #name {
            #(#clean_fields,)*
        }

        impl forte_sdk::Doc for #name {
            type Pk = #pk_name;
            type Sk = #sk_name;

            fn pk(_pk: Self::Pk) -> String {
                #pk_value.to_string()
            }

            fn sk(sk: Self::Sk) -> String {
                format!(#sk_format_string, #(#sk_format_args),*)
            }
        }
    };

    TokenStream::from(expanded)
}
