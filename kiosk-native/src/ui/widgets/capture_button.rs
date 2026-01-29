//! Capture button widget.

use gtk4 as gtk;
use gtk4::prelude::*;

/// Create the capture button
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

    button.connect_clicked(move |_| {
        on_click();
    });

    button
}
