//! Capture button widget.

use gtk4 as gtk;
use gtk4::prelude::*;

use super::animations;

/// Create the capture button with press animation feedback
pub fn create_capture_button<F>(on_click: F) -> gtk::Button
where
    F: Fn() + 'static,
{
    let button = gtk::Button::new();
    button.add_css_class("capture-button");
    button.set_size_request(120, 120);

    // Inner circle with camera icon
    let inner = gtk::Box::new(gtk::Orientation::Vertical, 0);
    inner.set_halign(gtk::Align::Center);
    inner.set_valign(gtk::Align::Center);

    // Use GTK symbolic icon (works everywhere without emoji fonts)
    let icon = gtk::Image::from_icon_name("camera-photo-symbolic");
    icon.set_pixel_size(48);
    icon.add_css_class("capture-icon");

    inner.append(&icon);
    button.set_child(Some(&inner));

    // Add press animation feedback
    let button_clone = button.clone();
    button.connect_clicked(move |_| {
        // Visual feedback - quick pulse animation
        animations::button_press(&button_clone);
        on_click();
    });

    button
}
