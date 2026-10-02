//! `mol-desktop` — optional egui energy harness window.
//!
//! Build: `cargo run -p mol-desktop --features gui --bin mol-desktop`
//! Headless API (prove): `cargo test -p mol-desktop`

fn main() -> eframe::Result<()> {
    mol_desktop::gui::run_native()
}
