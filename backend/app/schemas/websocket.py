"""WebSocket message schemas."""

from enum import Enum
from typing import Any

from pydantic import BaseModel


class WSMessageType(str, Enum):
    """WebSocket message types."""

    # Server -> Client messages
    SESSION_READY = "session_ready"
    PHONE_CONNECTED = "phone_connected"
    PHONE_DISCONNECTED = "phone_disconnected"
    KIOSK_CONNECTED = "kiosk_connected"
    COUNTDOWN = "countdown"
    CAPTURE_START = "capture_start"
    PHOTO_READY = "photo_ready"
    CAPTURE_COMPLETE = "capture_complete"
    SESSION_ENDED = "session_ended"
    ERROR = "error"

    # Client -> Server messages (Kiosk)
    NEW_SESSION = "new_session"
    START_CAPTURE = "start_capture"
    END_SESSION = "end_session"

    # Client -> Server messages (Phone)
    JOIN_SESSION = "join_session"
    REQUEST_PHOTO = "request_photo"


class WSMessage(BaseModel):
    """Base WebSocket message."""

    type: WSMessageType
    data: dict[str, Any] | None = None


class SessionReadyMessage(BaseModel):
    """Session ready message data."""

    sessionId: str
    qrUrl: str
    wifiQrUrl: str


class PhoneConnectedMessage(BaseModel):
    """Phone connected message data."""

    phoneId: str
    sessionId: str


class CountdownMessage(BaseModel):
    """Countdown tick message data."""

    value: int  # 3, 2, 1, 0
    sessionId: str


class PhotoReadyMessage(BaseModel):
    """Photo ready message data."""

    id: str
    sessionId: str
    sequence: int
    webUrl: str
    thumbnailUrl: str


class CaptureCompleteMessage(BaseModel):
    """Capture complete message data."""

    sessionId: str
    photoCount: int
    stripUrl: str


class SessionEndedMessage(BaseModel):
    """Session ended message data."""

    sessionId: str
