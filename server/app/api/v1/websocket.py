"""WebSocket endpoints for real-time communication."""

import json
from fastapi import APIRouter, WebSocket, WebSocketDisconnect, Depends
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.logging import get_logger
from app.db.session import get_db, async_session_maker
from app.models.session import Session, SessionStatus
from app.services import ws_manager

logger = get_logger(__name__)

router = APIRouter()


@router.websocket("/kiosk/{session_id}")
async def kiosk_websocket(
    websocket: WebSocket,
    session_id: str,
):
    """
    WebSocket endpoint for the kiosk touchscreen.

    The kiosk sends:
    - start_capture: Begin photo capture sequence
    - end_session: End the current session

    The kiosk receives:
    - phone_connected: A phone joined the session
    - phone_disconnected: A phone left
    - countdown: Countdown tick (3, 2, 1)
    - photo_ready: A photo was captured
    - capture_complete: All photos captured
    - session_ended: Session was ended
    """
    # Verify session exists
    async with async_session_maker() as db:
        result = await db.execute(select(Session).where(Session.id == session_id))
        session = result.scalar_one_or_none()

        if not session:
            await websocket.close(code=4004, reason="Session not found")
            return

        if session.status == SessionStatus.COMPLETED.value:
            await websocket.close(code=4001, reason="Session completed")
            return

    await ws_manager.connect_kiosk(websocket, session_id)

    # Send initial session state
    await websocket.send_json({
        "type": "kiosk_connected",
        "data": {
            "sessionId": session_id,
            "phoneCount": ws_manager.get_phone_count(session_id),
        }
    })

    try:
        while True:
            data = await websocket.receive_text()
            message = json.loads(data)
            msg_type = message.get("type")

            logger.info("Kiosk message", session_id=session_id, type=msg_type)

            if msg_type == "start_capture":
                # Trigger capture via HTTP endpoint (for proper DB handling)
                # The kiosk should call POST /api/v1/sessions/{id}/capture instead
                # This is just for acknowledgment
                await websocket.send_json({
                    "type": "ack",
                    "data": {"action": "start_capture", "sessionId": session_id}
                })

            elif msg_type == "end_session":
                # End session
                async with async_session_maker() as db:
                    result = await db.execute(
                        select(Session).where(Session.id == session_id)
                    )
                    session = result.scalar_one_or_none()
                    if session:
                        session.status = SessionStatus.COMPLETED.value
                        await db.commit()

                await ws_manager.send_session_ended(session_id)
                break

            elif msg_type == "ping":
                await websocket.send_json({"type": "pong"})

    except WebSocketDisconnect:
        logger.info("Kiosk disconnected", session_id=session_id)
    except Exception as e:
        logger.error("Kiosk WebSocket error", session_id=session_id, error=str(e))
    finally:
        await ws_manager.disconnect(websocket)


@router.websocket("/phone/{session_id}")
async def phone_websocket(
    websocket: WebSocket,
    session_id: str,
):
    """
    WebSocket endpoint for phone clients.

    Phones receive:
    - countdown: Countdown tick (3, 2, 1)
    - photo_ready: A photo was captured (with URLs)
    - capture_complete: All photos captured
    - session_ended: Session was ended

    Phones can send:
    - ping: Keep-alive
    """
    # Verify session exists
    async with async_session_maker() as db:
        result = await db.execute(select(Session).where(Session.id == session_id))
        session = result.scalar_one_or_none()

        if not session:
            await websocket.close(code=4004, reason="Session not found")
            return

        if session.status == SessionStatus.COMPLETED.value:
            await websocket.close(code=4001, reason="Session completed")
            return

    phone_id = await ws_manager.connect_phone(websocket, session_id)

    # Send current session state
    async with async_session_maker() as db:
        from app.models.photo import Photo
        from app.services import get_storage_service

        photos_result = await db.execute(
            select(Photo)
            .where(Photo.session_id == session_id)
            .order_by(Photo.sequence)
        )
        photos = photos_result.scalars().all()

        storage = get_storage_service()
        photo_data = [
            {
                "id": p.id,
                "sequence": p.sequence,
                "webUrl": storage.get_photo_url(p.web_path),
                "thumbnailUrl": storage.get_photo_url(p.thumbnail_path),
            }
            for p in photos
        ]

    await websocket.send_json({
        "type": "session_state",
        "data": {
            "sessionId": session_id,
            "phoneId": phone_id,
            "photos": photo_data,
            "kioskConnected": ws_manager.has_kiosk(session_id),
        }
    })

    try:
        while True:
            data = await websocket.receive_text()
            message = json.loads(data)
            msg_type = message.get("type")

            if msg_type == "ping":
                await websocket.send_json({"type": "pong"})

    except WebSocketDisconnect:
        logger.info("Phone disconnected", session_id=session_id, phone_id=phone_id)
    except Exception as e:
        logger.error("Phone WebSocket error", session_id=session_id, phone_id=phone_id, error=str(e))
    finally:
        await ws_manager.disconnect(websocket)
