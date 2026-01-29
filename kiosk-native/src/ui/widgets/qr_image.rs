//! QR code image widget that loads from URL with expand/collapse support.

use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::app::AppContext;
use crate::config;

/// Load an image from URL into a Picture widget
pub fn load_image_into_picture(ctx: &Rc<AppContext>, url: &str, picture: &gtk::Picture) {
    let full_url = if url.starts_with("http") {
        url.to_string()
    } else {
        format!("{}{}", config::API_BASE, url)
    };

    let picture = picture.clone();
    let api = ctx.api.clone();
    let runtime = ctx.runtime.clone();

    glib::spawn_future_local(async move {
        let result = runtime
            .spawn(async move { api.fetch_image(&full_url).await })
            .await;

        match result {
            Ok(Ok(bytes)) => {
                let gbytes = glib::Bytes::from(&bytes);
                let stream = gtk::gio::MemoryInputStream::from_bytes(&gbytes);
                if let Ok(pixbuf) = gtk::gdk_pixbuf::Pixbuf::from_stream(
                    &stream,
                    None::<&gtk::gio::Cancellable>,
                ) {
                    let texture = gtk::gdk::Texture::for_pixbuf(&pixbuf);
                    picture.set_paintable(Some(&texture));
                }
            }
            Ok(Err(e)) => {
                log::error!("Failed to load image: {}", e);
            }
            Err(e) => {
                log::error!("Task join error: {}", e);
            }
        }
    });
}

/// Create a QR code item with label below
fn create_qr_item(qr: &gtk::Picture, label_text: &str) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
    container.set_halign(gtk::Align::Center);
    container.set_valign(gtk::Align::Center);

    container.append(qr);

    let label = gtk::Label::new(Some(label_text));
    label.add_css_class("qr-label");
    container.append(&label);

    container
}

/// Expandable QR panel state
/// Displays QR codes in a horizontal row with labels, expands to fill screen on tap
pub struct ExpandableQrPanel {
    pub panel: gtk::Box,
    ctx: Rc<AppContext>,
    wifi_qr: gtk::Picture,
    session_qr: gtk::Picture,
    session_box: gtk::Box,
    is_expanded: Rc<Cell<bool>>,
    session_id: Rc<RefCell<Option<String>>>,
    /// Store animation reference to prevent it from being dropped
    animation: Rc<RefCell<Option<adw::TimedAnimation>>>,
}

impl ExpandableQrPanel {
    /// Create an expandable QR panel (small in corner, expands on tap)
    /// Uses horizontal layout with QR codes side by side
    pub fn new(ctx: &Rc<AppContext>) -> Rc<Self> {
        // Main panel - horizontal layout for QR codes in a row
        let panel = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        panel.add_css_class("qr-panel-small");
        panel.set_halign(gtk::Align::End);
        panel.set_valign(gtk::Align::Start);
        panel.set_hexpand(false);
        panel.set_vexpand(false);
        panel.set_margin_end(16);
        panel.set_margin_top(16);

        let is_expanded = Rc::new(Cell::new(false));
        let session_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        // Create WiFi QR
        let wifi_qr = gtk::Picture::new();
        wifi_qr.set_size_request(
            config::QR_SIZE_SMALL as i32,
            config::QR_SIZE_SMALL as i32,
        );
        wifi_qr.set_hexpand(false);
        wifi_qr.set_vexpand(false);
        wifi_qr.add_css_class("qr-image");
        load_image_into_picture(ctx, &config::wifi_qr_url(config::QR_SIZE_SMALL), &wifi_qr);

        // Session QR placeholder
        let session_qr = gtk::Picture::new();
        session_qr.set_size_request(
            config::QR_SIZE_SMALL as i32,
            config::QR_SIZE_SMALL as i32,
        );
        session_qr.set_hexpand(false);
        session_qr.set_vexpand(false);
        session_qr.add_css_class("qr-image");

        // WiFi box with label
        let wifi_box = create_qr_item(&wifi_qr, "WIFI");

        // Session box with label
        let session_box = create_qr_item(&session_qr, "PHOTOS");
        session_box.set_visible(false);

        panel.append(&wifi_box);
        panel.append(&session_box);

        let qr_panel = Rc::new(Self {
            panel,
            ctx: ctx.clone(),
            wifi_qr,
            session_qr,
            session_box,
            is_expanded,
            session_id,
            animation: Rc::new(RefCell::new(None)),
        });

        // Click handler
        let gesture = gtk::GestureClick::new();
        let qr_panel_clone = qr_panel.clone();
        gesture.connect_released(move |_, _, _, _| {
            qr_panel_clone.toggle_expanded();
        });
        qr_panel.panel.add_controller(gesture);

        qr_panel
    }

    /// Toggle expanded state with smooth animation
    fn toggle_expanded(&self) {
        let expanded = !self.is_expanded.get();
        self.is_expanded.set(expanded);

        let (from_size, to_size) = if expanded {
            (config::QR_SIZE_SMALL, config::QR_SIZE_LARGE)
        } else {
            (config::QR_SIZE_LARGE, config::QR_SIZE_SMALL)
        };

        // Reload QR codes at the target size for crisp rendering
        load_image_into_picture(&self.ctx, &config::wifi_qr_url(to_size), &self.wifi_qr);

        if let Some(ref id) = *self.session_id.borrow() {
            load_image_into_picture(
                &self.ctx,
                &config::session_qr_url(id, to_size),
                &self.session_qr,
            );
        }

        // Update CSS class and positioning immediately
        if expanded {
            self.panel.remove_css_class("qr-panel-small");
            self.panel.add_css_class("qr-panel-expanded");
            self.panel.set_halign(gtk::Align::Center);
            self.panel.set_valign(gtk::Align::Center);
            self.panel.set_margin_end(0);
            self.panel.set_margin_top(0);
        } else {
            self.panel.remove_css_class("qr-panel-expanded");
            self.panel.add_css_class("qr-panel-small");
            self.panel.set_halign(gtk::Align::End);
            self.panel.set_valign(gtk::Align::Start);
            self.panel.set_margin_end(16);
            self.panel.set_margin_top(16);
        }

        // Animate the size transition
        self.animate_size(from_size, to_size);
    }

    /// Animate QR code size with smooth easing
    fn animate_size(&self, from_size: u32, to_size: u32) {
        // Cancel any existing animation
        if let Some(ref anim) = *self.animation.borrow() {
            anim.skip();
        }

        let wifi_qr = self.wifi_qr.clone();
        let session_qr = self.session_qr.clone();
        let session_box = self.session_box.clone();

        // Create callback target that updates widget sizes
        let target = adw::CallbackAnimationTarget::new(move |value| {
            let size = value as i32;
            wifi_qr.set_size_request(size, size);
            if session_box.is_visible() {
                session_qr.set_size_request(size, size);
            }
        });

        // Create timed animation (300ms with ease-out-cubic)
        let animation = adw::TimedAnimation::builder()
            .widget(&self.panel)
            .value_from(from_size as f64)
            .value_to(to_size as f64)
            .duration(300)
            .easing(adw::Easing::EaseOutCubic)
            .target(&target)
            .build();

        animation.play();
        *self.animation.borrow_mut() = Some(animation);
    }

    /// Update the session QR with fade-in animation
    pub fn set_session(&self, ctx: &Rc<AppContext>, session_id: &str) {
        *self.session_id.borrow_mut() = Some(session_id.to_string());

        let size = if self.is_expanded.get() {
            config::QR_SIZE_LARGE
        } else {
            config::QR_SIZE_SMALL
        };

        self.session_qr.set_size_request(size as i32, size as i32);
        load_image_into_picture(ctx, &config::session_qr_url(session_id, size), &self.session_qr);

        // Fade in the session box
        self.session_box.set_opacity(0.0);
        self.session_box.set_visible(true);
        super::animations::fade_in(&self.session_box, super::animations::duration::NORMAL);
    }

    /// Hide the session QR with fade-out animation
    pub fn hide_session(&self) {
        *self.session_id.borrow_mut() = None;

        if self.session_box.is_visible() {
            let session_qr = self.session_qr.clone();
            let session_box = self.session_box.clone();
            super::animations::fade(
                &self.session_box,
                1.0,
                0.0,
                super::animations::duration::FAST,
                Some(Box::new(move || {
                    session_qr.set_paintable(None::<&gtk::gdk::Paintable>);
                    session_box.set_visible(false);
                    session_box.set_opacity(1.0);
                })),
            );
        }
    }

    /// Collapse if expanded
    pub fn collapse(&self) {
        if self.is_expanded.get() {
            self.toggle_expanded();
        }
    }
}
