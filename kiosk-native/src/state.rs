//! GTK-free state machine and business logic.
//!
//! This module contains the pure Rust state machine that can be tested
//! independently of GTK. The UI layer observes state changes and updates
//! accordingly.

use crate::api::PhotoInfo;

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KioskState {
    /// Welcome screen - waiting for user to start a session
    Welcome,
    /// Active session - user can capture photos and view them
    Session,
    /// Countdown in progress before capture
    Countdown,
    /// Processing photos after capture
    Processing,
}

/// Session data (GTK-free)
#[derive(Debug, Clone, Default)]
pub struct SessionData {
    pub id: String,
    pub phone_count: u32,
    pub photos: Vec<PhotoInfo>,
}

/// Events that trigger state transitions
#[derive(Debug, Clone)]
pub enum KioskEvent {
    // User actions
    StartSession,
    EndSession,
    TriggerCapture,

    // Photo viewing (in-place, no separate lightbox)
    SelectPhoto(usize), // View a photo from strip
    SelectLive,         // Return to live video view

    // Backend responses
    SessionCreated { id: String },
    SessionCreateFailed { error: String },
    SessionEnded,

    // WebSocket events
    PhoneConnected,
    PhoneDisconnected,
    CountdownTick { value: u32 },
    PhotoReady { photo: PhotoInfo },
    Processing,
    CaptureComplete,
    CaptureFailed { error: String },
    WebSocketConnected,
    WebSocketDisconnected,

    // Internal
    ClearError,
}

/// Commands emitted by the state machine for the UI/API layer to execute
#[derive(Debug, Clone)]
pub enum KioskCommand {
    /// Call the create session API
    CreateSession,
    /// Call the end session API
    EndSession { session_id: String },
    /// Call the capture API
    TriggerCapture { session_id: String },
    /// Connect to WebSocket for session
    ConnectWebSocket { session_id: String },
    /// Disconnect WebSocket
    DisconnectWebSocket,
    /// Schedule error clear after timeout
    ScheduleErrorClear,
    /// Update UI to reflect new state
    UpdateUI,
}

/// The kiosk state machine
#[derive(Debug)]
pub struct KioskStateMachine {
    pub state: KioskState,
    pub session: Option<SessionData>,
    pub countdown_value: Option<u32>,
    /// Which photo is being viewed (None = live video view)
    pub viewing_photo: Option<usize>,
    pub error: Option<String>,
    pub is_loading: bool,
}

impl Default for KioskStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl KioskStateMachine {
    pub fn new() -> Self {
        Self {
            state: KioskState::Welcome,
            session: None,
            countdown_value: None,
            viewing_photo: None,
            error: None,
            is_loading: false,
        }
    }

    /// Check if currently viewing live video (not a photo)
    pub fn is_live_view(&self) -> bool {
        self.viewing_photo.is_none()
    }

    /// Process an event and return commands to execute
    pub fn process(&mut self, event: KioskEvent) -> Vec<KioskCommand> {
        let mut commands = Vec::new();

        match event {
            KioskEvent::StartSession => {
                if self.state == KioskState::Welcome && !self.is_loading {
                    self.is_loading = true;
                    self.error = None;
                    commands.push(KioskCommand::CreateSession);
                    commands.push(KioskCommand::UpdateUI);
                }
            }

            KioskEvent::SessionCreated { id } => {
                self.state = KioskState::Session;
                self.is_loading = false;
                self.viewing_photo = None;
                self.session = Some(SessionData {
                    id: id.clone(),
                    phone_count: 0,
                    photos: Vec::new(),
                });
                commands.push(KioskCommand::ConnectWebSocket { session_id: id });
                commands.push(KioskCommand::UpdateUI);
            }

            KioskEvent::SessionCreateFailed { error } => {
                self.is_loading = false;
                self.error = Some(error);
                commands.push(KioskCommand::ScheduleErrorClear);
                commands.push(KioskCommand::UpdateUI);
            }

            KioskEvent::EndSession => {
                if let Some(ref session) = self.session {
                    let session_id = session.id.clone();
                    commands.push(KioskCommand::DisconnectWebSocket);
                    commands.push(KioskCommand::EndSession { session_id });
                }
            }

            KioskEvent::SessionEnded => {
                self.state = KioskState::Welcome;
                self.session = None;
                self.countdown_value = None;
                self.viewing_photo = None;
                commands.push(KioskCommand::UpdateUI);
            }

            KioskEvent::TriggerCapture => {
                // Only allow capture from live view in session state
                if self.state == KioskState::Session && self.is_live_view() {
                    if let Some(ref session) = self.session {
                        commands.push(KioskCommand::TriggerCapture {
                            session_id: session.id.clone(),
                        });
                    }
                }
            }

            KioskEvent::PhoneConnected => {
                if let Some(ref mut session) = self.session {
                    session.phone_count += 1;
                    commands.push(KioskCommand::UpdateUI);
                }
            }

            KioskEvent::PhoneDisconnected => {
                if let Some(ref mut session) = self.session {
                    session.phone_count = session.phone_count.saturating_sub(1);
                    commands.push(KioskCommand::UpdateUI);
                }
            }

            KioskEvent::CountdownTick { value } => {
                self.state = KioskState::Countdown;
                self.countdown_value = Some(value);
                self.viewing_photo = None; // Ensure we're on live view during countdown
                commands.push(KioskCommand::UpdateUI);
            }

            KioskEvent::PhotoReady { photo } => {
                if let Some(ref mut session) = self.session {
                    session.photos.push(photo);
                    commands.push(KioskCommand::UpdateUI);
                }
            }

            KioskEvent::Processing => {
                self.state = KioskState::Processing;
                self.countdown_value = None;
                commands.push(KioskCommand::UpdateUI);
            }

            KioskEvent::CaptureComplete => {
                self.state = KioskState::Session;
                self.countdown_value = None;
                // Stay on live view after capture
                commands.push(KioskCommand::UpdateUI);
            }

            KioskEvent::CaptureFailed { error } => {
                self.state = KioskState::Session;
                self.countdown_value = None;
                self.error = Some(error);
                commands.push(KioskCommand::ScheduleErrorClear);
                commands.push(KioskCommand::UpdateUI);
            }

            KioskEvent::SelectPhoto(index) => {
                if self.state == KioskState::Session {
                    if let Some(ref session) = self.session {
                        if index < session.photos.len() {
                            self.viewing_photo = Some(index);
                            commands.push(KioskCommand::UpdateUI);
                        }
                    }
                }
            }

            KioskEvent::SelectLive => {
                if self.state == KioskState::Session && self.viewing_photo.is_some() {
                    self.viewing_photo = None;
                    commands.push(KioskCommand::UpdateUI);
                }
            }

            KioskEvent::ClearError => {
                self.error = None;
                commands.push(KioskCommand::UpdateUI);
            }

            KioskEvent::WebSocketConnected | KioskEvent::WebSocketDisconnected => {
                // These are informational, no state change needed
            }
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = KioskStateMachine::new();
        assert_eq!(sm.state, KioskState::Welcome);
        assert!(sm.session.is_none());
        assert!(sm.is_live_view());
    }

    #[test]
    fn test_start_session_flow() {
        let mut sm = KioskStateMachine::new();

        // Start session
        let cmds = sm.process(KioskEvent::StartSession);
        assert!(sm.is_loading);
        assert!(cmds
            .iter()
            .any(|c| matches!(c, KioskCommand::CreateSession)));

        // Session created
        let cmds = sm.process(KioskEvent::SessionCreated {
            id: "test-123".into(),
        });
        assert_eq!(sm.state, KioskState::Session);
        assert!(!sm.is_loading);
        assert!(sm.session.is_some());
        assert!(sm.is_live_view());
        assert!(cmds
            .iter()
            .any(|c| matches!(c, KioskCommand::ConnectWebSocket { .. })));
    }

    #[test]
    fn test_capture_flow() {
        let mut sm = KioskStateMachine::new();
        sm.process(KioskEvent::StartSession);
        sm.process(KioskEvent::SessionCreated {
            id: "test-123".into(),
        });

        // Trigger capture
        let cmds = sm.process(KioskEvent::TriggerCapture);
        assert!(cmds
            .iter()
            .any(|c| matches!(c, KioskCommand::TriggerCapture { .. })));

        // Countdown
        sm.process(KioskEvent::CountdownTick { value: 3 });
        assert_eq!(sm.state, KioskState::Countdown);
        assert_eq!(sm.countdown_value, Some(3));

        // Photo ready
        sm.process(KioskEvent::PhotoReady {
            photo: PhotoInfo {
                id: "photo-1".into(),
                thumbnail_url: "/thumb.jpg".into(),
                web_url: "/photo.jpg".into(),
            },
        });
        assert_eq!(sm.session.as_ref().unwrap().photos.len(), 1);

        // Capture complete
        sm.process(KioskEvent::CaptureComplete);
        assert_eq!(sm.state, KioskState::Session);
        assert!(sm.countdown_value.is_none());
        assert!(sm.is_live_view());
    }

    #[test]
    fn test_phone_count() {
        let mut sm = KioskStateMachine::new();
        sm.process(KioskEvent::StartSession);
        sm.process(KioskEvent::SessionCreated {
            id: "test-123".into(),
        });

        sm.process(KioskEvent::PhoneConnected);
        assert_eq!(sm.session.as_ref().unwrap().phone_count, 1);

        sm.process(KioskEvent::PhoneConnected);
        assert_eq!(sm.session.as_ref().unwrap().phone_count, 2);

        sm.process(KioskEvent::PhoneDisconnected);
        assert_eq!(sm.session.as_ref().unwrap().phone_count, 1);
    }

    #[test]
    fn test_photo_viewing() {
        let mut sm = KioskStateMachine::new();
        sm.process(KioskEvent::StartSession);
        sm.process(KioskEvent::SessionCreated {
            id: "test-123".into(),
        });

        // Add some photos
        sm.process(KioskEvent::PhotoReady {
            photo: PhotoInfo {
                id: "photo-1".into(),
                thumbnail_url: "/thumb1.jpg".into(),
                web_url: "/photo1.jpg".into(),
            },
        });
        sm.process(KioskEvent::PhotoReady {
            photo: PhotoInfo {
                id: "photo-2".into(),
                thumbnail_url: "/thumb2.jpg".into(),
                web_url: "/photo2.jpg".into(),
            },
        });

        // Should start in live view
        assert!(sm.is_live_view());

        // Select a photo
        sm.process(KioskEvent::SelectPhoto(0));
        assert_eq!(sm.viewing_photo, Some(0));
        assert!(!sm.is_live_view());

        // Return to live view
        sm.process(KioskEvent::SelectLive);
        assert!(sm.is_live_view());
    }

    #[test]
    fn test_capture_only_from_live_view() {
        let mut sm = KioskStateMachine::new();
        sm.process(KioskEvent::StartSession);
        sm.process(KioskEvent::SessionCreated {
            id: "test-123".into(),
        });
        sm.process(KioskEvent::PhotoReady {
            photo: PhotoInfo {
                id: "photo-1".into(),
                thumbnail_url: "/thumb.jpg".into(),
                web_url: "/photo.jpg".into(),
            },
        });

        // Select photo (not live view)
        sm.process(KioskEvent::SelectPhoto(0));
        assert!(!sm.is_live_view());

        // Try to capture - should not emit command
        let cmds = sm.process(KioskEvent::TriggerCapture);
        assert!(!cmds
            .iter()
            .any(|c| matches!(c, KioskCommand::TriggerCapture { .. })));

        // Return to live and capture - should work
        sm.process(KioskEvent::SelectLive);
        let cmds = sm.process(KioskEvent::TriggerCapture);
        assert!(cmds
            .iter()
            .any(|c| matches!(c, KioskCommand::TriggerCapture { .. })));
    }

    #[test]
    fn test_end_session() {
        let mut sm = KioskStateMachine::new();
        sm.process(KioskEvent::StartSession);
        sm.process(KioskEvent::SessionCreated {
            id: "test-123".into(),
        });

        let cmds = sm.process(KioskEvent::EndSession);
        assert!(cmds
            .iter()
            .any(|c| matches!(c, KioskCommand::EndSession { .. })));

        sm.process(KioskEvent::SessionEnded);
        assert_eq!(sm.state, KioskState::Welcome);
        assert!(sm.session.is_none());
        assert!(sm.is_live_view());
    }
}
