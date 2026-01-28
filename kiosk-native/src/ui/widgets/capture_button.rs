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
    let inner = gtk::Box::new(gtk::Orientation::Vertical, 8);
    inner.set_halign(gtk::Align::Center);
    inner.set_valign(gtk::Align::Center);

    // Camera icon (using Unicode camera emoji as fallback, could use actual icon)
    let icon = gtk::Label::new(Some("\u{1F4F7}"));
    icon.add_css_class("capture-icon");

    inner.append(&icon);
    button.set_child(Some(&inner));

    button.connect_clicked(move |_| {
        on_click();
    });

    button
}

/// Create a status label below the capture button
pub fn create_capture_status(is_capturing: bool) -> gtk::Label {
    let text = if is_capturing {
        "Taking photos..."
    } else {
        "Tap to capture!"
    };

    let label = gtk::Label::new(Some(text));
    label.add_css_class("capture-status");

    label
}
