mod adapters;
mod airac;
mod models;
mod i18n;
mod pdf;
mod persistence;
mod state;

mod components;

use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::desktop::Config;
use dioxus::prelude::*;

use components::layout::AppShell;

const MAIN_CSS: &str = include_str!("../assets/main.css");

fn main() {
    LaunchBuilder::new()
        .with_cfg(desktop! {
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("ATC-BOOK")
                    .with_always_on_top(false),
            )
        })
        .launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Style { {MAIN_CSS} }
        AppShell {}
    }
}
