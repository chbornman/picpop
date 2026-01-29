//! Main application window with unified session screen.

use gtk4 as gtk;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{AppContext, AppMessage};
use crate::state::{KioskCommand, KioskEvent, KioskState};
use crate::ui::session::{self, SessionWidgets};

/// Main window containing the unified session screen
pub struct MainWindow {
    pub window: gtk::ApplicationWindow,
    ctx: Rc<AppContext>,
    widgets: RefCell<Option<SessionWidgets>>,
}

impl MainWindow {
    pub fn new(app: &gtk::Application, ctx: Rc<AppContext>) -> Rc<Self> {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("PicPop Kiosk")
            .default_width(1920)
            .default_height(1080)
            .build();

        // Make fullscreen and hide cursor after window is mapped
        window.connect_map(|window| {
            let window = window.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                window.fullscreen();
            });
        });

        // Set cursor to none
        window.set_cursor_from_name(Some("none"));

        // Initialize video pipeline
        let video_paintable = ctx
            .init_video()
            .expect("Failed to initialize video pipeline");

        // Create the unified session screen
        let widgets = session::create_session_screen(&ctx, &video_paintable);

        // Set as window content
        window.set_child(Some(&widgets.overlay));

        let main_window = Rc::new(Self {
            window,
            ctx,
            widgets: RefCell::new(Some(widgets)),
        });

        // Connect event handlers
        main_window.connect_handlers();

        // Load CSS
        main_window.load_css();

        // Initialize to welcome mode
        main_window.update_ui();

        main_window
    }

    fn load_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(include_str!("../../resources/style.css"));

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("No display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    /// Connect UI event handlers to state machine
    fn connect_handlers(self: &Rc<Self>) {
        let widgets = self.widgets.borrow();
        let widgets = widgets.as_ref().unwrap();

        // Start session button
        let ctx = self.ctx.clone();
        widgets.connect_start(move || {
            ctx.send_event(KioskEvent::StartSession);
        });

        // End session button
        let ctx = self.ctx.clone();
        widgets.connect_end(move || {
            ctx.send_event(KioskEvent::EndSession);
        });

        // Capture button
        let ctx = self.ctx.clone();
        widgets.connect_capture(move || {
            ctx.send_event(KioskEvent::TriggerCapture);
        });

        // Tap on photo to return to live
        let ctx = self.ctx.clone();
        widgets.connect_photo_tap(move || {
            ctx.send_event(KioskEvent::SelectLive);
        });
    }

    /// Handle app messages - main entry point for state updates
    pub fn handle_message(self: &Rc<Self>, msg: AppMessage) {
        match msg {
            AppMessage::Event(ref event) => {
                // Process event through state machine
                let commands = self.ctx.process_event(event.clone());

                // Check if UI update was requested
                if commands.iter().any(|c| matches!(c, KioskCommand::UpdateUI)) {
                    self.update_ui();
                }
            }
        }
    }

    /// Update the UI to reflect current state
    fn update_ui(self: &Rc<Self>) {
        let sm = self.ctx.state_machine.borrow();
        let state = sm.state;
        let session = sm.session.clone();
        let countdown_value = sm.countdown_value;
        let viewing_photo = sm.viewing_photo;
        let is_loading = sm.is_loading;
        let error = sm.error.clone();
        drop(sm);

        let widgets = self.widgets.borrow();
        let widgets = widgets.as_ref().unwrap();

        match state {
            KioskState::Welcome => {
                widgets.set_welcome_mode();
                widgets.set_start_loading(is_loading, error.as_deref());
            }

            KioskState::Session => {
                if let Some(ref sess) = session {
                    // Ensure session mode is set
                    widgets.set_session_mode(&self.ctx, &sess.id);
                    widgets.set_phone_count(sess.phone_count);
                    widgets.hide_countdown();

                    // Update photo strip with current selection
                    let ctx1 = self.ctx.clone();
                    let ctx2 = self.ctx.clone();
                    widgets.update_photos(
                        &self.ctx,
                        &sess.photos,
                        viewing_photo,
                        move || ctx1.send_event(KioskEvent::SelectLive),
                        move |idx| ctx2.send_event(KioskEvent::SelectPhoto(idx)),
                    );

                    // Show either live view or selected photo
                    if let Some(idx) = viewing_photo {
                        if let Some(photo) = sess.photos.get(idx) {
                            widgets.show_photo(&self.ctx, photo);
                        }
                    } else {
                        widgets.show_live_view();
                    }
                }
            }

            KioskState::Countdown => {
                if let Some(value) = countdown_value {
                    widgets.show_countdown(value);
                }
            }

            KioskState::Processing => {
                widgets.show_processing();
            }
        }
    }
}
