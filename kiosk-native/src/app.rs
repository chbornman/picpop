//! Application context - bridges the GTK-free state machine with GTK UI.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4 as gtk;
use tokio::sync::mpsc;

use crate::api::{ApiClient, WsEvent, WsHandle};
use crate::state::{KioskCommand, KioskEvent, KioskStateMachine};
use crate::video::VideoPipeline;



/// Messages sent from async tasks to the GTK main loop
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Process a kiosk event through the state machine
    Event(KioskEvent),
}

/// Sender that can dispatch messages to the GTK main loop from any thread
#[derive(Clone)]
pub struct MessageSender {
    /// We use a tokio channel + glib::idle_add for thread-safe dispatch
    tx: mpsc::UnboundedSender<AppMessage>,
}

impl MessageSender {
    pub fn send(&self, msg: AppMessage) {
        let _ = self.tx.send(msg);
    }
}

/// Application context - holds state and provides methods to interact with it
pub struct AppContext {
    /// The GTK-free state machine
    pub state_machine: RefCell<KioskStateMachine>,
    /// HTTP API client
    pub api: ApiClient,
    /// GStreamer video pipeline
    pub video: RefCell<Option<VideoPipeline>>,
    /// Tokio runtime for async operations
    pub runtime: Arc<tokio::runtime::Runtime>,
    /// Sender for dispatching messages to GTK main loop
    pub message_tx: MessageSender,
    /// WebSocket handle (for cleanup)
    ws_handle: RefCell<Option<WsHandle>>,
}

impl AppContext {
    pub fn new(runtime: Arc<tokio::runtime::Runtime>) -> (Rc<Self>, mpsc::UnboundedReceiver<AppMessage>) {
        let (tx, rx) = mpsc::unbounded_channel();

        let ctx = Rc::new(Self {
            state_machine: RefCell::new(KioskStateMachine::new()),
            api: ApiClient::new(),
            video: RefCell::new(None),
            runtime,
            message_tx: MessageSender { tx },
            ws_handle: RefCell::new(None),
        });

        (ctx, rx)
    }

    /// Initialize the video pipeline
    pub fn init_video(&self) -> Result<gtk::gdk::Paintable, crate::video::pipeline::PipelineError> {
        let pipeline = VideoPipeline::new()?;
        let paintable = pipeline.paintable().clone();

        // Set up error handling with automatic reconnection
        pipeline.setup_bus_watch_with_reconnect();

        pipeline.play()?;
        *self.video.borrow_mut() = Some(pipeline);

        Ok(paintable)
    }

    /// Send an event to the state machine (from any thread)
    pub fn send_event(&self, event: KioskEvent) {
        self.message_tx.send(AppMessage::Event(event));
    }

    /// Process an event and execute resulting commands
    /// This should be called from the GTK main loop
    pub fn process_event(self: &Rc<Self>, event: KioskEvent) -> Vec<KioskCommand> {
        let commands = self.state_machine.borrow_mut().process(event);

        // Execute commands
        for cmd in &commands {
            self.execute_command(cmd.clone());
        }

        commands
    }

    /// Execute a command from the state machine
    fn execute_command(self: &Rc<Self>, cmd: KioskCommand) {
        match cmd {
            KioskCommand::CreateSession => {
                let tx = self.message_tx.clone();
                let api = self.api.clone();

                self.runtime.spawn(async move {
                    match api.create_session().await {
                        Ok(response) => {
                            tx.send(AppMessage::Event(KioskEvent::SessionCreated {
                                id: response.id,
                            }));
                        }
                        Err(e) => {
                            tx.send(AppMessage::Event(KioskEvent::SessionCreateFailed {
                                error: e.to_string(),
                            }));
                        }
                    }
                });
            }

            KioskCommand::EndSession { session_id } => {
                let tx = self.message_tx.clone();
                let api = self.api.clone();

                self.runtime.spawn(async move {
                    match api.end_session(&session_id).await {
                        Ok(_) => {
                            tx.send(AppMessage::Event(KioskEvent::SessionEnded));
                        }
                        Err(e) => {
                            log::error!("Failed to end session: {}", e);
                            // Still transition to welcome even on error
                            tx.send(AppMessage::Event(KioskEvent::SessionEnded));
                        }
                    }
                });
            }

            KioskCommand::TriggerCapture { session_id } => {
                let tx = self.message_tx.clone();
                let api = self.api.clone();

                self.runtime.spawn(async move {
                    if let Err(e) = api.capture(&session_id).await {
                        tx.send(AppMessage::Event(KioskEvent::CaptureFailed {
                            error: e.to_string(),
                        }));
                    }
                    // Success is handled via WebSocket countdown events
                });
            }

            KioskCommand::ConnectWebSocket { session_id } => {
                let tx = self.message_tx.clone();
                let runtime = self.runtime.clone();

                // Connect to WebSocket with a callback that dispatches events
                let handle = crate::api::websocket::connect(runtime, session_id, move |ws_event| {
                    let event = match ws_event {
                        WsEvent::Connected => KioskEvent::WebSocketConnected,
                        WsEvent::Disconnected => KioskEvent::WebSocketDisconnected,
                        WsEvent::PhoneConnected => KioskEvent::PhoneConnected,
                        WsEvent::PhoneDisconnected => KioskEvent::PhoneDisconnected,
                        WsEvent::Countdown(value) => KioskEvent::CountdownTick { value },
                        WsEvent::PhotoReady(photo) => KioskEvent::PhotoReady { photo },
                        WsEvent::Processing => KioskEvent::Processing,
                        WsEvent::CaptureComplete => KioskEvent::CaptureComplete,
                        WsEvent::CaptureFailed(error) => KioskEvent::CaptureFailed { error },
                        WsEvent::SessionEnded => KioskEvent::SessionEnded,
                    };
                    tx.send(AppMessage::Event(event));
                });

                *self.ws_handle.borrow_mut() = Some(handle);
            }

            KioskCommand::DisconnectWebSocket => {
                if let Some(handle) = self.ws_handle.borrow_mut().take() {
                    let rt = self.runtime.clone();
                    rt.spawn(async move {
                        handle.close().await;
                    });
                }
            }

            KioskCommand::ScheduleErrorClear => {
                let tx = self.message_tx.clone();
                glib::timeout_add_once(
                    std::time::Duration::from_millis(crate::config::ERROR_DISPLAY_DURATION_MS),
                    move || {
                        tx.send(AppMessage::Event(KioskEvent::ClearError));
                    },
                );
            }

            KioskCommand::UpdateUI => {
                // This is handled by the window after processing events
            }
        }
    }

}
