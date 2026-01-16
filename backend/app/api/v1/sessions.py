"""Session endpoints."""

import asyncio
import secrets
from datetime import datetime, timedelta, timezone

from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.responses import Response
from sqlalchemy import select, update
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import settings
from app.db.session import get_db
from app.models.session import Session, SessionStatus
from app.models.photo import Photo
from app.schemas import (
    CreateSessionResponse,
    SessionResponse,
    SessionGalleryResponse,
    PhotoResponse,
)
from app.services import get_storage_service, generate_qr_code, generate_wifi_qr_code, ws_manager
from app.services.storage import generate_photo_strip
from app.services.camera import get_shared_camera, get_capture_lock, CameraError

router = APIRouter()


@router.get("/wifi-qr")
async def get_global_wifi_qr_code(size: int = 256) -> Response:
    """Get QR code for WiFi connection (no session required)."""
    qr_image = generate_wifi_qr_code(
        settings.wifi_ssid,
        settings.wifi_password,
        size=min(size, 512)
    )

    return Response(
        content=qr_image,
        media_type="image/png",
        headers={"Cache-Control": "no-cache"},
    )


@router.get("/wifi-qr/debug")
async def debug_wifi_qr() -> dict:
    """Debug endpoint to see WiFi QR code data."""
    password = settings.wifi_password
    ssid = settings.wifi_ssid
    if password:
        wifi_string = f"WIFI:T:WPA;S:{ssid};P:{password};;"
    else:
        wifi_string = f"WIFI:S:{ssid};;"
    return {
        "ssid": ssid,
        "password": password,
        "has_password": bool(password),
        "wifi_string": wifi_string,
    }


@router.post("", response_model=CreateSessionResponse)
async def create_session(
    db: AsyncSession = Depends(get_db),
) -> CreateSessionResponse:
    """
    Create a new photo session.

    This is called by the kiosk to start a new session.
    Any existing active sessions are automatically ended.
    """
    # End existing active sessions
    await db.execute(
        update(Session)
        .where(Session.status.in_([SessionStatus.ACTIVE.value, SessionStatus.CAPTURING.value]))
        .values(status=SessionStatus.COMPLETED.value)
    )
    await db.commit()

    # Create new session
    session = Session(
        expires_at=datetime.now(timezone.utc)
        + timedelta(minutes=settings.session_expiry_minutes),
        upload_token=secrets.token_urlsafe(32),
    )

    db.add(session)
    await db.commit()
    await db.refresh(session)

    gallery_url = f"{settings.public_url}/session/{session.id}"
    qr_code_url = f"{settings.public_url}/api/v1/sessions/{session.id}/qr"
    wifi_qr_url = f"{settings.public_url}/api/v1/sessions/{session.id}/wifi-qr"

    return CreateSessionResponse(
        id=session.id,
        uploadToken=session.upload_token,
        galleryUrl=gallery_url,
        qrCodeUrl=qr_code_url,
        wifiQrUrl=wifi_qr_url,
    )


@router.get("/{session_id}", response_model=SessionResponse)
async def get_session(
    session_id: str,
    db: AsyncSession = Depends(get_db),
) -> SessionResponse:
    """Get session by ID."""
    result = await db.execute(select(Session).where(Session.id == session_id))
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )

    stats = ws_manager.get_session_stats(session_id)

    return SessionResponse(
        id=session.id,
        createdAt=session.created_at,
        expiresAt=session.expires_at,
        status=session.status,
        photoCount=session.photo_count,
        kioskConnected=stats["kiosk_connected"],
        phoneConnected=stats["phone_count"] > 0,
    )


@router.get("/{session_id}/gallery", response_model=SessionGalleryResponse)
async def get_session_gallery(
    session_id: str,
    db: AsyncSession = Depends(get_db),
) -> SessionGalleryResponse:
    """Get session gallery with photos."""
    result = await db.execute(select(Session).where(Session.id == session_id))
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )

    # Get photos
    photos_result = await db.execute(
        select(Photo)
        .where(Photo.session_id == session_id)
        .order_by(Photo.sequence)
    )
    photos = photos_result.scalars().all()

    storage = get_storage_service()
    stats = ws_manager.get_session_stats(session_id)

    photo_responses = [
        PhotoResponse(
            id=photo.id,
            sessionId=photo.session_id,
            sequence=photo.sequence,
            capturedAt=photo.captured_at,
            webUrl=storage.get_photo_url(photo.web_path),
            thumbnailUrl=storage.get_photo_url(photo.thumbnail_path),
        )
        for photo in photos
    ]

    qr_code_url = f"{settings.public_url}/api/v1/sessions/{session.id}/qr"
    strip_url = f"{settings.public_url}/api/v1/sessions/{session.id}/strip" if photos else None

    return SessionGalleryResponse(
        session=SessionResponse(
            id=session.id,
            createdAt=session.created_at,
            expiresAt=session.expires_at,
            status=session.status,
            photoCount=session.photo_count,
            kioskConnected=stats["kiosk_connected"],
            phoneConnected=stats["phone_count"] > 0,
        ),
        photos=photo_responses,
        qrCodeUrl=qr_code_url,
        stripUrl=strip_url,
    )


@router.get("/{session_id}/qr")
async def get_session_qr_code(
    session_id: str,
    size: int = 256,
    db: AsyncSession = Depends(get_db),
) -> Response:
    """Get QR code for session gallery URL."""
    result = await db.execute(select(Session).where(Session.id == session_id))
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )

    gallery_url = f"{settings.public_url}/session/{session.id}"
    qr_image = generate_qr_code(gallery_url, size=min(size, 512))

    return Response(
        content=qr_image,
        media_type="image/png",
        headers={"Cache-Control": "max-age=3600"},
    )


@router.get("/{session_id}/wifi-qr")
async def get_wifi_qr_code(
    session_id: str,
    size: int = 256,
    db: AsyncSession = Depends(get_db),
) -> Response:
    """Get QR code for WiFi connection."""
    result = await db.execute(select(Session).where(Session.id == session_id))
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )

    qr_image = generate_wifi_qr_code(
        settings.wifi_ssid,
        settings.wifi_password,
        size=min(size, 512)
    )

    return Response(
        content=qr_image,
        media_type="image/png",
        headers={"Cache-Control": "max-age=3600"},
    )


@router.get("/{session_id}/strip")
async def get_photo_strip(
    session_id: str,
    db: AsyncSession = Depends(get_db),
) -> Response:
    """Get photo strip image for the session."""
    result = await db.execute(select(Session).where(Session.id == session_id))
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )

    # Get latest photos
    photos_result = await db.execute(
        select(Photo)
        .where(Photo.session_id == session_id)
        .order_by(Photo.sequence.desc())
        .limit(settings.photos_per_capture)
    )
    photos = list(reversed(photos_result.scalars().all()))

    if not photos:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="No photos in session",
        )

    photo_paths = [settings.photos_dir / photo.web_path for photo in photos]
    strip_image = generate_photo_strip(photo_paths)

    return Response(
        content=strip_image,
        media_type="image/jpeg",
        headers={
            "Content-Disposition": f'inline; filename="picpop_strip_{session_id[:8]}.jpg"',
            "Cache-Control": "no-cache",
        },
    )


@router.post("/{session_id}/end", response_model=SessionResponse)
async def end_session(
    session_id: str,
    db: AsyncSession = Depends(get_db),
) -> SessionResponse:
    """End a session explicitly."""
    result = await db.execute(select(Session).where(Session.id == session_id))
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )

    session.status = SessionStatus.COMPLETED.value
    await db.commit()
    await db.refresh(session)

    # Notify all clients
    await ws_manager.send_session_ended(session_id)

    return SessionResponse(
        id=session.id,
        createdAt=session.created_at,
        expiresAt=session.expires_at,
        status=session.status,
        photoCount=session.photo_count,
        kioskConnected=False,
        phoneConnected=False,
    )


@router.post("/{session_id}/capture", response_model=SessionGalleryResponse)
async def capture_photos(
    session_id: str,
    db: AsyncSession = Depends(get_db),
) -> SessionGalleryResponse:
    """
    Trigger camera capture for the session.

    This runs the full capture sequence:
    1. Countdown (3, 2, 1) - broadcast to all clients
    2. Capture photos with delay
    3. Stream each photo to clients as it's ready
    4. Return final gallery
    """
    result = await db.execute(select(Session).where(Session.id == session_id))
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )

    if session.status == SessionStatus.COMPLETED.value:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Session already completed",
        )

    if session.status in [SessionStatus.CAPTURING.value, SessionStatus.COUNTDOWN.value]:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Capture already in progress",
        )

    # Check camera lock
    capturing_result = await db.execute(
        select(Session).where(
            Session.status.in_([SessionStatus.CAPTURING.value, SessionStatus.COUNTDOWN.value])
        )
    )
    if capturing_result.scalar_one_or_none():
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Camera is busy with another session",
        )

    # Check camera BEFORE starting countdown
    camera = get_shared_camera(settings.camera_backend)
    if not await camera.is_connected():
        connected = await camera.connect()
        if not connected:
            raise HTTPException(
                status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
                detail="Camera not available",
            )

    # Start countdown
    session.status = SessionStatus.COUNTDOWN.value
    await db.commit()

    # Broadcast countdown
    for i in range(settings.countdown_seconds, 0, -1):
        await ws_manager.send_countdown(session_id, i)
        await asyncio.sleep(1)

    # Signal capture start
    await ws_manager.broadcast_to_session(session_id, {
        "type": "capture_start",
        "data": {"sessionId": session_id}
    })

    session.status = SessionStatus.CAPTURING.value
    await db.commit()

    storage = get_storage_service()
    photos_captured = []
    capture_lock = get_capture_lock()

    try:
        # Acquire capture lock to pause preview stream during capture
        async with capture_lock:
            for i in range(settings.photos_per_capture):
                sequence = session.photo_count + 1

                # Capture
                original_filename = f"{session_id}_{sequence:02d}_original.jpg"
                original_path = settings.photos_dir / session_id / original_filename

                await camera.capture(original_path)

                # Process and store
                web_path, thumb_path = await storage.process_and_store(
                    original_path,
                    session_id,
                    sequence,
                    save_raw=settings.save_raw_images,
                )

                # Create photo record
                photo = Photo(
                    session_id=session_id,
                    sequence=sequence,
                    web_path=web_path,
                    thumbnail_path=thumb_path,
                )
                db.add(photo)
                session.photo_count = sequence
                await db.commit()
                await db.refresh(photo)
                photos_captured.append(photo)

                # Notify clients immediately
                await ws_manager.send_photo_ready(
                    session_id,
                    photo.id,
                    photo.sequence,
                    storage.get_photo_url(photo.web_path),
                    storage.get_photo_url(photo.thumbnail_path),
                )

                # Delay between captures
                if i < settings.photos_per_capture - 1:
                    await asyncio.sleep(settings.capture_delay_seconds)

        session.status = SessionStatus.ACTIVE.value
        await db.commit()

        # Send capture complete
        strip_url = f"{settings.public_url}/api/v1/sessions/{session_id}/strip"
        await ws_manager.send_capture_complete(session_id, session.photo_count, strip_url)

    except CameraError as e:
        await db.rollback()
        session.status = SessionStatus.ACTIVE.value
        await db.commit()
        await ws_manager.send_capture_failed(session_id, f"Camera error: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Capture failed: {e}",
        )
    except Exception as e:
        await db.rollback()
        session.status = SessionStatus.ACTIVE.value
        await db.commit()
        await ws_manager.send_capture_failed(session_id, f"Unexpected error: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=f"Unexpected error: {e}",
        )

    # Get all photos
    await db.refresh(session)
    all_photos_result = await db.execute(
        select(Photo)
        .where(Photo.session_id == session_id)
        .order_by(Photo.sequence)
    )
    all_photos = all_photos_result.scalars().all()

    stats = ws_manager.get_session_stats(session_id)
    photo_responses = [
        PhotoResponse(
            id=photo.id,
            sessionId=photo.session_id,
            sequence=photo.sequence,
            capturedAt=photo.captured_at,
            webUrl=storage.get_photo_url(photo.web_path),
            thumbnailUrl=storage.get_photo_url(photo.thumbnail_path),
        )
        for photo in all_photos
    ]

    qr_code_url = f"{settings.public_url}/api/v1/sessions/{session.id}/qr"
    strip_url = f"{settings.public_url}/api/v1/sessions/{session.id}/strip"

    return SessionGalleryResponse(
        session=SessionResponse(
            id=session.id,
            createdAt=session.created_at,
            expiresAt=session.expires_at,
            status=session.status,
            photoCount=session.photo_count,
            kioskConnected=stats["kiosk_connected"],
            phoneConnected=stats["phone_count"] > 0,
        ),
        photos=photo_responses,
        qrCodeUrl=qr_code_url,
        stripUrl=strip_url,
    )
