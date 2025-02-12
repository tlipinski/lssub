use std::env;

fn main() {
    let api_key = env::var("OSBK").expect("OSBK not set");
    println!("cargo:rustc-env=OSBK={}", api_key);
}
