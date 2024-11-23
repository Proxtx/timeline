use std::fs::read_dir;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, ItemEnum};

pub fn generate_available_plugins(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);

    let enum_name = &input.ident;
    let enum_variants = &input.variants;
    let vis = &input.vis;
    let attrs = &input.attrs;

    let plugins = get_plugins();

    let variants = plugins.iter().map(|v| {
        let ident = &Ident::new(v, proc_macro2::Span::call_site());
        quote! {
            #ident
        }
    });

    let expaned = quote! {
        #(#attrs)*
        #vis enum #enum_name {
            #(#variants,)*
            #enum_variants
        }
    };

    TokenStream::from(expaned)
}

pub fn get_plugins() -> Vec<String> {
    let dir = read_dir("../plugins/").expect("Unable to read plugins directory");
    dir.map(|v| {
        v.expect("Unable to read entry in the plugins directory")
            .file_name()
            .into_string()
            .expect("Unable to convert plugin name to rust string")
    })
    .collect()
}
