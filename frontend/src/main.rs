#[macro_use]
extern crate log;
mod frontend;

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    console_error_panic_hook::set_once();
    yew::start_app::<frontend::App>();
}
