// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(target_os = "linux")]
            {
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                disable_gtk_gestures(&window);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "linux")]
fn disable_gtk_gestures(window: &tauri::WebviewWindow) {
    use gtk::prelude::*;

    window.with_webview(|webview| {
        use webkit2gtk::WebViewExt;

        let wv = webview.inner();

        // Disable context menu
        wv.connect_context_menu(|_, _, _, _| true);

        // Disable zoom and other gestures using unsafe GTK internals
        unsafe {
            use gobject_sys::g_signal_handlers_destroy;

            // Try to disable zoom gesture
            if let Some(data) = wv.data::<gtk::GestureZoom>("wk-view-zoom-gesture") {
                g_signal_handlers_destroy(data.as_ptr() as *mut _);
            }

            // Disable long-press and swipe by setting empty data
            wv.set_data::<()>("wk-view-long-press-gesture", ());
            wv.set_data::<()>("wk-view-swipe-gesture", ());
        }
    }).ok();
}
