use std::{
    fs::{read_dir, File},
    io::Read,
};

use {
    proc_macro::TokenStream,
    proc_macro2::Ident,
    quote::quote,
    syn::{parse_macro_input, ItemEnum},
};

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
    match File::open("./plugins.txt") {
        Ok(mut file) => {
            let mut plugins = String::new();
            file.read_to_string(&mut plugins).unwrap();
            plugins.trim().split(",").map(|v| v.to_string()).collect()
        }
        Err(_e) => {
            let dir = read_dir("../plugins/").expect(&format!("Unable to read plugins directory: {:?} {:?}", std::env::current_dir(), std::process::Command::new("ls").output().unwrap()));
            dir.map(|v| {
                v.expect("Unable to read entry in the plugins directory")
                    .file_name()
                    .into_string()
                    .expect("Unable to convert plugin name to rust string")
            })
            .collect()
        }
    }
}
