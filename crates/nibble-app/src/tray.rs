//! The menu bar icon: the Wedge as a template image, plus the Multiplier as text.

use nibble_core::{Combined, Wedge};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::view;

const FULL: &[u8] = include_bytes!("../icons/tray/wedge-100.png");
const TWO_THIRDS: &[u8] = include_bytes!("../icons/tray/wedge-66.png");
const ONE_THIRD: &[u8] = include_bytes!("../icons/tray/wedge-33.png");
const CRUMBS: &[u8] = include_bytes!("../icons/tray/crumbs.png");
const PAID_OFF: &[u8] = include_bytes!("../icons/tray/paid-off.png");

/// The tray icon's id.
pub(crate) const ID: &str = "nibble";
/// The popover window's label, as in `tauri.conf.json`.
pub(crate) const POPOVER: &str = "popover";

/// Builds the tray icon with its click and menu behaviour.
pub(crate) fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<TrayIcon<R>> {
    let quit = MenuItem::with_id(app, "quit", "Quit nibble", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;
    TrayIconBuilder::with_id(ID)
        .icon(Image::from_bytes(FULL)?)
        .icon_as_template(true)
        .tooltip("nibble")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|handle, event| {
            if event.id() == "quit" {
                handle.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_popover(tray.app_handle());
            }
        })
        .build(app)
}

/// Shows the popover under the tray icon, or hides it if open.
fn toggle_popover<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window(POPOVER) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _hidden = window.hide();
    } else {
        let _moved = window.move_window(Position::TrayBottomCenter);
        let _shown = window.show();
        let _focused = window.set_focus();
        crate::shell::refresh_soon(app);
    }
}

/// Redraws the icon and Multiplier for the combined verdict.
pub(crate) fn show<R: Runtime>(app: &AppHandle<R>, combined: &Combined) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id(ID) else {
        return Ok(());
    };
    let bytes = match combined.wedge {
        Some(Wedge::PaidOff) => PAID_OFF,
        Some(Wedge::Crumbs) => CRUMBS,
        Some(Wedge::OneThird) => ONE_THIRD,
        Some(Wedge::TwoThirds) => TWO_THIRDS,
        Some(Wedge::Full) | None => FULL,
    };
    tray.set_icon(Some(Image::from_bytes(bytes)?))?;
    tray.set_icon_as_template(true)?;
    let label = combined.multiplier.map(view::multiplier);
    let tooltip = label.as_ref().map_or_else(
        || "nibble: eating through your plan".to_owned(),
        |m| format!("nibble: paid off, {m}"),
    );
    tray.set_title(label)?;
    tray.set_tooltip(Some(tooltip))
}
