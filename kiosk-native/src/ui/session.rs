//! Unified session screen - handles both welcome and active session states.
//! Uses overlay-based layout with floating elements and switchable main area.

use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::api::PhotoInfo;
use crate::app::AppContext;
use crate::config;
use crate::ui::widgets::{self, animations, qr_image::ExpandableQrPanel};


/// References to updateable widgets in the session screen
#[allow(dead_code)]
pub struct SessionWidgets {
    /// The main overlay container
    pub overlay: gtk::Overlay,
    /// Main area stack (switches between video and photo)
    pub main_stack: gtk::Stack,
    /// Video preview picture
    pub video_picture: gtk::Picture,
    /// Photo view picture (for viewing selected photos)
    pub photo_picture: gtk::Picture,
    /// Photo strip scroll window
    pub photo_strip: gtk::ScrolledWindow,
    /// Photo strip inner box (for in-place updates)
    photo_strip_inner: Rc<RefCell<gtk::Box>>,
    /// Currently loaded photo URLs (to avoid reloading)
    loaded_photos: Rc<RefCell<Vec<String>>>,
    /// Current selection state
    current_selection: Rc<RefCell<Option<usize>>>,
    /// Capture button
    pub capture_button: gtk::Button,
    /// Start session button (welcome mode)
    pub start_button: gtk::Button,
    /// Phone count label
    pub phone_count_label: gtk::Label,
    /// Phone count box (parent of label)
    phone_box: gtk::Box,
    /// End session button
    pub end_button: gtk::Button,
    /// QR panel (expandable)
    pub qr_panel: Rc<ExpandableQrPanel>,
    /// Welcome content box
    welcome_box: gtk::Box,
    /// Countdown overlay
    pub countdown_overlay: gtk::Box,
    /// Countdown label
    pub countdown_label: gtk::Label,
    /// Active countdown animation
    countdown_animation: Rc<RefCell<Option<adw::TimedAnimation>>>,
}

/// Create the unified session screen
pub fn create_session_screen(
    ctx: &Rc<AppContext>,
    video_paintable: &gtk::gdk::Paintable,
) -> SessionWidgets {
    // Main overlay - everything floats on top of the main area
    let overlay = gtk::Overlay::new();
    overlay.add_css_class("session-screen");
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);

    // === Main area stack (video OR photo) ===
    let main_stack = gtk::Stack::new();
    main_stack.set_transition_type(gtk::StackTransitionType::None); // No transition to prevent flicker
    main_stack.set_hexpand(true);
    main_stack.set_vexpand(true);

    // Video preview (live view)
    let video_picture = gtk::Picture::new();
    video_picture.set_paintable(Some(video_paintable));
    video_picture.set_content_fit(gtk::ContentFit::Cover);
    video_picture.set_hexpand(true);
    video_picture.set_vexpand(true);
    video_picture.add_css_class("main-video");
    main_stack.add_named(&video_picture, Some("live"));

    // Photo view (for viewing selected photos)
    let photo_picture = gtk::Picture::new();
    photo_picture.set_content_fit(gtk::ContentFit::Contain);
    photo_picture.set_hexpand(true);
    photo_picture.set_vexpand(true);
    photo_picture.add_css_class("main-photo");
    main_stack.add_named(&photo_picture, Some("photo"));

    // Start with live view
    main_stack.set_visible_child_name("live");
    overlay.set_child(Some(&main_stack));

    // Click handler on main area to collapse QR panel when expanded
    // We'll connect this after qr_panel is created

    // === Floating phone count (top-left) - session only ===
    let phone_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    phone_box.add_css_class("floating-status");
    phone_box.set_halign(gtk::Align::Start);
    phone_box.set_valign(gtk::Align::Start);
    phone_box.set_margin_start(24);
    phone_box.set_margin_top(24);
    phone_box.set_visible(false);

    let phone_icon = gtk::Image::from_icon_name("phone-symbolic");
    phone_icon.set_pixel_size(24);
    phone_icon.add_css_class("phone-icon");

    let phone_count_label = gtk::Label::new(Some("0 connected"));
    phone_count_label.add_css_class("phone-count");

    phone_box.append(&phone_icon);
    phone_box.append(&phone_count_label);
    overlay.add_overlay(&phone_box);

    // === Floating end session button (top-left, below status) - session only ===
    let end_button = gtk::Button::with_label("End Session");
    end_button.add_css_class("end-button");
    end_button.add_css_class("floating-button");
    end_button.set_halign(gtk::Align::Start);
    end_button.set_valign(gtk::Align::Start);
    end_button.set_margin_start(24);
    end_button.set_margin_top(70);
    end_button.set_visible(false);
    overlay.add_overlay(&end_button);

    // === Welcome content (center) - welcome only ===
    let welcome_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
    welcome_box.set_halign(gtk::Align::Center);
    welcome_box.set_valign(gtk::Align::Center);
    welcome_box.add_css_class("welcome-content");

    let icon_frame = gtk::Frame::new(None);
    icon_frame.add_css_class("welcome-icon");
    icon_frame.set_halign(gtk::Align::Center);

    let icon_image = gtk::Image::from_icon_name("camera-photo-symbolic");
    icon_image.set_pixel_size(64);
    icon_image.add_css_class("welcome-icon-image");
    icon_frame.set_child(Some(&icon_image));

    let title = gtk::Label::new(Some("PicPop"));
    title.add_css_class("welcome-title");

    let subtitle = gtk::Label::new(Some("Photo Booth"));
    subtitle.add_css_class("welcome-subtitle");

    let start_button = gtk::Button::with_label("Start Session");
    start_button.add_css_class("start-button");

    welcome_box.append(&icon_frame);
    welcome_box.append(&title);
    welcome_box.append(&subtitle);
    welcome_box.append(&start_button);
    overlay.add_overlay(&welcome_box);

    // === Floating capture button (bottom center) - session/live only ===
    let capture_button = widgets::create_capture_button(|| {});
    capture_button.set_halign(gtk::Align::Center);
    capture_button.set_valign(gtk::Align::End);
    capture_button.set_margin_bottom(160);
    capture_button.set_visible(false);
    overlay.add_overlay(&capture_button);

    // === QR panel (top-right, small and expandable) ===
    let qr_panel = ExpandableQrPanel::new(ctx);
    overlay.add_overlay(&qr_panel.panel);

    // Click on main area collapses QR panel
    let qr_panel_for_main = qr_panel.clone();
    let main_gesture = gtk::GestureClick::new();
    main_gesture.connect_released(move |_, _, _, _| {
        qr_panel_for_main.collapse();
    });
    main_stack.add_controller(main_gesture);

    // === Photo strip (bottom) - session only ===
    let photo_strip = gtk::ScrolledWindow::new();
    photo_strip.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    photo_strip.set_min_content_height(120);
    photo_strip.add_css_class("photo-strip");
    photo_strip.set_kinetic_scrolling(true);
    photo_strip.set_halign(gtk::Align::Fill);
    photo_strip.set_valign(gtk::Align::End);
    photo_strip.set_margin_start(24);
    photo_strip.set_margin_end(24);
    photo_strip.set_margin_bottom(16);
    photo_strip.set_visible(false);

    // Create initial empty inner box
    let photo_strip_inner = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    photo_strip_inner.set_margin_start(12);
    photo_strip_inner.set_margin_end(12);
    photo_strip_inner.add_css_class("photo-strip-inner");
    photo_strip.set_child(Some(&photo_strip_inner));

    overlay.add_overlay(&photo_strip);

    // === Countdown overlay (center, over everything) ===
    let countdown_overlay = gtk::Box::new(gtk::Orientation::Vertical, 0);
    countdown_overlay.add_css_class("countdown-overlay");
    countdown_overlay.set_halign(gtk::Align::Fill);
    countdown_overlay.set_valign(gtk::Align::Fill);
    countdown_overlay.set_visible(false);

    let countdown_label = gtk::Label::new(Some("3"));
    countdown_label.add_css_class("countdown-number");
    countdown_label.set_halign(gtk::Align::Center);
    countdown_label.set_valign(gtk::Align::Center);
    countdown_label.set_hexpand(true);
    countdown_label.set_vexpand(true);
    countdown_overlay.append(&countdown_label);

    overlay.add_overlay(&countdown_overlay);

    SessionWidgets {
        overlay,
        main_stack,
        video_picture,
        photo_picture,
        photo_strip,
        photo_strip_inner: Rc::new(RefCell::new(photo_strip_inner)),
        loaded_photos: Rc::new(RefCell::new(Vec::new())),
        current_selection: Rc::new(RefCell::new(None)),
        capture_button,
        start_button,
        phone_count_label,
        phone_box,
        end_button,
        qr_panel,
        welcome_box,
        countdown_overlay,
        countdown_label,
        countdown_animation: Rc::new(RefCell::new(None)),
    }
}

impl SessionWidgets {
    /// Configure for welcome mode (with animations)
    pub fn set_welcome_mode(&self) {
        // Fade out session elements
        animations::fade_out(&self.phone_box, animations::duration::FAST);
        animations::fade_out(&self.end_button, animations::duration::FAST);
        animations::fade_out(&self.capture_button, animations::duration::FAST);
        animations::fade_out(&self.photo_strip, animations::duration::FAST);
        self.countdown_overlay.set_visible(false);

        // Fade in welcome elements
        animations::fade_in(&self.welcome_box, animations::duration::NORMAL);

        // Show live video
        self.main_stack.set_visible_child_name("live");

        // Collapse QR panel and hide session QR
        self.qr_panel.collapse();
        self.qr_panel.hide_session();
    }

    /// Configure for session mode (with animations)
    pub fn set_session_mode(&self, ctx: &Rc<AppContext>, session_id: &str) {
        // Fade out welcome elements
        animations::fade_out(&self.welcome_box, animations::duration::FAST);

        // Fade in session elements with staggered timing
        animations::fade_in(&self.phone_box, animations::duration::NORMAL);
        animations::fade_in(&self.end_button, animations::duration::NORMAL);
        animations::fade_in(&self.capture_button, animations::duration::NORMAL);
        animations::slide_in_from_bottom(&self.photo_strip, animations::duration::NORMAL);
        self.countdown_overlay.set_visible(false);

        // Update QR panel with session QR
        self.qr_panel.set_session(ctx, session_id);

        // Show live video
        self.main_stack.set_visible_child_name("live");
    }

    /// Update phone count display
    pub fn set_phone_count(&self, count: u32) {
        self.phone_count_label.set_text(&format!("{} connected", count));
    }

    /// Update the photo strip in-place (no flashing)
    pub fn update_photos<F1, F2>(
        &self,
        ctx: &Rc<AppContext>,
        photos: &[PhotoInfo],
        selection: Option<usize>,
        on_live: F1,
        on_photo: F2,
    ) where
        F1: Fn() + Clone + 'static,
        F2: Fn(usize) + Clone + 'static,
    {
        let mut loaded = self.loaded_photos.borrow_mut();
        let mut current_sel = self.current_selection.borrow_mut();
        let inner = self.photo_strip_inner.borrow();

        // Check if we need to add new photos
        let photo_urls: Vec<String> = photos.iter().map(|p| p.thumbnail_url.clone()).collect();
        let needs_rebuild = loaded.is_empty()
            || photo_urls.len() < loaded.len()
            || (!photo_urls.is_empty() && !loaded.is_empty() && photo_urls[0] != loaded[0]);

        if needs_rebuild {
            // Full rebuild needed (first time, or photos changed significantly)
            // Clear existing
            while let Some(child) = inner.first_child() {
                inner.remove(&child);
            }
            loaded.clear();

            // Add LIVE tile
            let live_tile = create_live_tile(selection.is_none(), on_live.clone());
            inner.append(&live_tile);

            // Add photo thumbnails
            for (idx, photo) in photos.iter().enumerate() {
                let is_selected = selection == Some(idx);
                let thumb = create_photo_thumbnail(ctx, photo, is_selected, {
                    let on_click = on_photo.clone();
                    move || on_click(idx)
                });
                inner.append(&thumb);
                loaded.push(photo.thumbnail_url.clone());
            }

            *current_sel = selection;
        } else {
            // Incremental update - just add new photos and update selection
            let existing_count = loaded.len();

            // Add any new photos with slide-in animation
            for (idx, photo) in photos.iter().enumerate().skip(existing_count) {
                let is_selected = selection == Some(idx);
                let thumb = create_photo_thumbnail(ctx, photo, is_selected, {
                    let on_click = on_photo.clone();
                    move || on_click(idx)
                });
                inner.append(&thumb);
                loaded.push(photo.thumbnail_url.clone());

                // Animate the new thumbnail sliding in
                animations::slide_in_from_right(&thumb, animations::duration::NORMAL);
            }

            // Update selection if changed
            if *current_sel != selection {
                // Update LIVE tile selection
                if let Some(live_tile) = inner.first_child() {
                    if selection.is_none() {
                        live_tile.add_css_class("selected");
                    } else {
                        live_tile.remove_css_class("selected");
                    }
                }

                // Update photo selections
                let mut child = inner.first_child();
                let mut idx = 0;
                while let Some(widget) = child {
                    if idx > 0 {
                        // Skip LIVE tile (idx 0)
                        let photo_idx = idx - 1;
                        if selection == Some(photo_idx) {
                            widget.add_css_class("selected");
                        } else {
                            widget.remove_css_class("selected");
                        }
                    }
                    child = widget.next_sibling();
                    idx += 1;
                }

                *current_sel = selection;
            }
        }
    }

    /// Show live video view
    pub fn show_live_view(&self) {
        // Only switch if not already on live
        if self.main_stack.visible_child_name().as_deref() != Some("live") {
            self.main_stack.set_visible_child_name("live");
        }
        self.capture_button.set_visible(true);
    }

    /// Show a photo in the main area
    pub fn show_photo(&self, ctx: &Rc<AppContext>, photo: &PhotoInfo) {
        // Clear the old photo immediately to avoid showing stale content
        self.photo_picture.set_paintable(None::<&gtk::gdk::Paintable>);
        self.photo_picture.set_opacity(0.0);

        // Only switch if not already on photo
        if self.main_stack.visible_child_name().as_deref() != Some("photo") {
            self.main_stack.set_visible_child_name("photo");
        }
        self.capture_button.set_visible(false);

        // Load the photo
        let url = config::photo_url(&photo.web_url);
        let picture = self.photo_picture.clone();
        let api = ctx.api.clone();
        let runtime = ctx.runtime.clone();

        glib::spawn_future_local(async move {
            let result = runtime.spawn(async move { api.fetch_image(&url).await }).await;

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
                        // Fade in the new photo
                        animations::fade_in(&picture, animations::duration::FAST);
                    }
                }
                Ok(Err(e)) => log::error!("Failed to load photo: {}", e),
                Err(e) => log::error!("Task join error: {}", e),
            }
        });
    }

    /// Show countdown overlay with animated number
    pub fn show_countdown(&self, value: u32) {
        // Cancel any existing countdown animation
        if let Some(ref anim) = *self.countdown_animation.borrow() {
            anim.skip();
        }

        // Show overlay with fade if not visible
        if !self.countdown_overlay.is_visible() {
            self.countdown_overlay.set_opacity(0.0);
            self.countdown_overlay.set_visible(true);
            animations::fade_in(&self.countdown_overlay, animations::duration::FAST);
        }

        self.capture_button.set_sensitive(false);

        // Animate the countdown number (scale down + fade in)
        self.countdown_label.set_text(&value.to_string());
        self.countdown_label.set_opacity(0.0);

        let label = self.countdown_label.clone();
        let target = adw::CallbackAnimationTarget::new(move |progress| {
            // Opacity: quick fade in during first 30%
            let opacity = if progress < 0.3 {
                progress / 0.3
            } else {
                1.0
            };
            label.set_opacity(opacity);
        });

        let animation = adw::TimedAnimation::builder()
            .widget(&self.countdown_label)
            .value_from(0.0)
            .value_to(1.0)
            .duration(animations::duration::COUNTDOWN)
            .easing(adw::Easing::EaseOutCubic)
            .target(&target)
            .build();

        animation.play();
        *self.countdown_animation.borrow_mut() = Some(animation);
    }

    /// Hide countdown overlay with fade out
    pub fn hide_countdown(&self) {
        // Cancel any running countdown animation
        if let Some(ref anim) = *self.countdown_animation.borrow() {
            anim.skip();
        }
        *self.countdown_animation.borrow_mut() = None;

        if self.countdown_overlay.is_visible() {
            animations::fade_out(&self.countdown_overlay, animations::duration::FAST);
        }
        self.capture_button.set_sensitive(true);
    }

    /// Show processing state
    pub fn show_processing(&self) {
        self.countdown_label.set_text("...");
        self.countdown_overlay.set_visible(true);
        self.capture_button.set_sensitive(false);
    }

    /// Update start button state
    pub fn set_start_loading(&self, is_loading: bool, error: Option<&str>) {
        self.start_button.set_sensitive(!is_loading);
        if let Some(err) = error {
            self.start_button.set_label(err);
        } else if is_loading {
            self.start_button.set_label("Starting...");
        } else {
            self.start_button.set_label("Start Session");
        }
    }

    /// Connect start button click handler
    pub fn connect_start<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.start_button.connect_clicked(move |_| callback());
    }

    /// Connect end button click handler
    pub fn connect_end<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.end_button.connect_clicked(move |_| callback());
    }

    /// Connect capture button click handler
    pub fn connect_capture<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.capture_button.connect_clicked(move |_| callback());
    }

    /// Connect tap on photo to return to live
    pub fn connect_photo_tap<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        let gesture = gtk::GestureClick::new();
        gesture.connect_released(move |_, _, _, _| callback());
        self.photo_picture.add_controller(gesture);
    }
}

/// Create the LIVE tile button
fn create_live_tile<F>(is_selected: bool, on_click: F) -> gtk::Button
where
    F: Fn() + 'static,
{
    let button = gtk::Button::new();
    button.add_css_class("photo-thumbnail");
    button.add_css_class("live-tile");
    if is_selected {
        button.add_css_class("selected");
    }
    button.set_size_request(100, 100);

    // LIVE label with icon
    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    content.set_halign(gtk::Align::Center);
    content.set_valign(gtk::Align::Center);

    let icon = gtk::Image::from_icon_name("camera-video-symbolic");
    icon.set_pixel_size(32);
    icon.add_css_class("live-icon");

    let label = gtk::Label::new(Some("LIVE"));
    label.add_css_class("live-label");

    content.append(&icon);
    content.append(&label);

    button.set_child(Some(&content));
    button.connect_clicked(move |_| on_click());

    button
}

/// Create a single photo thumbnail
fn create_photo_thumbnail<F>(
    ctx: &Rc<AppContext>,
    photo: &PhotoInfo,
    is_selected: bool,
    on_click: F,
) -> gtk::Button
where
    F: Fn() + 'static,
{
    let button = gtk::Button::new();
    button.add_css_class("photo-thumbnail");
    if is_selected {
        button.add_css_class("selected");
    }
    button.set_size_request(100, 100);

    let picture = gtk::Picture::new();
    picture.set_size_request(100, 100);
    picture.set_content_fit(gtk::ContentFit::Cover);
    picture.set_opacity(0.0); // Start invisible for fade-in

    // Load thumbnail using spawn_future_local to stay on main thread
    let url = config::photo_url(&photo.thumbnail_url);
    let picture_clone = picture.clone();
    let api = ctx.api.clone();
    let runtime = ctx.runtime.clone();

    glib::spawn_future_local(async move {
        let result = runtime.spawn(async move { api.fetch_image(&url).await }).await;

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
                    // Fade in the thumbnail once loaded
                    animations::fade_in(&picture_clone, animations::duration::FAST);
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
