use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Fields, ItemStruct, LitStr};

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
        .map(|n| {
            let name_str = n.as_ref().unwrap().to_string();
            format!("{}={{}}", name_str)
        })
        .collect();
    let sk_format_string = sk_format_parts.join(",");

    let sk_format_args: Vec<_> = sk_field_names
        .iter()
        .map(|n| {
            let ident = n.as_ref().unwrap();
            quote! { sk.#ident }
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
