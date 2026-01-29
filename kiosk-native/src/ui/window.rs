//! Main application window with screen stack.

use gtk4 as gtk;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{AppContext, AppMessage, KioskState, SessionData};
use crate::state::{KioskCommand, KioskEvent};
use crate::ui::{countdown, lightbox, session, welcome};

/// Main window containing all screens
pub struct MainWindow {
    pub window: gtk::ApplicationWindow,
    stack: gtk::Stack,
    ctx: Rc<AppContext>,
    video_paintable: gtk::gdk::Paintable,
    session_widgets: RefCell<Option<session::SessionWidgets>>,
}

impl MainWindow {
    pub fn new(app: &gtk::Application, ctx: Rc<AppContext>) -> Rc<Self> {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("PicPop Kiosk")
            .default_width(1280)
            .default_height(800)
            .build();

        // Make fullscreen and hide cursor after window is mapped
        // Use connect_map instead of connect_realize to ensure Wayland surface is ready
        // This avoids "surface->initialized" assertion failures in Cage/wlroots
        window.connect_map(|window| {
            // Delay fullscreen slightly to ensure Wayland surface is fully initialized
            let window = window.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                // Hide cursor for kiosk
                if let Some(surface) = window.surface() {
                    if let Some(cursor) = gtk::gdk::Cursor::from_name("none", None) {
                        surface.set_cursor(Some(&cursor));
                    }
                }
                // Fullscreen after surface is ready
                window.fullscreen();
            });
        });

        // Create stack for screen transitions
        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        stack.set_transition_duration(300);

        window.set_child(Some(&stack));

        // Initialize video pipeline
        let video_paintable = ctx
            .init_video()
            .expect("Failed to initialize video pipeline");

        let main_window = Rc::new(Self {
            window,
            stack,
            ctx,
            video_paintable,
            session_widgets: RefCell::new(None),
        });

        // Set up initial welcome screen
        main_window.show_welcome();

        // Load CSS
        main_window.load_css();

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

    /// Handle app messages - main entry point for state updates
    pub fn handle_message(self: &Rc<Self>, msg: AppMessage) {
        match msg {
            AppMessage::Event(event) => {
                // Process event through state machine
                let commands = self.ctx.process_event(event);

                // Check if UI update was requested
                if commands.iter().any(|c| matches!(c, KioskCommand::UpdateUI)) {
                    self.update_ui();
                }
            }
            AppMessage::ImageLoaded { url, bytes } => {
                // Image loading is handled by individual widgets
                log::debug!("Image loaded: {} ({} bytes)", url, bytes.len());
            }
        }
    }

    /// Update the UI to reflect current state
    fn update_ui(self: &Rc<Self>) {
        let sm = self.ctx.state_machine.borrow();
        let state = sm.state;
        let session = sm.session.clone();
        let countdown_value = sm.countdown_value;
        let lightbox_index = sm.lightbox_index;
        let is_loading = sm.is_loading;
        let error = sm.error.clone();
        drop(sm);

        match state {
            KioskState::Welcome => {
                if self.stack.visible_child_name().as_deref() != Some("welcome") {
                    self.show_welcome();
                }
                // Update button state
                self.update_welcome_button(is_loading, error.as_deref());
            }

            KioskState::Session => {
                if self.stack.visible_child_name().as_deref() != Some("session") {
                    if let Some(ref sess) = session {
                        self.show_session(&sess.id);
                    }
                } else {
                    // Update session widgets
                    self.update_session_widgets(&session);
                }
                // Make sure countdown is hidden
                if self.stack.child_by_name("countdown").is_some() {
                    self.hide_countdown();
                }
            }

            KioskState::Countdown => {
                if let Some(value) = countdown_value {
                    if self.stack.visible_child_name().as_deref() == Some("countdown") {
                        self.update_countdown(value);
                    } else {
                        self.show_countdown(value);
                    }
                }
            }

            KioskState::Capturing => {
                if let Some(ref widgets) = *self.session_widgets.borrow() {
                    widgets.set_capturing(true);
                }
            }

            KioskState::Lightbox => {
                if let Some(index) = lightbox_index {
                    if self.stack.visible_child_name().as_deref() == Some("lightbox") {
                        self.update_lightbox(index);
                    } else {
                        self.show_lightbox(index);
                    }
                }
            }
        }
    }

    /// Show the welcome screen
    fn show_welcome(self: &Rc<Self>) {
        // Remove old screens
        while let Some(child) = self.stack.first_child() {
            self.stack.remove(&child);
        }

        let ctx = self.ctx.clone();
        let screen =
            welcome::create_welcome_screen(&self.ctx, &self.video_paintable, false, move || {
                ctx.send_event(KioskEvent::StartSession)
            });
        self.stack.add_named(&screen, Some("welcome"));
        self.stack.set_visible_child_name("welcome");

        *self.session_widgets.borrow_mut() = None;
    }

    /// Update welcome screen button
    fn update_welcome_button(&self, is_loading: bool, error: Option<&str>) {
        if let Some(child) = self.stack.child_by_name("welcome") {
            if let Some(screen) = child.downcast_ref::<gtk::Overlay>() {
                welcome::update_start_button(screen, is_loading, error);
            }
        }
    }

    /// Show the session screen
    fn show_session(self: &Rc<Self>, session_id: &str) {
        let session = self.ctx.session().unwrap_or_default();

        // Remove old screens
        while let Some(child) = self.stack.first_child() {
            self.stack.remove(&child);
        }

        let ctx1 = self.ctx.clone();
        let ctx2 = self.ctx.clone();
        let ctx3 = self.ctx.clone();

        let (screen, widgets) = session::create_session_screen(
            &self.ctx,
            &self.video_paintable,
            session_id,
            session.phone_count,
            &session.photos,
            false,
            move || ctx1.send_event(KioskEvent::TriggerCapture),
            move || ctx2.send_event(KioskEvent::EndSession),
            move |idx| ctx3.send_event(KioskEvent::OpenLightbox(idx)),
        );

        self.stack.add_named(&screen, Some("session"));
        self.stack.set_visible_child_name("session");

        *self.session_widgets.borrow_mut() = Some(widgets);
    }

    /// Update session screen widgets
    fn update_session_widgets(self: &Rc<Self>, session: &Option<SessionData>) {
        if let Some(ref widgets) = *self.session_widgets.borrow() {
            if let Some(ref sess) = session {
                widgets.set_phone_count(sess.phone_count);
                widgets.set_capturing(false);

                let ctx = self.ctx.clone();
                widgets.update_photos(&self.ctx, &sess.photos, move |idx| {
                    ctx.send_event(KioskEvent::OpenLightbox(idx))
                });
            }
        }
    }

    /// Show countdown overlay
    fn show_countdown(self: &Rc<Self>, value: u32) {
        let overlay = countdown::create_countdown_overlay(value);
        self.stack.add_named(&overlay, Some("countdown"));
        self.stack.set_visible_child_name("countdown");
    }

    /// Update countdown value
    fn update_countdown(&self, value: u32) {
        if let Some(child) = self.stack.child_by_name("countdown") {
            if let Some(overlay) = child.downcast_ref::<gtk::Box>() {
                countdown::update_countdown(overlay, value);
            }
        }
    }

    /// Hide countdown overlay
    fn hide_countdown(&self) {
        if let Some(child) = self.stack.child_by_name("countdown") {
            self.stack.remove(&child);
        }
        self.stack.set_visible_child_name("session");
    }

    /// Show lightbox for a photo
    fn show_lightbox(self: &Rc<Self>, index: usize) {
        let photos = self
            .ctx
            .state_machine
            .borrow()
            .session
            .as_ref()
            .map(|s| s.photos.clone())
            .unwrap_or_default();

        if photos.is_empty() {
            return;
        }

        let ctx1 = self.ctx.clone();
        let ctx2 = self.ctx.clone();

        let lb = lightbox::create_lightbox(
            &self.ctx,
            &photos,
            index,
            move || ctx1.send_event(KioskEvent::CloseLightbox),
            move |new_idx| ctx2.send_event(KioskEvent::NavigateLightbox(new_idx)),
        );

        // Remove old lightbox if present
        if let Some(child) = self.stack.child_by_name("lightbox") {
            self.stack.remove(&child);
        }

        self.stack.add_named(&lb, Some("lightbox"));
        self.stack.set_visible_child_name("lightbox");
    }

    /// Update lightbox to show a different photo
    fn update_lightbox(&self, index: usize) {
        let photos = self
            .ctx
            .state_machine
            .borrow()
            .session
            .as_ref()
            .map(|s| s.photos.clone())
            .unwrap_or_default();

        if let Some(child) = self.stack.child_by_name("lightbox") {
            if let Some(lb) = child.downcast_ref::<gtk::Box>() {
                lightbox::update_lightbox(&self.ctx, lb, &photos, index);
            }
        }
    }
}
