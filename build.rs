use std::env;

fn main() {
    let api_key = env::var("OSB_API_KEY").expect("OSB_API_KEY not set");
    println!("cargo:rustc-env=OSB_API_KEY={}", api_key);
}
