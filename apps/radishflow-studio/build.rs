use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    match target_env.as_str() {
        "msvc" => {
            println!("cargo:rustc-link-arg-bin=radishflow-studio=/STACK:16777216");
        }
        "gnu" => {
            println!("cargo:rustc-link-arg-bin=radishflow-studio=-Wl,--stack,16777216");
        }
        _ => {}
    }
}
