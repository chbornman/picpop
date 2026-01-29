//! PicPop Native Kiosk - GTK4 + GStreamer photo booth kiosk application.
//!
//! Architecture:
//! - `state` module: GTK-free state machine with business logic (testable)
//! - `app` module: Bridges state machine to GTK and async operations
//! - `api` module: HTTP and WebSocket clients
//! - `video` module: GStreamer pipeline for camera preview
//! - `ui` module: GTK4 widgets and screens

use std::sync::Arc;

use gtk4 as gtk;
use gtk4::prelude::*;

mod api;
mod app;
mod config;
mod state;
mod ui;
mod video;

use app::AppContext;
use ui::MainWindow;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting PicPop Kiosk");

    // Create tokio runtime for async operations
    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime"),
    );

    // Set cursor theme to invisible before GTK init
    std::env::set_var("XCURSOR_THEME", "InvisibleCursor");
    std::env::set_var("XCURSOR_SIZE", "1");

    // Create GTK application
    let app = gtk::Application::builder()
        .application_id("com.picpop.kiosk")
        .build();

    let runtime_clone = runtime.clone();

    app.connect_activate(move |app| {
        // Create application context (includes GTK-free state machine)
        let (ctx, mut rx) = AppContext::new(runtime_clone.clone());

        // Create main window (GTK layer)
        let main_window = MainWindow::new(app, ctx.clone());

        // Poll the tokio channel from the GTK main loop
        let window = main_window.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            // Process all pending messages
            while let Ok(msg) = rx.try_recv() {
                window.handle_message(msg);
            }
            glib::ControlFlow::Continue
        });

        main_window.window.present();
    });

    // Run the application
    app.run();

    log::info!("PicPop Kiosk shutting down");
}
