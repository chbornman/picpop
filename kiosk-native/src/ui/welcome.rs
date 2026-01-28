//! Welcome screen - initial kiosk view with start button.

use gtk4 as gtk;
use gtk4::prelude::*;
use std::rc::Rc;

use crate::app::AppContext;

/// Create the welcome screen
pub fn create_welcome_screen(
    _ctx: &Rc<AppContext>,
    video_paintable: &gtk::gdk::Paintable,
    is_loading: bool,
    on_start: impl Fn() + 'static,
) -> gtk::Overlay {
    let overlay = gtk::Overlay::new();
    overlay.add_css_class("welcome-screen");

    // Video background
    let video = gtk::Picture::new();
    video.set_paintable(Some(video_paintable));
    video.set_content_fit(gtk::ContentFit::Cover);
    video.set_hexpand(true);
    video.set_vexpand(true);
    video.add_css_class("video-background");

    overlay.set_child(Some(&video));

    // Center content overlay
    let center_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
    center_box.set_halign(gtk::Align::Center);
    center_box.set_valign(gtk::Align::Center);
    center_box.add_css_class("welcome-content");

    // Logo/icon
    let icon_frame = gtk::Frame::new(None);
    icon_frame.add_css_class("welcome-icon");
    icon_frame.set_halign(gtk::Align::Center);

    let icon_label = gtk::Label::new(Some("\u{1F4F7}"));
    icon_label.add_css_class("welcome-icon-emoji");
    icon_frame.set_child(Some(&icon_label));

    // Title
    let title = gtk::Label::new(Some("PicPop"));
    title.add_css_class("welcome-title");

    // Subtitle
    let subtitle = gtk::Label::new(Some("Photo Booth"));
    subtitle.add_css_class("welcome-subtitle");

    // Start button
    let button = gtk::Button::new();
    button.add_css_class("start-button");
    button.set_sensitive(!is_loading);

    let button_label = if is_loading {
        "Starting..."
    } else {
        "Start Session"
    };
    button.set_label(button_label);

    button.connect_clicked(move |btn| {
        btn.set_sensitive(false);
        btn.set_label("Starting...");
        on_start();
    });

    center_box.append(&icon_frame);
    center_box.append(&title);
    center_box.append(&subtitle);
    center_box.append(&button);

    overlay.add_overlay(&center_box);

    overlay
}

/// Update the start button state
pub fn update_start_button(screen: &gtk::Overlay, is_loading: bool, error: Option<&str>) {
    // Find the button in the overlay
    let mut child = screen.first_child();
    while let Some(widget) = child {
        if widget.css_classes().iter().any(|c| c == "welcome-content") {
            // Found the content box, look for button
            if let Some(vbox) = widget.downcast_ref::<gtk::Box>() {
                let mut btn_child = vbox.first_child();
                while let Some(btn_widget) = btn_child {
                    if let Some(button) = btn_widget.downcast_ref::<gtk::Button>() {
                        button.set_sensitive(!is_loading);
                        if let Some(err) = error {
                            button.set_label(err);
                        } else if is_loading {
                            button.set_label("Starting...");
                        } else {
                            button.set_label("Start Session");
                        }
                        return;
                    }
                    btn_child = btn_widget.next_sibling();
                }
            }
        }
        child = widget.next_sibling();
    }
}
