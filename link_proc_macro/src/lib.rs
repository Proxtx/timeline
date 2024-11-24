use proc_macro::TokenStream;
mod available_plugins;
mod client_plugins;
mod server_plugins;

#[proc_macro_attribute]
pub fn generate_available_plugins(_attr: TokenStream, item: TokenStream) -> TokenStream {
    available_plugins::generate_available_plugins(item)
}

#[proc_macro_attribute]
pub fn generate_client_plugins(_attr: TokenStream, item: TokenStream) -> TokenStream {
    client_plugins::generate_client_plugins(item)
}

#[proc_macro_attribute]
pub fn generate_server_plugins(_attr: TokenStream, item: TokenStream) -> TokenStream {
    server_plugins::generate_server_plugins(item)
}
