//! Animation utilities using libadwaita.
//!
//! Provides reusable animation helpers for common UI transitions.

#![allow(dead_code)]

use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Animation durations (in milliseconds)
pub mod duration {
    pub const FAST: u32 = 150;
    pub const NORMAL: u32 = 250;
    pub const SLOW: u32 = 400;
    pub const COUNTDOWN: u32 = 800;
}

/// Animate a widget's opacity (fade in/out)
pub fn fade(
    widget: &impl IsA<gtk::Widget>,
    from: f64,
    to: f64,
    duration_ms: u32,
    on_complete: Option<Box<dyn Fn()>>,
) -> adw::TimedAnimation {
    widget.set_opacity(from);

    let widget_clone = widget.clone().upcast::<gtk::Widget>();
    let target = adw::CallbackAnimationTarget::new(move |value| {
        widget_clone.set_opacity(value);
    });

    let animation = adw::TimedAnimation::builder()
        .widget(widget)
        .value_from(from)
        .value_to(to)
        .duration(duration_ms)
        .easing(adw::Easing::EaseOutCubic)
        .target(&target)
        .build();

    if let Some(callback) = on_complete {
        animation.connect_done(move |_| callback());
    }

    animation.play();
    animation
}

/// Fade in a widget (0 -> 1 opacity)
pub fn fade_in(widget: &impl IsA<gtk::Widget>, duration_ms: u32) -> adw::TimedAnimation {
    widget.set_visible(true);
    fade(widget, 0.0, 1.0, duration_ms, None)
}

/// Fade out a widget (1 -> 0 opacity), then hide it
pub fn fade_out(widget: &impl IsA<gtk::Widget>, duration_ms: u32) -> adw::TimedAnimation {
    let widget_clone = widget.clone().upcast::<gtk::Widget>();
    fade(
        widget,
        1.0,
        0.0,
        duration_ms,
        Some(Box::new(move || {
            widget_clone.set_visible(false);
            widget_clone.set_opacity(1.0); // Reset for next show
        })),
    )
}

/// Animate widget scale using CSS transform
/// Note: Requires the widget to support CSS transforms
pub fn scale_bounce(widget: &impl IsA<gtk::Widget>, duration_ms: u32) -> adw::TimedAnimation {
    let widget_clone = widget.clone().upcast::<gtk::Widget>();

    // Scale down then back up (0 -> 0.5 -> 1.0 maps to 1.0 -> 0.9 -> 1.0)
    let target = adw::CallbackAnimationTarget::new(move |value| {
        // value goes 0 -> 1, we want scale: 1.0 -> 0.9 -> 1.0
        let scale = if value < 0.5 {
            1.0 - (value * 0.2) // 1.0 -> 0.9
        } else {
            0.9 + ((value - 0.5) * 0.2) // 0.9 -> 1.0
        };
        // Apply scale via CSS class or direct style
        // GTK4 doesn't have direct scale, so we use size hints
        let current_width = widget_clone.width();
        let current_height = widget_clone.height();
        if current_width > 0 && current_height > 0 {
            widget_clone.set_size_request(
                (current_width as f64 * scale) as i32,
                (current_height as f64 * scale) as i32,
            );
        }
    });

    let animation = adw::TimedAnimation::builder()
        .widget(widget)
        .value_from(0.0)
        .value_to(1.0)
        .duration(duration_ms)
        .easing(adw::Easing::EaseOutBack)
        .target(&target)
        .build();

    animation.play();
    animation
}

/// Countdown animation - scale down with fade for dramatic effect
pub struct CountdownAnimator {
    label: gtk::Label,
    overlay: gtk::Box,
    animation: Rc<RefCell<Option<adw::TimedAnimation>>>,
}

impl CountdownAnimator {
    pub fn new(label: gtk::Label, overlay: gtk::Box) -> Self {
        Self {
            label,
            overlay,
            animation: Rc::new(RefCell::new(None)),
        }
    }

    /// Show the countdown overlay with fade-in
    pub fn show(&self) {
        self.overlay.set_opacity(0.0);
        self.overlay.set_visible(true);
        fade(&self.overlay, 0.0, 1.0, duration::FAST, None);
    }

    /// Hide the countdown overlay with fade-out
    pub fn hide(&self) {
        let overlay = self.overlay.clone();
        fade(
            &self.overlay,
            1.0,
            0.0,
            duration::FAST,
            Some(Box::new(move || {
                overlay.set_visible(false);
                overlay.set_opacity(1.0);
            })),
        );
    }

    /// Animate a countdown number (scale from large to normal with fade)
    pub fn animate_number(&self, number: i32) {
        // Cancel any existing animation
        if let Some(ref anim) = *self.animation.borrow() {
            anim.skip();
        }

        self.label.set_text(&number.to_string());
        self.label.set_opacity(0.0);

        let label = self.label.clone();

        // Animate opacity and we'll use CSS for the scale effect
        let target = adw::CallbackAnimationTarget::new(move |value| {
            // Opacity: 0 -> 1 in first half, stay at 1
            let opacity = if value < 0.3 { value / 0.3 } else { 1.0 };
            label.set_opacity(opacity);

            // Scale effect via font size (countdown numbers are large)
            // Start at 120% size, end at 100%
            let scale = 1.2 - (value * 0.2);
            let base_size = 200; // Base font size in px
            let font_size = (base_size as f64 * scale) as i32;

            // Apply via CSS class or markup
            label.set_markup(&format!(
                "<span font_size='{}pt' weight='bold'>{}</span>",
                font_size / 1000, // Pango uses 1/1024 points
                label.text()
            ));
        });

        let animation = adw::TimedAnimation::builder()
            .widget(&self.label)
            .value_from(0.0)
            .value_to(1.0)
            .duration(duration::COUNTDOWN)
            .easing(adw::Easing::EaseOutCubic)
            .target(&target)
            .build();

        animation.play();
        *self.animation.borrow_mut() = Some(animation);
    }
}

/// Animate a widget sliding in from a direction
pub fn slide_in_from_right(
    widget: &impl IsA<gtk::Widget>,
    duration_ms: u32,
) -> adw::TimedAnimation {
    widget.set_visible(true);
    widget.set_opacity(0.0);

    let widget_clone = widget.clone().upcast::<gtk::Widget>();
    let start_margin = 50; // Start 50px to the right

    // Get current margin
    let _original_margin = widget_clone.margin_end();

    let target = adw::CallbackAnimationTarget::new(move |value| {
        // Slide: margin goes from +50 to 0
        let offset = ((1.0 - value) * start_margin as f64) as i32;
        widget_clone.set_margin_start(offset);

        // Fade in
        widget_clone.set_opacity(value);
    });

    let animation = adw::TimedAnimation::builder()
        .widget(widget)
        .value_from(0.0)
        .value_to(1.0)
        .duration(duration_ms)
        .easing(adw::Easing::EaseOutCubic)
        .target(&target)
        .build();

    let widget_final = widget.clone().upcast::<gtk::Widget>();
    animation.connect_done(move |_| {
        widget_final.set_margin_start(0);
    });

    animation.play();
    animation
}

/// Animate a widget sliding up from bottom
pub fn slide_in_from_bottom(
    widget: &impl IsA<gtk::Widget>,
    duration_ms: u32,
) -> adw::TimedAnimation {
    widget.set_visible(true);
    widget.set_opacity(0.0);

    let widget_clone = widget.clone().upcast::<gtk::Widget>();
    let start_offset = 30;

    let target = adw::CallbackAnimationTarget::new(move |value| {
        let offset = ((1.0 - value) * start_offset as f64) as i32;
        widget_clone.set_margin_top(offset);
        widget_clone.set_opacity(value);
    });

    let animation = adw::TimedAnimation::builder()
        .widget(widget)
        .value_from(0.0)
        .value_to(1.0)
        .duration(duration_ms)
        .easing(adw::Easing::EaseOutCubic)
        .target(&target)
        .build();

    let widget_final = widget.clone().upcast::<gtk::Widget>();
    animation.connect_done(move |_| {
        widget_final.set_margin_top(0);
    });

    animation.play();
    animation
}

/// Button press animation - quick scale down and back
pub fn button_press(widget: &impl IsA<gtk::Widget>) -> adw::TimedAnimation {
    let widget_clone = widget.clone().upcast::<gtk::Widget>();
    let _original_opacity = widget_clone.opacity();

    let target = adw::CallbackAnimationTarget::new(move |value| {
        // Quick opacity dip: 1.0 -> 0.7 -> 1.0
        let opacity = if value < 0.5 {
            1.0 - (value * 0.6) // 1.0 -> 0.7
        } else {
            0.7 + ((value - 0.5) * 0.6) // 0.7 -> 1.0
        };
        widget_clone.set_opacity(opacity);
    });

    let animation = adw::TimedAnimation::builder()
        .widget(widget)
        .value_from(0.0)
        .value_to(1.0)
        .duration(duration::FAST)
        .easing(adw::Easing::EaseOutCubic)
        .target(&target)
        .build();

    animation.play();
    animation
}

/// Crossfade between two widgets (fade out first, fade in second)
pub fn crossfade(
    fade_out_widget: &impl IsA<gtk::Widget>,
    fade_in_widget: &impl IsA<gtk::Widget>,
    duration_ms: u32,
) {
    let fade_in_widget = fade_in_widget.clone().upcast::<gtk::Widget>();
    let fade_out_clone = fade_out_widget.clone().upcast::<gtk::Widget>();

    // Start fade out
    fade(
        fade_out_widget,
        1.0,
        0.0,
        duration_ms / 2,
        Some(Box::new(move || {
            fade_out_clone.set_visible(false);
            fade_out_clone.set_opacity(1.0);

            // Start fade in
            fade_in_widget.set_opacity(0.0);
            fade_in_widget.set_visible(true);
            fade(&fade_in_widget, 0.0, 1.0, duration_ms / 2, None);
        })),
    );
}

/// Pulse animation for attention (opacity throb)
pub fn pulse(widget: &impl IsA<gtk::Widget>, duration_ms: u32) -> adw::TimedAnimation {
    let widget_clone = widget.clone().upcast::<gtk::Widget>();

    let target = adw::CallbackAnimationTarget::new(move |value| {
        // Sine wave for smooth pulse: 1.0 -> 0.6 -> 1.0
        let opacity = 0.8 + 0.2 * (value * std::f64::consts::PI * 2.0).cos();
        widget_clone.set_opacity(opacity);
    });

    let animation = adw::TimedAnimation::builder()
        .widget(widget)
        .value_from(0.0)
        .value_to(1.0)
        .duration(duration_ms)
        .easing(adw::Easing::Linear)
        .target(&target)
        .build();

    animation.play();
    animation
}
