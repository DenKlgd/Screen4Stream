//! Окно приложения Screen4Stream.
//!
//! Tauri здесь — только оболочка: окно показывает веб-интерфейс с нашего же
//! локального сервера (тот же, что открывается в браузере), плюс иконка в трее.

// В release-сборке не показывать консольное окно.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder, WindowEvent};

const MAIN_WINDOW: &str = "main";

fn main() {
    s4s_server::init_logging();
    s4s_core::init().expect("не удалось инициализировать GStreamer — он установлен?");

    tauri::Builder::default()
        // Повторный запуск не открывает вторую копию, а показывает уже открытое окно.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .setup(|app| {
            // Сначала занимаем порты, потом открываем окно — так страница точно загрузится.
            let server =
                tauri::async_runtime::block_on(s4s_server::Server::bind(Default::default()))?;
            let url = server.local_url();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = server.run().await {
                    tracing::error!("сервер остановился: {err:#}");
                }
            });

            create_main_window(app.handle(), &url)?;
            create_tray(app.handle(), url)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Как в Discord: крестик прячет окно в трей, трансляция продолжается.
            // Полностью выйти — пункт «Выход» в меню иконки.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение");
}

fn create_main_window(app: &AppHandle, server_url: &str) -> tauri::Result<()> {
    // ?app=1 — интерфейс узнаёт, что открыт в окне приложения, а не в браузере.
    let url: Url = format!("{server_url}/?app=1")
        .parse()
        .expect("адрес сервера всегда корректный");
    WebviewWindowBuilder::new(app, MAIN_WINDOW, WebviewUrl::External(url))
        .title("Screen4Stream")
        .inner_size(1100.0, 760.0)
        .min_inner_size(720.0, 540.0)
        .build()?;
    Ok(())
}

fn create_tray(app: &AppHandle, server_url: String) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Открыть", true, None::<&str>)?;
    let browser = MenuItem::with_id(app, "browser", "Открыть в браузере", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &browser, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().expect("иконка задана в tauri.conf.json"))
        .tooltip("Screen4Stream")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "browser" => s4s_server::open_in_browser(&server_url),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
