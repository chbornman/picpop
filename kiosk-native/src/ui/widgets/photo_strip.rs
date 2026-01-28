//! Horizontal photo strip/gallery widget.

use gtk4 as gtk;
use gtk4::prelude::*;
use std::rc::Rc;

use crate::api::PhotoInfo;
use crate::app::AppContext;
use crate::config;

/// Create a horizontal photo strip
pub fn create_photo_strip<F>(ctx: &Rc<AppContext>, photos: &[PhotoInfo], on_photo_click: F) -> gtk::ScrolledWindow
where
    F: Fn(usize) + Clone + 'static,
{
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    scroll.set_min_content_height(120);
    scroll.add_css_class("photo-strip");

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    hbox.add_css_class("photo-strip-inner");

    for (idx, photo) in photos.iter().enumerate() {
        let thumb = create_photo_thumbnail(ctx, photo, {
            let on_click = on_photo_click.clone();
            move || on_click(idx)
        });
        hbox.append(&thumb);
    }

    scroll.set_child(Some(&hbox));
    scroll
}

/// Create a single photo thumbnail
fn create_photo_thumbnail<F>(ctx: &Rc<AppContext>, photo: &PhotoInfo, on_click: F) -> gtk::Button
where
    F: Fn() + 'static,
{
    let button = gtk::Button::new();
    button.add_css_class("photo-thumbnail");
    button.set_size_request(100, 100);

    let picture = gtk::Picture::new();
    picture.set_size_request(100, 100);
    picture.set_content_fit(gtk::ContentFit::Cover);

    // Load thumbnail using spawn_future_local to stay on main thread
    let url = config::photo_url(&photo.thumbnail_url);
    let picture_clone = picture.clone();
    let api = ctx.api.clone();
    let runtime = ctx.runtime.clone();

    glib::spawn_future_local(async move {
        let result = runtime.spawn(async move {
            api.fetch_image(&url).await
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
                    picture_clone.set_paintable(Some(&texture));
                }
            }
            Ok(Err(e)) => {
                log::error!("Failed to load thumbnail: {}", e);
            }
            Err(e) => {
                log::error!("Task join error: {}", e);
            }
        }
    });

    button.set_child(Some(&picture));
    button.connect_clicked(move |_| on_click());

    button
}

/// Update the photo strip with new photos
pub fn update_photo_strip<F>(
    ctx: &Rc<AppContext>,
    strip: &gtk::ScrolledWindow,
    photos: &[PhotoInfo],
    on_photo_click: F,
) where
    F: Fn(usize) + Clone + 'static,
{
    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hbox.set_margin_start(12);
    hbox.set_margin_end(12);
    hbox.add_css_class("photo-strip-inner");

    for (idx, photo) in photos.iter().enumerate() {
        let thumb = create_photo_thumbnail(ctx, photo, {
            let on_click = on_photo_click.clone();
            move || on_click(idx)
        });
        hbox.append(&thumb);
    }

    strip.set_child(Some(&hbox));

    // Scroll to end to show newest photo
    let adj = strip.hadjustment();
    glib::idle_add_local_once(move || {
        adj.set_value(adj.upper() - adj.page_size());
    });
}
