//! WASM entry point for Leptos CSR app

use ccboard_web::App;
use leptos::mount::mount_to_body;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
