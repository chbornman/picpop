//! Main application window with unified session screen.

use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

use crate::app::{AppContext, AppMessage};
use crate::state::{KioskCommand, KioskEvent, KioskState};
use crate::ui::session::{self, SessionWidgets};

/// Main window containing the unified session screen
pub struct MainWindow {
    pub window: adw::ApplicationWindow,
    ctx: Rc<AppContext>,
    widgets: RefCell<Option<SessionWidgets>>,
}

impl MainWindow {
    pub fn new(app: &adw::Application, ctx: Rc<AppContext>) -> Rc<Self> {
        let window = adw::ApplicationWindow::builder()
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

        // Create a top-level overlay for escape hatch (above everything)
        let top_overlay = gtk::Overlay::new();
        top_overlay.set_child(Some(&widgets.overlay));

        // Add escape zones at window level (above all content)
        Self::setup_escape_hatch(&top_overlay, &window);

        // Set as window content
        window.set_content(Some(&top_overlay));

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

    /// Setup escape hatch at window level - two-tap sequence to return to launcher
    fn setup_escape_hatch(overlay: &gtk::Overlay, window: &adw::ApplicationWindow) {
        let escape_state: Rc<RefCell<Option<std::time::Instant>>> = Rc::new(RefCell::new(None));

        // Bottom-left zone (first tap)
        let escape_left = gtk::Box::new(gtk::Orientation::Vertical, 0);
        escape_left.set_size_request(100, 100);
        escape_left.set_halign(gtk::Align::Start);
        escape_left.set_valign(gtk::Align::End);
        escape_left.add_css_class("escape-zone");

        let state_for_left = escape_state.clone();
        let gesture_left = gtk::GestureClick::new();
        gesture_left.set_propagation_phase(gtk::PropagationPhase::Capture);
        gesture_left.connect_pressed(move |_, _, _, _| {
            log::info!("Escape hatch: left corner tapped (step 1)");
            *state_for_left.borrow_mut() = Some(std::time::Instant::now());
        });
        escape_left.add_controller(gesture_left);
        overlay.add_overlay(&escape_left);

        // Bottom-right zone (second tap)
        let escape_right = gtk::Box::new(gtk::Orientation::Vertical, 0);
        escape_right.set_size_request(100, 100);
        escape_right.set_halign(gtk::Align::End);
        escape_right.set_valign(gtk::Align::End);
        escape_right.add_css_class("escape-zone");

        let state_for_right = escape_state.clone();
        let window_for_right = window.clone();
        let gesture_right = gtk::GestureClick::new();
        gesture_right.set_propagation_phase(gtk::PropagationPhase::Capture);
        gesture_right.connect_pressed(move |_, _, _, _| {
            let mut state = state_for_right.borrow_mut();
            if let Some(first_tap) = *state {
                if first_tap.elapsed() < std::time::Duration::from_secs(3) {
                    log::info!("Escape hatch: right corner tapped (step 2) - showing dialog");
                    *state = None;
                    Self::show_escape_confirmation(&window_for_right);
                } else {
                    log::info!("Escape hatch: right corner tapped but too slow, resetting");
                    *state = None;
                }
            } else {
                log::info!("Escape hatch: right corner tapped but left wasn't tapped first");
            }
        });
        escape_right.add_controller(gesture_right);
        overlay.add_overlay(&escape_right);
    }

    /// Show confirmation dialog to return to launcher
    fn show_escape_confirmation(window: &adw::ApplicationWindow) {
        let dialog = adw::AlertDialog::new(
            Some("Return to Launcher?"),
            Some("This will end the current session and return to the app selector."),
        );

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("confirm", "Return to Launcher");
        dialog.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        dialog.connect_response(None, |_, response| {
            if response == "confirm" {
                Self::return_to_launcher();
            }
        });

        dialog.present(Some(window));
    }

    /// Return to the launcher by updating kiosk.conf and restarting getty
    fn return_to_launcher() {
        log::info!("Escape hatch triggered - returning to launcher");

        // Write launcher as the target user
        let _ = Command::new("sudo")
            .args(["tee", "/etc/kiosk.conf"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(b"KIOSK_USER=launcher\n")?;
                }
                child.wait()
            });

        // Restart getty to switch user
        let _ = Command::new("sudo")
            .args(["systemctl", "restart", "getty@tty1"])
            .status();
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
