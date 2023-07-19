use std::env;

use winres::WindowsResource;

fn main() {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("icon/icon.ico")
            .compile().unwrap();
    }
}