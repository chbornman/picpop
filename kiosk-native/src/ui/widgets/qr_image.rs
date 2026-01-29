//! QR code image widget that loads from URL with expand/collapse support.

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
pub struct ExpandableQrPanel {
    pub panel: gtk::Box,
    ctx: Rc<AppContext>,
    wifi_qr: gtk::Picture,
    session_qr: gtk::Picture,
    session_box: gtk::Box,
    wifi_label: gtk::Label,
    session_label: gtk::Label,
    is_expanded: Rc<Cell<bool>>,
    session_id: Rc<RefCell<Option<String>>>,
}

impl ExpandableQrPanel {
    /// Create an expandable QR panel (small in corner, expands on tap)
    pub fn new(ctx: &Rc<AppContext>) -> Rc<Self> {
        let panel = gtk::Box::new(gtk::Orientation::Vertical, 4);
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
        let wifi_qr = gtk::Picture::new();
        wifi_qr.set_hexpand(false);
        wifi_qr.set_vexpand(false);
        wifi_qr.add_css_class("qr-image");
        load_image_into_picture(ctx, &config::wifi_qr_url(config::QR_SIZE_SMALL), &wifi_qr);

        // Session QR placeholder
        let session_qr = gtk::Picture::new();
        session_qr.set_hexpand(false);
        session_qr.set_vexpand(false);
        session_qr.add_css_class("qr-image");

        // Labels (hidden when collapsed)
        let wifi_label = gtk::Label::new(Some("WiFi"));
        wifi_label.add_css_class("qr-label-small");
        wifi_label.set_visible(false);

        let session_label = gtk::Label::new(Some("Photos"));
        session_label.add_css_class("qr-label-small");
        session_label.set_visible(false);

        // WiFi section
        let wifi_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        wifi_box.set_halign(gtk::Align::Center);
        wifi_box.append(&wifi_qr);
        wifi_box.append(&wifi_label);

        // Session section
        let session_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        session_box.set_halign(gtk::Align::Center);
        session_box.append(&session_qr);
        session_box.append(&session_label);
        session_box.set_visible(false);

        panel.append(&wifi_box);
        panel.append(&session_box);

        let qr_panel = Rc::new(Self {
            panel,
            ctx: ctx.clone(),
            wifi_qr,
            session_qr,
            session_box,
            wifi_label,
            session_label,
            is_expanded,
            session_id,
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

    /// Toggle expanded state
    fn toggle_expanded(&self) {
        let expanded = !self.is_expanded.get();
        self.is_expanded.set(expanded);

        // Reload QR codes at the appropriate size
        let size = if expanded {
            config::QR_SIZE_LARGE
        } else {
            config::QR_SIZE_SMALL
        };

        // Reload WiFi QR at new size
        load_image_into_picture(&self.ctx, &config::wifi_qr_url(size), &self.wifi_qr);

        // Reload session QR at new size if we have one
        if let Some(ref id) = *self.session_id.borrow() {
            load_image_into_picture(
                &self.ctx,
                &config::session_qr_url(id, size),
                &self.session_qr,
            );
        }

        // Show/hide labels
        self.wifi_label.set_visible(expanded);
        if self.session_box.is_visible() {
            self.session_label.set_visible(expanded);
        }

        // Update CSS class
        if expanded {
            self.panel.remove_css_class("qr-panel-small");
            self.panel.add_css_class("qr-panel-expanded");
        } else {
            self.panel.remove_css_class("qr-panel-expanded");
            self.panel.add_css_class("qr-panel-small");
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

        load_image_into_picture(ctx, &config::session_qr_url(session_id, size), &self.session_qr);
        self.session_box.set_visible(true);
        self.session_label.set_visible(self.is_expanded.get());
    }

    /// Hide the session QR
    pub fn hide_session(&self) {
        *self.session_id.borrow_mut() = None;
        self.session_qr.set_paintable(None::<&gtk::gdk::Paintable>);
        self.session_box.set_visible(false);
        self.session_label.set_visible(false);
    }

    /// Collapse if expanded
    pub fn collapse(&self) {
        if self.is_expanded.get() {
            self.toggle_expanded();
        }
    }
}
