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
use quote::{format_ident, quote, quote_spanned};
use syn::{Fields, ItemFn, ItemStruct, parse_macro_input, spanned::Spanned};

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    if input.sig.asyncness.is_none() {
        return quote_spanned! { input.sig.fn_token.span()=>
            compile_error!("fn must be `async fn`");
        }
        .into();
    }

    if !input.sig.inputs.is_empty() {
        return quote_spanned! { input.sig.inputs.span()=>
            compile_error!("arguments to test functions are not supported");
        }
        .into();
    }

    let name = input.sig.ident;
    let attrs = input.attrs;
    let output = input.sig.output;
    let block = input.block;
    quote! {
        #[::core::prelude::v1::test]
        pub fn #name() #output {
            #(#attrs)*
            async fn __run() #output {
                #block
            }
            ::forte_sdk::runtime::block_on(async { __run().await })
        }
    }
    .into()
}

fn format_placeholder(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        match segment.ident.to_string().as_str() {
            "u8" | "i8" => return "{:03}".to_string(),
            "u16" | "i16" => return "{:05}".to_string(),
            "u32" | "i32" => return "{:010}".to_string(),
            "u64" | "i64" | "usize" | "isize" => return "{:020}".to_string(),
            _ => {}
        }
    }
    "{}".to_string()
}

fn wrap_expr(ty: &syn::Type, expr: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        match segment.ident.to_string().as_str() {
            "i8" => return quote! { (#expr as u8).wrapping_add(128u8) },
            "i16" => return quote! { (#expr as u16).wrapping_add(32768u16) },
            "i32" => return quote! { (#expr as u32).wrapping_add(2147483648u32) },
            "i64" | "isize" => {
                return quote! { (#expr as u64).wrapping_add(9223372036854775808u64) };
            }
            _ => {}
        }
    }
    expr
}

fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "String";
    }
    false
}

fn make_generics(
    pk_is_string: &[bool],
    sk_is_string: &[bool],
) -> (
    Vec<Option<proc_macro2::Ident>>,
    Vec<Option<proc_macro2::Ident>>,
) {
    let mut counter = 0usize;
    let pk = pk_is_string
        .iter()
        .map(|&s| {
            if s {
                let ident = format_ident!("__T{}", counter);
                counter += 1;
                Some(ident)
            } else {
                None
            }
        })
        .collect();
    let sk = sk_is_string
        .iter()
        .map(|&s| {
            if s {
                let ident = format_ident!("__T{}", counter);
                counter += 1;
                Some(ident)
            } else {
                None
            }
        })
        .collect();
    (pk, sk)
}

#[proc_macro_attribute]
pub fn forte_doc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;
    let vis = &input.vis;
    let get_name = format_ident!("{}Get", name);
    let put_name = format_ident!("{}Put", name);
    let query_name = format_ident!("{}Query", name);
    let delete_name = format_ident!("{}Delete", name);

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

    let pk_is_string: Vec<bool> = pk_field_types.iter().map(|ty| is_string_type(ty)).collect();
    let sk_is_string: Vec<bool> = sk_field_types.iter().map(|ty| is_string_type(ty)).collect();

    let (gpk, gsk) = make_generics(&pk_is_string, &sk_is_string);
    let all_generics: Vec<_> = gpk
        .iter()
        .chain(gsk.iter())
        .filter_map(|g| g.as_ref())
        .collect();

    let generic_def = if all_generics.is_empty() {
        quote! {}
    } else {
        quote! { <#(#all_generics: AsRef<str>),*> }
    };
    let generic_use = if all_generics.is_empty() {
        quote! {}
    } else {
        quote! { <#(#all_generics),*> }
    };

    let query_generics: Vec<_> = gpk.iter().filter_map(|g| g.as_ref()).collect();
    let query_generic_def = if query_generics.is_empty() {
        quote! {}
    } else {
        quote! { <#(#query_generics: AsRef<str>),*> }
    };
    let query_generic_use = if query_generics.is_empty() {
        quote! {}
    } else {
        quote! { <#(#query_generics),*> }
    };

    let get_pk_fields: Vec<_> = pk_field_names
        .iter()
        .zip(pk_field_types.iter())
        .zip(gpk.iter())
        .map(|((name, ty), gp)| {
            let field_name = name.as_ref().unwrap();
            if let Some(g) = gp {
                quote! { pub #field_name: #g }
            } else {
                quote! { pub #field_name: #ty }
            }
        })
        .collect();

    let get_sk_fields: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .zip(gsk.iter())
        .map(|((name, ty), gp)| {
            let field_name = name.as_ref().unwrap();
            if let Some(g) = gp {
                quote! { pub #field_name: #g }
            } else {
                quote! { pub #field_name: #ty }
            }
        })
        .collect();

    let query_pk_fields: Vec<_> = pk_field_names
        .iter()
        .zip(pk_field_types.iter())
        .zip(gpk.iter())
        .map(|((name, ty), gp)| {
            let field_name = name.as_ref().unwrap();
            if let Some(g) = gp {
                quote! { pub #field_name: #g }
            } else {
                quote! { pub #field_name: #ty }
            }
        })
        .collect();

    let query_sk_fields: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(name, ty)| {
            let field_name = name.as_ref().unwrap();
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
            .zip(pk_is_string.iter())
            .map(|((n, ty), &is_str)| {
                let field_name = n.as_ref().unwrap();
                if is_str {
                    quote! { self.#field_name.as_ref() }
                } else {
                    wrap_expr(ty, quote! { self.#field_name })
                }
            })
            .collect();
        quote! { format!(#pk_format_string, #(#pk_format_args),*) }
    };

    let query_sk_build: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(n, ty)| {
            let name_str = n.as_ref().unwrap().to_string();
            let field_name = n.as_ref().unwrap();
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
            .zip(pk_is_string.iter())
            .map(|((n, ty), &is_str)| {
                let field_name = n.as_ref().unwrap();
                if is_str {
                    quote! { self.#field_name.as_ref() }
                } else {
                    wrap_expr(ty, quote! { self.#field_name })
                }
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
        .zip(sk_is_string.iter())
        .map(|((n, ty), &is_str)| {
            let field_name = n.as_ref().unwrap();
            if is_str {
                quote! { self.#field_name.as_ref() }
            } else {
                wrap_expr(ty, quote! { self.#field_name })
            }
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
                wrap_expr(ty, quote! { self.0.#field_name })
            })
            .collect();
        quote! { format!(#pk_format_string, #(#pk_format_args),*) }
    };

    let doc_pk_str = if pk_fields.is_empty() {
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
            .zip(pk_is_string.iter())
            .map(|((n, ty), &is_str)| {
                let field_name = n.as_ref().unwrap();
                if is_str {
                    quote! { self.#field_name.as_str() }
                } else {
                    wrap_expr(ty, quote! { self.#field_name })
                }
            })
            .collect();
        quote! { format!(#pk_format_string, #(#pk_format_args),*) }
    };

    let put_sk_format_args: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .map(|(n, ty)| {
            let field_name = n.as_ref().unwrap();
            wrap_expr(ty, quote! { self.0.#field_name })
        })
        .collect();

    let doc_sk_format_args: Vec<_> = sk_field_names
        .iter()
        .zip(sk_field_types.iter())
        .zip(sk_is_string.iter())
        .map(|((n, ty), &is_str)| {
            let field_name = n.as_ref().unwrap();
            if is_str {
                quote! { self.#field_name.as_str() }
            } else {
                wrap_expr(ty, quote! { self.#field_name })
            }
        })
        .collect();

    let clean_fields: Vec<_> = fields
        .iter()
        .map(|f| {
            let mut f = f.clone();
            f.attrs
                .retain(|a| !a.path().is_ident("pk") && !a.path().is_ident("sk"));
            f
        })
        .collect();

    let expanded = quote! {
        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        #vis struct #name {
            #(#clean_fields,)*
        }

        impl forte_db::Document for #name {
            fn key(&self) -> forte_db::DocKey {
                let pk = #doc_pk_str;
                let sk = format!(#sk_format_string, #(#doc_sk_format_args),*);
                forte_db::DocKey::new(pk, sk)
            }
        }

        #vis struct #put_name(pub #name);

        impl forte_db::DbRequest for #put_name {
            type Output = ();
            fn prepare(self) -> forte_db::Prepared<Self::Output> {
                let pk = #put_pk_str;
                let sk = format!(#sk_format_string, #(#put_sk_format_args),*);
                let data = serde_json::to_vec(&self.0).expect("failed to serialize");
                forte_db::Prepared {
                    ops: vec![forte_db::DbOp::Put { pk, sk, data }],
                    parse: Box::new(|iter| {
                        match iter.next().ok_or_else(|| anyhow::anyhow!("missing result"))? {
                            forte_db::DbResult::Done => Ok(()),
                            _ => anyhow::bail!("unexpected result type"),
                        }
                    }),
                }
            }
        }

        #vis struct #get_name #generic_def {
            #(#get_pk_fields,)*
            #(#get_sk_fields,)*
        }

        impl #generic_def forte_db::DocGet for #get_name #generic_use {
            type Doc = #name;

            fn key(&self) -> forte_db::DocKey {
                let pk = #pk_str;
                let sk = format!(#sk_format_string, #(#sk_format_args),*);
                forte_db::DocKey::new(pk, sk)
            }
        }

        impl #generic_def forte_db::DbRequest for #get_name #generic_use {
            type Output = Option<#name>;
            fn prepare(self) -> forte_db::Prepared<Self::Output> {
                let pk = #pk_str;
                let sk = format!(#sk_format_string, #(#sk_format_args),*);
                forte_db::Prepared {
                    ops: vec![forte_db::DbOp::Get { pk, sk }],
                    parse: Box::new(|iter| {
                        match iter.next().ok_or_else(|| anyhow::anyhow!("missing result"))? {
                            forte_db::DbResult::Single(opt) => {
                                opt.map(|data| serde_json::from_slice(&data))
                                    .transpose()
                                    .map_err(Into::into)
                            }
                            _ => anyhow::bail!("unexpected result type"),
                        }
                    }),
                }
            }
        }

        #vis struct #query_name #query_generic_def {
            #(#query_pk_fields,)*
            #(#query_sk_fields,)*
            pub limit: Option<usize>,
        }

        impl #query_generic_def forte_db::DbRequest for #query_name #query_generic_use {
            type Output = Vec<#name>;
            fn prepare(self) -> forte_db::Prepared<Self::Output> {
                let pk = #query_pk_str;
                let after_sk: Option<String> = {
                    let mut parts: Vec<String> = Vec::new();
                    'build: {
                        #(#query_sk_build)*
                    }
                    if parts.is_empty() { None } else { Some(parts.join("&")) }
                };
                let limit = self.limit;
                forte_db::Prepared {
                    ops: vec![forte_db::DbOp::Query { pk, after_sk, limit }],
                    parse: Box::new(|iter| {
                        match iter.next().ok_or_else(|| anyhow::anyhow!("missing result"))? {
                            forte_db::DbResult::Multiple(items) => {
                                items.into_iter()
                                    .map(|(_sk, data)| serde_json::from_slice(&data))
                                    .collect::<Result<Vec<_>, _>>()
                                    .map_err(Into::into)
                            }
                            _ => anyhow::bail!("unexpected result type"),
                        }
                    }),
                }
            }
        }

        #vis struct #delete_name #generic_def {
            #(#get_pk_fields,)*
            #(#get_sk_fields,)*
        }

        impl #generic_def forte_db::DbRequest for #delete_name #generic_use {
            type Output = ();
            fn prepare(self) -> forte_db::Prepared<Self::Output> {
                let pk = #pk_str;
                let sk = format!(#sk_format_string, #(#sk_format_args),*);
                forte_db::Prepared {
                    ops: vec![forte_db::DbOp::Delete { pk, sk }],
                    parse: Box::new(|iter| {
                        match iter.next().ok_or_else(|| anyhow::anyhow!("missing result"))? {
                            forte_db::DbResult::Done => Ok(()),
                            _ => anyhow::bail!("unexpected result type"),
                        }
                    }),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
