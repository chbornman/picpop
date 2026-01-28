//! Full-screen photo lightbox viewer.

use gtk4 as gtk;
use gtk4::prelude::*;
use std::rc::Rc;

use crate::api::PhotoInfo;
use crate::app::AppContext;
use crate::config;

/// Create the lightbox overlay
pub fn create_lightbox(
    ctx: &Rc<AppContext>,
    photos: &[PhotoInfo],
    initial_index: usize,
    on_close: impl Fn() + Clone + 'static,
    on_navigate: impl Fn(usize) + Clone + 'static,
) -> gtk::Box {
    let lightbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    lightbox.add_css_class("lightbox");
    lightbox.set_hexpand(true);
    lightbox.set_vexpand(true);

    // Top bar with close button and counter
    let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    top_bar.add_css_class("lightbox-top-bar");
    top_bar.set_margin_start(24);
    top_bar.set_margin_end(24);
    top_bar.set_margin_top(16);

    // Photo counter
    let counter = gtk::Label::new(Some(&format!(
        "{} / {}",
        initial_index + 1,
        photos.len()
    )));
    counter.add_css_class("lightbox-counter");

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    // Close button
    let on_close_clone = on_close.clone();
    let close_button = gtk::Button::with_label("\u{2715}");
    close_button.add_css_class("lightbox-close");
    close_button.connect_clicked(move |_| on_close_clone());

    top_bar.append(&counter);
    top_bar.append(&spacer);
    top_bar.append(&close_button);

    // Main image area with navigation
    let image_area = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    image_area.set_vexpand(true);

    // Previous button
    let prev_button = gtk::Button::with_label("\u{2039}");
    prev_button.add_css_class("lightbox-nav");
    prev_button.add_css_class("lightbox-prev");
    prev_button.set_valign(gtk::Align::Center);
    prev_button.set_sensitive(initial_index > 0);

    // Image
    let picture = gtk::Picture::new();
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.add_css_class("lightbox-image");

    // Load initial image
    if let Some(photo) = photos.get(initial_index) {
        load_lightbox_image(ctx, &photo.web_url, &picture);
    }

    // Next button
    let next_button = gtk::Button::with_label("\u{203A}");
    next_button.add_css_class("lightbox-nav");
    next_button.add_css_class("lightbox-next");
    next_button.set_valign(gtk::Align::Center);
    next_button.set_sensitive(initial_index < photos.len().saturating_sub(1));

    // Navigation callbacks
    let on_navigate_prev = on_navigate.clone();
    let current_idx = initial_index;
    prev_button.connect_clicked(move |_| {
        if current_idx > 0 {
            on_navigate_prev(current_idx - 1);
        }
    });

    let on_navigate_next = on_navigate.clone();
    let photo_count = photos.len();
    next_button.connect_clicked(move |_| {
        if current_idx < photo_count.saturating_sub(1) {
            on_navigate_next(current_idx + 1);
        }
    });

    image_area.append(&prev_button);
    image_area.append(&picture);
    image_area.append(&next_button);

    lightbox.append(&top_bar);
    lightbox.append(&image_area);

    // Keyboard navigation
    let key_controller = gtk::EventControllerKey::new();
    let on_close_key = on_close.clone();
    let on_nav_key = on_navigate.clone();
    let idx = initial_index;
    let count = photos.len();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        match key {
            gtk::gdk::Key::Escape => {
                on_close_key();
                glib::Propagation::Stop
            }
            gtk::gdk::Key::Left if idx > 0 => {
                on_nav_key(idx - 1);
                glib::Propagation::Stop
            }
            gtk::gdk::Key::Right if idx < count.saturating_sub(1) => {
                on_nav_key(idx + 1);
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed
        }
    });
    lightbox.add_controller(key_controller);

    lightbox
}

/// Load an image into the lightbox picture widget
fn load_lightbox_image(ctx: &Rc<AppContext>, url: &str, picture: &gtk::Picture) {
    let full_url = config::photo_url(url);
    let picture = picture.clone();
    let api = ctx.api.clone();
    let runtime = ctx.runtime.clone();

    glib::spawn_future_local(async move {
        let result = runtime.spawn(async move {
            api.fetch_image(&full_url).await
        }).await;

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
                log::error!("Failed to load lightbox image: {}", e);
            }
            Err(e) => {
                log::error!("Task join error: {}", e);
            }
        }
    });
}

/// Update lightbox to show a different photo
pub fn update_lightbox(
    ctx: &Rc<AppContext>,
    lightbox: &gtk::Box,
    photos: &[PhotoInfo],
    index: usize,
) {
    // Find and update the counter
    if let Some(top_bar) = lightbox.first_child() {
        if let Some(bar) = top_bar.downcast_ref::<gtk::Box>() {
            if let Some(counter) = bar.first_child() {
                if let Some(label) = counter.downcast_ref::<gtk::Label>() {
                    label.set_text(&format!("{} / {}", index + 1, photos.len()));
                }
            }
        }
    }

    // Find and update the image
    if let Some(child) = lightbox.first_child() {
        let mut sibling = child.next_sibling();
        while let Some(widget) = sibling {
            if let Some(image_area) = widget.downcast_ref::<gtk::Box>() {
                // Find the picture in the image area
                let mut img_child = image_area.first_child();
                while let Some(img_widget) = img_child {
                    if let Some(picture) = img_widget.downcast_ref::<gtk::Picture>() {
                        if let Some(photo) = photos.get(index) {
                            load_lightbox_image(ctx, &photo.web_url, picture);
                        }
                        break;
                    }
                    // Update nav button sensitivity
                    if let Some(button) = img_widget.downcast_ref::<gtk::Button>() {
                        if button.css_classes().iter().any(|c| c == "lightbox-prev") {
                            button.set_sensitive(index > 0);
                        } else if button.css_classes().iter().any(|c| c == "lightbox-next") {
                            button.set_sensitive(index < photos.len().saturating_sub(1));
                        }
                    }
                    img_child = img_widget.next_sibling();
                }
                break;
            }
            sibling = widget.next_sibling();
        }
    }
}
