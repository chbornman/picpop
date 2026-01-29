//! QR code image widget that loads from URL with expand/collapse support.

use adw::prelude::*;
use adw::Animation;
use adw::TimedAnimation;
use gtk4 as gtk;
use gtk4::prelude::*;
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

/// Expandable QR panel state
/// Displays QR codes in a horizontal row, expands to fill screen on tap
pub struct ExpandableQrPanel {
    pub panel: gtk::Box,
    ctx: Rc<AppContext>,
    wifi_qr: gtk::Picture,
    session_qr: gtk::Picture,
    session_box: gtk::Box,
    is_expanded: Rc<Cell<bool>>,
    session_id: Rc<RefCell<Option<String>>>,
    animation: Rc<RefCell<Option<TimedAnimation>>>,
}

impl ExpandableQrPanel {
    /// Create an expandable QR panel (small in corner, expands on tap)
    /// Uses horizontal layout with QR codes side by side
    pub fn new(ctx: &Rc<AppContext>) -> Rc<Self> {
        // Main panel - horizontal layout for QR codes in a row
        let panel = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        panel.add_css_class("qr-panel-small");
        panel.set_halign(gtk::Align::End);
        panel.set_valign(gtk::Align::Start);
        panel.set_hexpand(false);
        panel.set_vexpand(false);
        panel.set_margin_end(16);
        panel.set_margin_top(16);

        let is_expanded = Rc::new(Cell::new(false));
        let session_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        // Create WiFi QR (small size initially)
        // Labels are now embedded in the QR code image itself
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

        // WiFi box (just the QR, label is embedded)
        let wifi_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        wifi_box.set_halign(gtk::Align::Center);
        wifi_box.set_valign(gtk::Align::Center);
        wifi_box.append(&wifi_qr);

        // Session box (just the QR, label is embedded)
        let session_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        session_box.set_halign(gtk::Align::Center);
        session_box.set_valign(gtk::Align::Center);
        session_box.append(&session_qr);
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

    /// Animate the QR panel size transition
    fn animate_size(&self, from_size: u32, to_size: u32) {
        // Cancel any existing animation
        if let Some(ref anim) = *self.animation.borrow() {
            anim.skip();
        }

        let wifi_qr = self.wifi_qr.clone();
        let session_qr = self.session_qr.clone();
        let session_box = self.session_box.clone();

        // Create animation target that updates widget sizes
        let target = adw::CallbackAnimationTarget::new(move |value| {
            let size = value as i32;
            wifi_qr.set_size_request(size, size);
            if session_box.is_visible() {
                session_qr.set_size_request(size, size);
            }
        });

        // Create timed animation (300ms with ease-out curve)
        let animation = TimedAnimation::builder()
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

        // Animate the size transition
        self.animate_size(from_size, to_size);

        // Update CSS class for styling
        if expanded {
            self.panel.remove_css_class("qr-panel-small");
            self.panel.add_css_class("qr-panel-expanded");
            // Center the panel when expanded
            self.panel.set_halign(gtk::Align::Center);
            self.panel.set_valign(gtk::Align::Center);
            self.panel.set_margin_end(0);
            self.panel.set_margin_top(0);
        } else {
            self.panel.remove_css_class("qr-panel-expanded");
            self.panel.add_css_class("qr-panel-small");
            // Position in corner when collapsed
            self.panel.set_halign(gtk::Align::End);
            self.panel.set_valign(gtk::Align::Start);
            self.panel.set_margin_end(16);
            self.panel.set_margin_top(16);
        }
    }

    /// Update the session QR
    pub fn set_session(&self, ctx: &Rc<AppContext>, session_id: &str) {
        *self.session_id.borrow_mut() = Some(session_id.to_string());

        let size = if self.is_expanded.get() {
            config::QR_SIZE_LARGE
        } else {
            config::QR_SIZE_SMALL
        };

        self.session_qr.set_size_request(size as i32, size as i32);
        load_image_into_picture(ctx, &config::session_qr_url(session_id, size), &self.session_qr);
        self.session_box.set_visible(true);
    }

    /// Hide the session QR
    pub fn hide_session(&self) {
        *self.session_id.borrow_mut() = None;
        self.session_qr.set_paintable(None::<&gtk::gdk::Paintable>);
        self.session_box.set_visible(false);
    }

    /// Collapse if expanded
    pub fn collapse(&self) {
        if self.is_expanded.get() {
            self.toggle_expanded();
        }
    }
}
