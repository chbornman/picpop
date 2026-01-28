//! GStreamer pipeline for MJPEG camera preview.

use gstreamer as gst;
use gstreamer::prelude::*;
use gtk4 as gtk;
use thiserror::Error;

use crate::config;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("GStreamer error: {0}")]
    Gstreamer(#[from] glib::Error),
    #[error("GStreamer bool error: {0}")]
    GstreamerBool(#[from] glib::BoolError),
    #[error("Failed to create element: {0}")]
    ElementCreation(String),
    #[error("Failed to get paintable sink")]
    NoPaintable,
    #[error("State change failed")]
    StateChange,
}

/// Video pipeline for camera preview
pub struct VideoPipeline {
    pipeline: gst::Pipeline,
    paintable: gtk::gdk::Paintable,
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

        // GTK4 paintable sink
        let sink = gst::ElementFactory::make("gtk4paintablesink")
            .build()
            .map_err(|_| PipelineError::ElementCreation("gtk4paintablesink".into()))?;

        // Get the paintable from the sink
        let paintable = sink
            .property::<gtk::gdk::Paintable>("paintable");

        // Add elements to pipeline
        pipeline.add_many([&source, &demux, &decoder, &convert, &sink])?;

        // Link source to demux
        source.link(&demux)?;

        // Link decoder to convert to sink
        decoder.link(&convert)?;
        convert.link(&sink)?;

        // Connect demux pad-added signal to link to decoder
        let decoder_weak = decoder.downgrade();
        demux.connect_pad_added(move |_demux, src_pad| {
            log::debug!("Demux pad added: {}", src_pad.name());

            if let Some(decoder) = decoder_weak.upgrade() {
                if let Some(sink_pad) = decoder.static_pad("sink") {
                    if !sink_pad.is_linked() {
                        if let Err(e) = src_pad.link(&sink_pad) {
                            log::error!("Failed to link demux to decoder: {:?}", e);
                        } else {
                            log::info!("Linked demux to decoder");
                        }
                    }
                }
            }
        });

        Ok(Self { pipeline, paintable })
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

    /// Pause the pipeline
    pub fn pause(&self) -> Result<(), PipelineError> {
        log::info!("Pausing video pipeline");
        self.pipeline
            .set_state(gst::State::Paused)
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

    /// Restart the pipeline (useful for reconnecting)
    pub fn restart(&self) -> Result<(), PipelineError> {
        self.stop()?;
        self.play()?;
        Ok(())
    }

    /// Set up bus message handling
    pub fn setup_bus_watch<F>(&self, callback: F)
    where
        F: Fn(&gst::Bus, &gst::Message) -> glib::ControlFlow + Send + Sync + 'static,
    {
        if let Some(bus) = self.pipeline.bus() {
            let _ = bus.add_watch(move |bus, msg| callback(bus, msg));
        }
    }
}

impl Drop for VideoPipeline {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
