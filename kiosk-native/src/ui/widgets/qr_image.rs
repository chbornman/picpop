//! QR code image widget that loads from URL.

use gtk4 as gtk;
use gtk4::prelude::*;
use std::rc::Rc;

use crate::app::AppContext;
use crate::config;

/// Create a QR code image widget
pub fn create_qr_image(
    ctx: &Rc<AppContext>,
    url: &str,
    size: i32,
) -> gtk::Picture {
    let picture = gtk::Picture::new();
    picture.set_size_request(size, size);
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.add_css_class("qr-image");

    // Load the image
    load_image_into_picture(ctx, url, &picture);

    picture
}

/// Load an image from URL into a Picture widget
/// This uses glib::spawn_future_local to stay on the main thread
pub fn load_image_into_picture(ctx: &Rc<AppContext>, url: &str, picture: &gtk::Picture) {
    let full_url = if url.starts_with("http") {
        url.to_string()
    } else {
        format!("{}{}", config::API_BASE, url)
    };

    let picture = picture.clone();
    let api = ctx.api.clone();
    let runtime = ctx.runtime.clone();

    // Use spawn_future_local to keep everything on the main thread
    glib::spawn_future_local(async move {
        // Run the HTTP request in the tokio runtime
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
                log::error!("Failed to load image: {}", e);
            }
            Err(e) => {
                log::error!("Task join error: {}", e);
            }
        }
    });
}

/// Create a WiFi QR code widget with label
pub fn create_wifi_qr_section(ctx: &Rc<AppContext>) -> gtk::Box {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
    vbox.add_css_class("qr-section");

    let title = gtk::Label::new(Some("Join WiFi"));
    title.add_css_class("qr-title");

    let subtitle = gtk::Label::new(Some("Scan to connect to \"PicPop\""));
    subtitle.add_css_class("qr-subtitle");

    let qr = create_qr_image(ctx, &config::wifi_qr_url(), 200);

    vbox.append(&title);
    vbox.append(&qr);
    vbox.append(&subtitle);

    vbox
}

/// Create a session QR code widget with label
pub fn create_session_qr_section(ctx: &Rc<AppContext>, session_id: &str) -> gtk::Box {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
    vbox.add_css_class("qr-section");

    let title = gtk::Label::new(Some("Get Photos"));
    title.add_css_class("qr-title");

    let subtitle = gtk::Label::new(Some("Scan to view & download"));
    subtitle.add_css_class("qr-subtitle");

    let qr = create_qr_image(ctx, &config::session_qr_url(session_id), 200);

    vbox.append(&title);
    vbox.append(&qr);
    vbox.append(&subtitle);

    vbox
}
