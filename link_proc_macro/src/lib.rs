use proc_macro::TokenStream;
mod available_plugins;
mod frontend_plugins;

#[proc_macro_attribute]
pub fn generate_available_plugins(_attr: TokenStream, item: TokenStream) -> TokenStream {
    available_plugins::generate_available_plugins(item)
}

#[proc_macro_attribute]
pub fn generate_frontend_plugins(_attr: TokenStream, item: TokenStream) -> TokenStream {
    frontend_plugins::generate_frontend_plugins(item)
}
