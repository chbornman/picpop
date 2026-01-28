//! Session screen - main operational view during active session.

use gtk4 as gtk;
use gtk4::prelude::*;
use std::rc::Rc;

use crate::api::PhotoInfo;
use crate::app::AppContext;
use crate::ui::widgets;

/// References to updateable widgets in the session screen
pub struct SessionWidgets {
    pub phone_count_label: gtk::Label,
    pub capture_button: gtk::Button,
    pub capture_status: gtk::Label,
    pub photo_strip: gtk::ScrolledWindow,
    pub capturing_overlay: gtk::Box,
    pub qr_container: gtk::Box,
}

/// Create the session screen
pub fn create_session_screen(
    ctx: &Rc<AppContext>,
    video_paintable: &gtk::gdk::Paintable,
    session_id: &str,
    phone_count: u32,
    photos: &[PhotoInfo],
    is_capturing: bool,
    on_capture: impl Fn() + Clone + 'static,
    on_end: impl Fn() + 'static,
    on_photo_click: impl Fn(usize) + Clone + 'static,
) -> (gtk::Box, SessionWidgets) {
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.add_css_class("session-screen");

    // === Top bar ===
    let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    top_bar.add_css_class("top-bar");
    top_bar.set_margin_start(24);
    top_bar.set_margin_end(24);
    top_bar.set_margin_top(16);
    top_bar.set_margin_bottom(16);

    // Phone count (left side)
    let phone_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let phone_icon = gtk::Label::new(Some("\u{1F4F1}"));
    phone_icon.add_css_class("phone-icon");

    let phone_count_label = gtk::Label::new(Some(&format!("{} connected", phone_count)));
    phone_count_label.add_css_class("phone-count");

    phone_box.append(&phone_icon);
    phone_box.append(&phone_count_label);

    // Spacer
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    // End session button (right side)
    let end_button = gtk::Button::with_label("End Session");
    end_button.add_css_class("end-button");
    end_button.connect_clicked(move |_| on_end());

    top_bar.append(&phone_box);
    top_bar.append(&spacer);
    top_bar.append(&end_button);

    // === Main content area ===
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 24);
    content.add_css_class("session-content");
    content.set_vexpand(true);
    content.set_margin_start(24);
    content.set_margin_end(24);

    // Left panel - video preview and capture button
    let left_panel = gtk::Box::new(gtk::Orientation::Vertical, 16);
    left_panel.set_hexpand(true);
    left_panel.add_css_class("preview-panel");

    // Video preview with overlay for capturing state
    let preview_overlay = gtk::Overlay::new();
    preview_overlay.set_vexpand(true);

    let video = gtk::Picture::new();
    video.set_paintable(Some(video_paintable));
    video.set_content_fit(gtk::ContentFit::Contain);
    video.set_hexpand(true);
    video.set_vexpand(true);
    video.add_css_class("video-preview");

    preview_overlay.set_child(Some(&video));

    // Capturing overlay
    let capturing_overlay = gtk::Box::new(gtk::Orientation::Vertical, 8);
    capturing_overlay.add_css_class("capturing-overlay");
    capturing_overlay.set_halign(gtk::Align::Center);
    capturing_overlay.set_valign(gtk::Align::Center);
    capturing_overlay.set_visible(is_capturing);

    let capturing_label = gtk::Label::new(Some("Capturing..."));
    capturing_label.add_css_class("capturing-label");
    capturing_overlay.append(&capturing_label);

    preview_overlay.add_overlay(&capturing_overlay);

    // Capture button and status
    let capture_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    capture_box.set_halign(gtk::Align::Center);
    capture_box.set_margin_top(16);
    capture_box.set_margin_bottom(16);

    let on_capture_clone = on_capture.clone();
    let capture_button = widgets::create_capture_button(move || on_capture_clone());
    capture_button.set_sensitive(!is_capturing);

    let capture_status = widgets::create_capture_status(is_capturing);

    capture_box.append(&capture_button);
    capture_box.append(&capture_status);

    left_panel.append(&preview_overlay);
    left_panel.append(&capture_box);

    // Right panel - QR codes
    let right_panel = gtk::Box::new(gtk::Orientation::Vertical, 24);
    right_panel.set_width_request(280);
    right_panel.add_css_class("qr-panel");
    right_panel.set_valign(gtk::Align::Center);

    let qr_container = gtk::Box::new(gtk::Orientation::Vertical, 24);

    let wifi_qr = widgets::create_wifi_qr_section(ctx);
    let session_qr = widgets::create_session_qr_section(ctx, session_id);

    qr_container.append(&wifi_qr);
    qr_container.append(&session_qr);
    right_panel.append(&qr_container);

    content.append(&left_panel);
    content.append(&right_panel);

    // === Bottom photo strip ===
    let photo_strip = widgets::create_photo_strip(ctx, photos, on_photo_click);
    photo_strip.set_margin_start(24);
    photo_strip.set_margin_end(24);
    photo_strip.set_margin_bottom(16);

    main_box.append(&top_bar);
    main_box.append(&content);
    main_box.append(&photo_strip);

    let widgets = SessionWidgets {
        phone_count_label,
        capture_button,
        capture_status,
        photo_strip,
        capturing_overlay,
        qr_container,
    };

    (main_box, widgets)
}

impl SessionWidgets {
    /// Update phone count display
    pub fn set_phone_count(&self, count: u32) {
        self.phone_count_label.set_text(&format!("{} connected", count));
    }

    /// Update capturing state
    pub fn set_capturing(&self, is_capturing: bool) {
        self.capturing_overlay.set_visible(is_capturing);
        self.capture_button.set_sensitive(!is_capturing);
        self.capture_status.set_text(if is_capturing {
            "Taking photos..."
        } else {
            "Tap to capture!"
        });
    }

    /// Update photo strip
    pub fn update_photos<F>(&self, ctx: &Rc<AppContext>, photos: &[PhotoInfo], on_click: F)
    where
        F: Fn(usize) + Clone + 'static,
    {
        widgets::update_photo_strip(ctx, &self.photo_strip, photos, on_click);
    }
}
