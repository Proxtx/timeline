//mod config;
mod db;
include!(concat!(env!("OUT_DIR"), "/plugins.rs"));

pub trait Plugin {
    fn init(&self);
}

#[tokio::main]
async fn main() {
    let t = Plugins::init().await;
    t.plugins["test"].init();
}
