"""Camera preview streaming endpoint."""

import asyncio
from typing import AsyncGenerator

from fastapi import APIRouter
from fastapi.responses import StreamingResponse, JSONResponse

from app.core.config import settings
from app.services.camera import (
    get_shared_camera,
    get_capture_lock,
    CameraNotFoundError,
    CaptureError,
)

router = APIRouter()

_preview_active = False


async def generate_mjpeg_frames(target_fps: int = 30) -> AsyncGenerator[bytes, None]:
    """
    Generator that yields MJPEG frames from the camera.

    Each frame is wrapped in multipart boundaries for MJPEG streaming.
    Pauses when capture is in progress (respects capture lock).
    """
    global _preview_active
    _preview_active = True

    camera = get_shared_camera(settings.camera_backend)
    capture_lock = get_capture_lock()

    # Target frame interval (0 = as fast as possible)
    frame_interval = 1.0 / target_fps if target_fps > 0 else 0

    try:
        while _preview_active:
            # Check if capture is in progress - if so, wait
            if capture_lock.locked():
                await asyncio.sleep(0.1)
                continue

            # Ensure camera is connected before each frame attempt
            if not await camera.ensure_connected():
                await asyncio.sleep(1.0)
                continue

            try:
                # Capture preview frame
                frame_data = await camera.capture_preview()

                # Yield MJPEG frame with boundary
                yield (
                    b"--frame\r\n"
                    b"Content-Type: image/jpeg\r\n"
                    b"Content-Length: " + str(len(frame_data)).encode() + b"\r\n"
                    b"\r\n" + frame_data + b"\r\n"
                )

                # Control frame rate (if specified)
                if frame_interval > 0:
                    await asyncio.sleep(frame_interval)

            except (CaptureError, CameraNotFoundError):
                # Camera busy or disconnected, wait and retry
                await asyncio.sleep(0.5)
            except Exception:
                # Other error, wait and retry
                await asyncio.sleep(0.1)
    finally:
        _preview_active = False


@router.get("/preview")
async def camera_preview_stream(fps: int = 30):
    """
    Stream live camera preview as MJPEG.

    This endpoint returns a multipart stream that can be used directly
    in an <img> tag for live preview:

        <img src="/api/v1/camera/preview" />
        <img src="/api/v1/camera/preview?fps=60" />

    Args:
        fps: Target frames per second (default 30, use 0 for max speed)

    Note: The stream will auto-reconnect if the camera is unavailable.
    """
    # Clamp FPS to reasonable range
    target_fps = max(0, min(fps, 60))

    # Start the stream - it will handle connection internally
    # This allows the stream to reconnect if camera becomes available later
    return StreamingResponse(
        generate_mjpeg_frames(target_fps=target_fps),
        media_type="multipart/x-mixed-replace; boundary=frame",
        headers={
            "Cache-Control": "no-cache, no-store, must-revalidate",
            "Pragma": "no-cache",
            "Expires": "0",
            "Access-Control-Allow-Origin": "*",
        },
    )


@router.get("/preview/status")
async def preview_status():
    """
    Get camera preview status.

    Returns whether the camera is connected and supports preview.
    """
    camera = get_shared_camera(settings.camera_backend)
    connected = await camera.is_connected()

    return JSONResponse({
        "connected": connected,
        "supportsPreview": camera.supports_preview() if connected else False,
        "previewActive": _preview_active,
    })


@router.post("/preview/stop")
async def stop_preview():
    """Stop the preview stream."""
    global _preview_active
    _preview_active = False
    return JSONResponse({"status": "stopped"})
