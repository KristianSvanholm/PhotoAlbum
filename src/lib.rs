pub mod app;
pub mod auth;
pub mod components;
pub mod db;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;
pub mod image_filter;
pub mod session;
#[cfg(feature = "ssr")]
pub mod state;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
