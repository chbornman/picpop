//! GStreamer pipeline for MJPEG camera preview with auto-reconnect.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use gstreamer as gst;
use gstreamer::prelude::*;
use gtk4 as gtk;
use thiserror::Error;

use crate::config;

/// Delay before attempting to reconnect after an error (in milliseconds)
const RECONNECT_DELAY_MS: u64 = 2000;

/// How often to check for stale frames (in milliseconds)  
const STALE_CHECK_INTERVAL_MS: u64 = 3000;

/// If no new frame for this long, consider stream stale (in milliseconds)
const STALE_THRESHOLD_MS: u64 = 5000;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("GStreamer error: {0}")]
    Gstreamer(#[from] glib::Error),
    #[error("GStreamer bool error: {0}")]
    GstreamerBool(#[from] glib::BoolError),
    #[error("Failed to create element: {0}")]
    ElementCreation(String),
    #[error("State change failed")]
    StateChange,
}

/// Video pipeline for camera preview
pub struct VideoPipeline {
    pipeline: gst::Pipeline,
    paintable: gtk::gdk::Paintable,
    is_reconnecting: Arc<AtomicBool>,
    /// Timestamp of last received frame (unix millis)
    last_frame_time: Arc<AtomicU64>,
    /// Total frames received
    frame_count: Arc<AtomicU64>,
}

impl VideoPipeline {
    /// Create a new video pipeline for MJPEG preview
    pub fn new() -> Result<Self, PipelineError> {
        gst::init()?;

        // Build the pipeline
        let pipeline = gst::Pipeline::new();

        // Source: HTTP stream
        let source = gst::ElementFactory::make("souphttpsrc")
            .property("location", config::CAMERA_PREVIEW_URL)
            .property("is-live", true)
            .property("do-timestamp", true)
            .build()
            .map_err(|_| PipelineError::ElementCreation("souphttpsrc".into()))?;

        // Demux: multipart/x-mixed-replace
        let demux = gst::ElementFactory::make("multipartdemux")
            .build()
            .map_err(|_| PipelineError::ElementCreation("multipartdemux".into()))?;

        // Decoder: JPEG
        let decoder = gst::ElementFactory::make("jpegdec")
            .build()
            .map_err(|_| PipelineError::ElementCreation("jpegdec".into()))?;

        // Video convert for format compatibility
        let convert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| PipelineError::ElementCreation("videoconvert".into()))?;

        // Queue to decouple the pipeline and prevent buffer drops
        let queue = gst::ElementFactory::make("queue")
            .property("max-size-buffers", 3u32)
            .property("max-size-time", 0u64)
            .property("max-size-bytes", 0u32)
            .build()
            .map_err(|_| PipelineError::ElementCreation("queue".into()))?;

        // GTK4 paintable sink
        let sink = gst::ElementFactory::make("gtk4paintablesink")
            .build()
            .map_err(|_| PipelineError::ElementCreation("gtk4paintablesink".into()))?;

        // Get the paintable from the sink
        let paintable = sink.property::<gtk::gdk::Paintable>("paintable");

        // Add elements to pipeline
        pipeline.add_many([&source, &demux, &decoder, &convert, &queue, &sink])?;

        // Link source to demux
        source.link(&demux)?;

        // Link decoder to convert to queue to sink
        decoder.link(&convert)?;
        convert.link(&queue)?;
        queue.link(&sink)?;

        // Track frame timing
        let last_frame_time = Arc::new(AtomicU64::new(0));
        let frame_count = Arc::new(AtomicU64::new(0));

        // Connect demux pad-added signal to link to decoder
        let decoder_weak = decoder.downgrade();
        let last_frame_demux = last_frame_time.clone();
        let frame_count_demux = frame_count.clone();
        demux.connect_pad_added(move |_demux, src_pad| {
            log::info!(
                "[PIPELINE] Demux pad added: {} - stream connected",
                src_pad.name()
            );

            // Reset frame tracking on new connection
            last_frame_demux.store(now_millis(), Ordering::SeqCst);
            frame_count_demux.store(0, Ordering::SeqCst);

            if let Some(decoder) = decoder_weak.upgrade() {
                if let Some(sink_pad) = decoder.static_pad("sink") {
                    if !sink_pad.is_linked() {
                        if let Err(e) = src_pad.link(&sink_pad) {
                            log::error!("[PIPELINE] Failed to link demux to decoder: {:?}", e);
                        } else {
                            log::info!("[PIPELINE] Linked demux to decoder successfully");
                        }
                    }
                }
            }
        });

        // Add probe on decoder src pad to track frames
        let last_frame_probe = last_frame_time.clone();
        let frame_count_probe = frame_count.clone();
        if let Some(src_pad) = decoder.static_pad("src") {
            src_pad.add_probe(gst::PadProbeType::BUFFER, move |_, _| {
                let count = frame_count_probe.fetch_add(1, Ordering::SeqCst) + 1;
                last_frame_probe.store(now_millis(), Ordering::SeqCst);

                // Log periodically
                if count == 1 {
                    log::info!("[PIPELINE] First frame decoded!");
                } else if count % 300 == 0 {
                    log::debug!("[PIPELINE] Frames decoded: {}", count);
                }

                gst::PadProbeReturn::Ok
            });
        }

        Ok(Self {
            pipeline,
            paintable,
            is_reconnecting: Arc::new(AtomicBool::new(false)),
            last_frame_time,
            frame_count,
        })
    }

    /// Get the paintable for use in GTK widgets
    pub fn paintable(&self) -> &gtk::gdk::Paintable {
        &self.paintable
    }

    /// Start the pipeline
    pub fn play(&self) -> Result<(), PipelineError> {
        log::info!("Starting video pipeline");
        self.pipeline
            .set_state(gst::State::Playing)
            .map_err(|_| PipelineError::StateChange)?;
        Ok(())
    }

    /// Stop the pipeline
    pub fn stop(&self) -> Result<(), PipelineError> {
        log::info!("Stopping video pipeline");
        self.pipeline
            .set_state(gst::State::Null)
            .map_err(|_| PipelineError::StateChange)?;
        Ok(())
    }

    /// Set up bus message handling with automatic reconnection on errors
    pub fn setup_bus_watch_with_reconnect(&self) {
        let pipeline_weak = self.pipeline.downgrade();
        let is_reconnecting = self.is_reconnecting.clone();
        let last_frame_time = self.last_frame_time.clone();
        let frame_count = self.frame_count.clone();

        if let Some(bus) = self.pipeline.bus() {
            let is_reconnecting_bus = is_reconnecting.clone();
            let _ = bus.add_watch_local(move |_bus, msg| {
                use gstreamer::MessageView;

                match msg.view() {
                    MessageView::Error(err) => {
                        let src_name = msg
                            .src()
                            .map(|s| s.name().to_string())
                            .unwrap_or_else(|| "unknown".into());
                        log::error!(
                            "[PIPELINE] Error from {}: {} (debug: {:?})",
                            src_name,
                            err.error(),
                            err.debug()
                        );
                        schedule_reconnect(&pipeline_weak, &is_reconnecting_bus);
                    }
                    MessageView::Eos(_) => {
                        log::warn!(
                            "[PIPELINE] End of stream - camera disconnected or stream ended"
                        );
                        schedule_reconnect(&pipeline_weak, &is_reconnecting_bus);
                    }
                    MessageView::Warning(warn) => {
                        let src_name = msg
                            .src()
                            .map(|s| s.name().to_string())
                            .unwrap_or_else(|| "unknown".into());
                        log::warn!(
                            "[PIPELINE] Warning from {}: {} (debug: {:?})",
                            src_name,
                            warn.error(),
                            warn.debug()
                        );
                    }
                    MessageView::StateChanged(state) => {
                        if let Some(src) = msg.src() {
                            if src.type_() == gst::Pipeline::static_type() {
                                log::info!(
                                    "[PIPELINE] State: {:?} -> {:?} (pending: {:?})",
                                    state.old(),
                                    state.current(),
                                    state.pending()
                                );
                            }
                        }
                    }
                    MessageView::Buffering(buffering) => {
                        log::debug!("[PIPELINE] Buffering: {}%", buffering.percent());
                    }
                    MessageView::Latency(_) => {
                        log::debug!("[PIPELINE] Latency update");
                    }
                    _ => {}
                }
                glib::ControlFlow::Continue
            });
        }

        // Set up stale frame detection
        self.setup_stale_frame_detection(is_reconnecting, last_frame_time, frame_count);
    }

    /// Periodically check if frames are still coming in
    fn setup_stale_frame_detection(
        &self,
        is_reconnecting: Arc<AtomicBool>,
        last_frame_time: Arc<AtomicU64>,
        frame_count: Arc<AtomicU64>,
    ) {
        let pipeline_weak = self.pipeline.downgrade();

        glib::timeout_add_local(Duration::from_millis(STALE_CHECK_INTERVAL_MS), move || {
            // Don't check if we're already reconnecting
            if is_reconnecting.load(Ordering::SeqCst) {
                return glib::ControlFlow::Continue;
            }

            let last_frame = last_frame_time.load(Ordering::SeqCst);
            let frames = frame_count.load(Ordering::SeqCst);
            let now = now_millis();

            // Only check if we've received at least one frame
            if last_frame > 0 {
                let elapsed = now.saturating_sub(last_frame);

                if elapsed > STALE_THRESHOLD_MS {
                    log::warn!(
                        "[PIPELINE] Stream appears stale! No frames for {}ms (total frames: {})",
                        elapsed,
                        frames
                    );

                    // Trigger reconnect
                    if let Some(pipeline) = pipeline_weak.upgrade() {
                        log::info!("[PIPELINE] Forcing reconnect due to stale stream");
                        schedule_reconnect_pipeline(pipeline, is_reconnecting.clone());
                    }
                }
            } else if frames == 0 {
                // Never received any frames - check pipeline state
                if let Some(pipeline) = pipeline_weak.upgrade() {
                    let (_, current, _) = pipeline.state(gst::ClockTime::from_mseconds(10));
                    if current == gst::State::Playing {
                        log::warn!("[PIPELINE] Playing but no frames received yet");
                    }
                }
            }

            glib::ControlFlow::Continue
        });
    }
}

/// Get current time in milliseconds
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Schedule a reconnection attempt by restarting the pipeline
fn schedule_reconnect(
    pipeline_weak: &glib::WeakRef<gst::Pipeline>,
    is_reconnecting: &Arc<AtomicBool>,
) {
    if let Some(pipeline) = pipeline_weak.upgrade() {
        schedule_reconnect_pipeline(pipeline, is_reconnecting.clone());
    }
}

/// Schedule reconnection for a pipeline (used by both error handler and stale detection)
fn schedule_reconnect_pipeline(pipeline: gst::Pipeline, is_reconnecting: Arc<AtomicBool>) {
    if !is_reconnecting.swap(true, Ordering::SeqCst) {
        let (_, current_state, _) = pipeline.state(gst::ClockTime::from_mseconds(10));

        log::info!(
            "[PIPELINE] Initiating reconnect (current state: {:?}), will retry in {}ms",
            current_state,
            RECONNECT_DELAY_MS
        );

        // Stop pipeline
        log::info!("[PIPELINE] Setting state to NULL");
        match pipeline.set_state(gst::State::Null) {
            Ok(_) => log::info!("[PIPELINE] State set to NULL successfully"),
            Err(e) => log::error!("[PIPELINE] Failed to set NULL state: {:?}", e),
        }

        // Schedule restart
        schedule_restart(pipeline, is_reconnecting, 0);
    } else {
        log::debug!("[PIPELINE] Reconnect already in progress, skipping");
    }
}

/// Maximum number of restart attempts before giving up
const MAX_RESTART_ATTEMPTS: u32 = 10;

/// Try to restart the pipeline, retrying if it fails
fn schedule_restart(pipeline: gst::Pipeline, is_reconnecting: Arc<AtomicBool>, attempt: u32) {
    glib::timeout_add_local_once(Duration::from_millis(RECONNECT_DELAY_MS), move || {
        log::info!(
            "[PIPELINE] Reconnection attempt {} of {}",
            attempt + 1,
            MAX_RESTART_ATTEMPTS
        );

        // First set to NULL to fully reset
        log::debug!("[PIPELINE] Ensuring NULL state before restart");
        let _ = pipeline.set_state(gst::State::Null);

        // Small delay then try to play
        let pipeline_clone = pipeline.clone();
        let is_reconnecting_clone = is_reconnecting.clone();

        glib::timeout_add_local_once(Duration::from_millis(500), move || {
            log::info!("[PIPELINE] Attempting to set PLAYING state");

            match pipeline_clone.set_state(gst::State::Playing) {
                Ok(gst::StateChangeSuccess::Success) => {
                    log::info!("[PIPELINE] State change to PLAYING succeeded immediately");
                    verify_reconnection(pipeline_clone, is_reconnecting_clone, attempt);
                }
                Ok(gst::StateChangeSuccess::Async) => {
                    log::info!("[PIPELINE] State change to PLAYING is async, waiting...");
                    verify_reconnection(pipeline_clone, is_reconnecting_clone, attempt);
                }
                Ok(gst::StateChangeSuccess::NoPreroll) => {
                    log::info!("[PIPELINE] State change succeeded (no preroll - live source)");
                    verify_reconnection(pipeline_clone, is_reconnecting_clone, attempt);
                }
                Err(e) => {
                    log::error!("[PIPELINE] Failed to set PLAYING state: {:?}", e);
                    retry_or_give_up(pipeline_clone, is_reconnecting_clone, attempt);
                }
            }
        });
    });
}

/// Verify that reconnection actually worked by checking state after a delay
fn verify_reconnection(pipeline: gst::Pipeline, is_reconnecting: Arc<AtomicBool>, attempt: u32) {
    glib::timeout_add_local_once(Duration::from_millis(2000), move || {
        let (result, current, pending) = pipeline.state(gst::ClockTime::from_mseconds(100));

        log::info!(
            "[PIPELINE] Verification: state={:?}, pending={:?}, result={:?}",
            current,
            pending,
            result
        );

        if current == gst::State::Playing {
            log::info!("[PIPELINE] Reconnection SUCCESSFUL - stream should resume");
            is_reconnecting.store(false, Ordering::SeqCst);
        } else {
            log::warn!(
                "[PIPELINE] Not playing after restart (state: {:?}), will retry",
                current
            );
            retry_or_give_up(pipeline, is_reconnecting, attempt);
        }
    });
}

/// Either retry or give up based on attempt count
fn retry_or_give_up(pipeline: gst::Pipeline, is_reconnecting: Arc<AtomicBool>, attempt: u32) {
    if attempt < MAX_RESTART_ATTEMPTS {
        log::info!("[PIPELINE] Scheduling retry attempt {}", attempt + 2);
        schedule_restart(pipeline, is_reconnecting, attempt + 1);
    } else {
        log::error!(
            "[PIPELINE] GIVING UP after {} attempts - camera preview unavailable!",
            MAX_RESTART_ATTEMPTS
        );
        is_reconnecting.store(false, Ordering::SeqCst);
    }
}

impl Drop for VideoPipeline {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
