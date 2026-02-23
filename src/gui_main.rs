#![cfg(feature = "gui")]

use dioxus::prelude::*;

mod gui;

fn main() {
    dioxus::launch(gui::App);
}
