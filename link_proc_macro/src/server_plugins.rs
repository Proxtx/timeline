use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

use crate::available_plugins;

pub fn generate_server_plugins(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let struct_name = &input.ident;
    let generics = &input.generics;

    let plugins = available_plugins::get_plugins()
        .into_iter()
        .map(|v| (v.clone(), format!("{}_server", v)))
        .collect::<Vec<_>>();
    let av_idents = plugins.iter().map(|v| {
        let ident = Ident::new(&v.0, Span::call_site());
        quote! {
            #ident
        }
    });

    let av_idents_clone = av_idents.clone();

    let im_idents = plugins.iter().map(|v| {
        let ident = Ident::new(&v.1, Span::call_site());
        quote! {
            #ident
        }
    });

    let im_idents_clone = im_idents.clone();

    let expaned = quote! {
        #input

        impl #generics #struct_name #generics {
            pub async fn init(mut handler: impl FnMut(AvailablePlugins) -> PluginData) -> #struct_name #generics {
                #struct_name {
                    plugins: HashMap::from([#((AvailablePlugins::#av_idents, Box::new(#im_idents::Plugin::new(handler(AvailablePlugins::#av_idents)).await) as Box<dyn PluginTrait>),)*]),
                    routes: HashMap::from([#((AvailablePlugins::#av_idents_clone, #im_idents_clone::Plugin::get_routes()),)*])
                }
            }
        }
    };

    TokenStream::from(expaned)
}
