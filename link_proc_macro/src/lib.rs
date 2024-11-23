use proc_macro::TokenStream;
mod available_plugins;

#[proc_macro_attribute]
pub fn generate_available_plugins(_attr: TokenStream, item: TokenStream) -> TokenStream {
    available_plugins::generate_available_plugins(item)
}
