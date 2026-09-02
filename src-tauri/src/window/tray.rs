//! Owned system-tray resources for the main window.

use std::sync::Arc;

use tauri::{
    AppHandle, Manager, Wry,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use super::WindowCoordinator;

pub const TRAY_ID: &str = "wubilex-main-tray";
pub const SHOW_MENU_ID: &str = "tray.show";
pub const EXIT_MENU_ID: &str = "tray.exit";

pub fn create_owned_tray(app: &AppHandle<Wry>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, SHOW_MENU_ID, "显示 WubiLex", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, EXIT_MENU_ID, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &exit])?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("WubiLex")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let Some(coordinator) = app.try_state::<Arc<WindowCoordinator>>() else {
                return;
            };
            if event.id() == SHOW_MENU_ID {
                let _ = coordinator.restore();
            } else if event.id() == EXIT_MENU_ID {
                coordinator.request_exit();
            }
        })
        .on_tray_icon_event(|tray, event| {
            if !matches!(
                event,
                TrayIconEvent::Click {
                    ref id,
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } if id == TRAY_ID
            ) {
                return;
            }
            if let Some(coordinator) = tray.app_handle().try_state::<Arc<WindowCoordinator>>() {
                let _ = coordinator.restore();
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn remove_owned_tray(app: &AppHandle<Wry>) {
    drop(app.remove_tray_by_id(TRAY_ID));
}
