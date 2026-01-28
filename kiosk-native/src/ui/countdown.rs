//! Countdown overlay for photo capture.

use gtk4 as gtk;
use gtk4::prelude::*;

/// Create the countdown overlay
pub fn create_countdown_overlay(value: u32) -> gtk::Box {
    let overlay_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
    overlay_box.add_css_class("countdown-overlay");
    overlay_box.set_halign(gtk::Align::Fill);
    overlay_box.set_valign(gtk::Align::Fill);
    overlay_box.set_hexpand(true);
    overlay_box.set_vexpand(true);

    // Center content
    let center = gtk::Box::new(gtk::Orientation::Vertical, 16);
    center.set_halign(gtk::Align::Center);
    center.set_valign(gtk::Align::Center);
    center.set_vexpand(true);

    // Countdown number with ring effect
    let number_container = gtk::Overlay::new();
    number_container.set_halign(gtk::Align::Center);

    // Ring effect (background circle that pulses)
    let ring = gtk::Frame::new(None);
    ring.add_css_class("countdown-ring");
    ring.set_halign(gtk::Align::Center);
    ring.set_valign(gtk::Align::Center);
    number_container.set_child(Some(&ring));

    // The number itself
    let number = gtk::Label::new(Some(&value.to_string()));
    number.add_css_class("countdown-number");
    number_container.add_overlay(&number);

    // "Get ready!" text
    let ready_label = gtk::Label::new(Some("Get ready!"));
    ready_label.add_css_class("countdown-ready");

    center.append(&number_container);
    center.append(&ready_label);

    overlay_box.append(&center);

    overlay_box
}

/// Update the countdown number
pub fn update_countdown(overlay: &gtk::Box, value: u32) {
    // Find the number label and update it
    if let Some(center) = overlay.first_child() {
        if let Some(center_box) = center.downcast_ref::<gtk::Box>() {
            if let Some(number_container) = center_box.first_child() {
                if let Some(container) = number_container.downcast_ref::<gtk::Overlay>() {
                    // Find the label in the overlay
                    let mut child = container.first_child();
                    while let Some(widget) = child {
                        if let Some(label) = widget.downcast_ref::<gtk::Label>() {
                            if label.css_classes().iter().any(|c| c == "countdown-number") {
                                label.set_text(&value.to_string());
                                // Trigger animation by toggling a class
                                label.remove_css_class("countdown-animate");
                                label.add_css_class("countdown-animate");
                                return;
                            }
                        }
                        child = widget.next_sibling();
                    }
                }
            }
        }
    }
}
