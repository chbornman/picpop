"""WebSocket connection manager for real-time communication."""

import asyncio
import json
from typing import Any
from uuid import uuid4

from fastapi import WebSocket

from app.core.logging import get_logger

logger = get_logger(__name__)


class WebSocketManager:
    """
    Manages WebSocket connections for kiosk and phone clients.

    Architecture:
    - One kiosk connection per session (the touchscreen)
    - Multiple phone connections per session (viewers)
    - Messages are broadcast to relevant clients based on session
    """

    def __init__(self) -> None:
        # session_id -> WebSocket (only one kiosk per session)
        self._kiosk_connections: dict[str, WebSocket] = {}

        # session_id -> {phone_id: WebSocket}
        self._phone_connections: dict[str, dict[str, WebSocket]] = {}

        # WebSocket -> session_id (reverse lookup)
        self._connection_sessions: dict[WebSocket, str] = {}

        # Lock for thread-safe operations
        self._lock = asyncio.Lock()

    async def connect_kiosk(self, websocket: WebSocket, session_id: str) -> None:
        """Register a kiosk connection for a session."""
        await websocket.accept()

        async with self._lock:
            # Disconnect existing kiosk for this session if any
            if session_id in self._kiosk_connections:
                old_ws = self._kiosk_connections[session_id]
                try:
                    await old_ws.close()
                except Exception:
                    pass
                self._connection_sessions.pop(old_ws, None)

            self._kiosk_connections[session_id] = websocket
            self._connection_sessions[websocket] = session_id

        logger.info("Kiosk connected", session_id=session_id)

    async def connect_phone(self, websocket: WebSocket, session_id: str) -> str:
        """
        Register a phone connection for a session.

        Returns:
            phone_id: Unique identifier for this phone connection
        """
        await websocket.accept()
        phone_id = str(uuid4())[:8]

        async with self._lock:
            if session_id not in self._phone_connections:
                self._phone_connections[session_id] = {}

            self._phone_connections[session_id][phone_id] = websocket
            self._connection_sessions[websocket] = session_id

        logger.info("Phone connected", session_id=session_id, phone_id=phone_id)

        # Notify kiosk that a phone connected
        await self.send_to_kiosk(
            session_id,
            {"type": "phone_connected", "data": {"phoneId": phone_id, "sessionId": session_id}},
        )

        return phone_id

    async def disconnect(self, websocket: WebSocket) -> None:
        """Disconnect a WebSocket (kiosk or phone)."""
        async with self._lock:
            session_id = self._connection_sessions.pop(websocket, None)
            if not session_id:
                return

            # Check if it's a kiosk
            if self._kiosk_connections.get(session_id) == websocket:
                del self._kiosk_connections[session_id]
                logger.info("Kiosk disconnected", session_id=session_id)
                return

            # Check if it's a phone
            if session_id in self._phone_connections:
                for phone_id, ws in list(self._phone_connections[session_id].items()):
                    if ws == websocket:
                        del self._phone_connections[session_id][phone_id]
                        logger.info("Phone disconnected", session_id=session_id, phone_id=phone_id)

                        # Notify kiosk
                        await self._send_to_kiosk_unlocked(
                            session_id,
                            {
                                "type": "phone_disconnected",
                                "data": {"phoneId": phone_id, "sessionId": session_id},
                            },
                        )
                        break

                # Clean up empty session dict
                if not self._phone_connections[session_id]:
                    del self._phone_connections[session_id]

    async def send_to_kiosk(self, session_id: str, message: dict[str, Any]) -> bool:
        """Send a message to the kiosk for a session."""
        async with self._lock:
            return await self._send_to_kiosk_unlocked(session_id, message)

    async def _send_to_kiosk_unlocked(self, session_id: str, message: dict[str, Any]) -> bool:
        """Send to kiosk without acquiring lock (internal use)."""
        websocket = self._kiosk_connections.get(session_id)
        if websocket:
            try:
                await websocket.send_json(message)
                return True
            except Exception as e:
                logger.error("Failed to send to kiosk", session_id=session_id, error=str(e))
        return False

    async def send_to_phones(self, session_id: str, message: dict[str, Any]) -> int:
        """
        Send a message to all phones connected to a session.

        Returns:
            Number of phones that received the message
        """
        async with self._lock:
            phones = self._phone_connections.get(session_id, {})
            sent_count = 0

            for phone_id, websocket in list(phones.items()):
                try:
                    await websocket.send_json(message)
                    sent_count += 1
                except Exception as e:
                    logger.error(
                        "Failed to send to phone",
                        session_id=session_id,
                        phone_id=phone_id,
                        error=str(e),
                    )
                    # Remove dead connection
                    del phones[phone_id]
                    self._connection_sessions.pop(websocket, None)

            return sent_count

    async def broadcast_to_session(self, session_id: str, message: dict[str, Any]) -> None:
        """Broadcast a message to kiosk and all phones for a session."""
        await self.send_to_kiosk(session_id, message)
        await self.send_to_phones(session_id, message)

    async def send_countdown(
        self, session_id: str, value: int, photo_number: int = 1, total_photos: int = 1
    ) -> None:
        """Send countdown tick to all clients.

        Args:
            session_id: The session ID
            value: Countdown value (3, 2, 1)
            photo_number: Which photo we're counting down for (1, 2, 3...)
            total_photos: Total photos in this capture sequence
        """
        await self.broadcast_to_session(
            session_id,
            {
                "type": "countdown",
                "data": {
                    "value": value,
                    "sessionId": session_id,
                    "photoNumber": photo_number,
                    "totalPhotos": total_photos,
                },
            },
        )

    async def send_photo_ready(
        self,
        session_id: str,
        photo_id: str,
        sequence: int,
        web_url: str,
        thumbnail_url: str,
    ) -> None:
        """Notify all clients that a photo is ready."""
        await self.broadcast_to_session(
            session_id,
            {
                "type": "photo_ready",
                "data": {
                    "id": photo_id,
                    "sessionId": session_id,
                    "sequence": sequence,
                    "webUrl": web_url,
                    "thumbnailUrl": thumbnail_url,
                },
            },
        )

    async def send_capture_complete(
        self,
        session_id: str,
        photo_count: int,
        strip_url: str,
    ) -> None:
        """Notify all clients that capture is complete."""
        await self.broadcast_to_session(
            session_id,
            {
                "type": "capture_complete",
                "data": {
                    "sessionId": session_id,
                    "photoCount": photo_count,
                    "stripUrl": strip_url,
                },
            },
        )

    async def send_capture_failed(
        self,
        session_id: str,
        error: str,
    ) -> None:
        """Notify all clients that capture failed."""
        await self.broadcast_to_session(
            session_id,
            {
                "type": "capture_failed",
                "data": {
                    "sessionId": session_id,
                    "error": error,
                },
            },
        )

    async def send_session_ended(self, session_id: str) -> None:
        """Notify all clients that session has ended."""
        await self.broadcast_to_session(
            session_id, {"type": "session_ended", "data": {"sessionId": session_id}}
        )

        # Clean up connections for this session
        async with self._lock:
            # Close kiosk
            if session_id in self._kiosk_connections:
                ws = self._kiosk_connections.pop(session_id)
                self._connection_sessions.pop(ws, None)
                try:
                    await ws.close()
                except Exception:
                    pass

            # Close phones
            if session_id in self._phone_connections:
                for phone_id, ws in self._phone_connections.pop(session_id, {}).items():
                    self._connection_sessions.pop(ws, None)
                    try:
                        await ws.close()
                    except Exception:
                        pass

    def get_session_stats(self, session_id: str) -> dict[str, Any]:
        """Get connection stats for a session."""
        return {
            "kiosk_connected": session_id in self._kiosk_connections,
            "phone_count": len(self._phone_connections.get(session_id, {})),
        }

    def has_kiosk(self, session_id: str) -> bool:
        """Check if a kiosk is connected for the session."""
        return session_id in self._kiosk_connections

    def get_phone_count(self, session_id: str) -> int:
        """Get number of connected phones for a session."""
        return len(self._phone_connections.get(session_id, {}))


# Global WebSocket manager instance
ws_manager = WebSocketManager()
