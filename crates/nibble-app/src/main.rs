//! nibble: a menu bar mouse that tells you whether your AI subscription pays for itself.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod settings;
mod shell;
mod tray;
mod view;

fn main() {
    if let Err(error) = shell::run() {
        report_fatal(&error);
    }
}

#[expect(
    clippy::print_stderr,
    reason = "last resort when the app cannot start at all"
)]
fn report_fatal(error: &tauri::Error) {
    eprintln!("nibble could not start: {error}");
}
