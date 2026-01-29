"""Camera preview streaming."""

import asyncio
import logging
from typing import AsyncGenerator

from fastapi import APIRouter
from fastapi.responses import StreamingResponse, JSONResponse

from app.core.config import settings
from app.services.camera import (
    get_camera,
    wait_if_paused,
    is_preview_paused,
    CameraError,
)

logger = logging.getLogger(__name__)
router = APIRouter()


async def mjpeg_stream(fps: int = 30) -> AsyncGenerator[bytes, None]:
    """Generate MJPEG frames from camera."""
    camera = get_camera(settings.camera_backend)
    frame_interval = 1.0 / fps if fps > 0 else 0
    frame_count = 0

    # Exponential backoff state
    connect_retry_delay = 1.0
    max_connect_retry_delay = 5.0
    error_retry_delay = 0.5
    max_error_retry_delay = 5.0

    logger.info(f"Starting preview stream at {fps} fps")

    try:
        while True:
            # Wait if capture is happening
            await wait_if_paused()

            # Connect if needed
            if not camera.is_connected():
                connected = await camera.connect()
                if not connected:
                    logger.warning(f"Camera connect failed, retrying in {connect_retry_delay:.1f}s")
                    await asyncio.sleep(connect_retry_delay)
                    # Exponential backoff for connection failures
                    connect_retry_delay = min(connect_retry_delay * 1.5, max_connect_retry_delay)
                    continue
                else:
                    # Reset backoff on successful connection
                    connect_retry_delay = 1.0
                    error_retry_delay = 0.5

            try:
                frame = await camera.get_preview_frame()
                frame_count += 1

                # Reset error backoff on successful frame
                error_retry_delay = 0.5

                if frame_count == 1:
                    logger.info("First preview frame captured")
                elif frame_count % 300 == 0:
                    logger.debug(f"Preview: {frame_count} frames")

                yield (
                    b"--frame\r\n"
                    b"Content-Type: image/jpeg\r\n"
                    b"Content-Length: " + str(len(frame)).encode() + b"\r\n"
                    b"\r\n" + frame + b"\r\n"
                )

                if frame_interval > 0:
                    await asyncio.sleep(frame_interval)

            except CameraError as e:
                logger.warning(f"Preview frame error: {e}, retrying in {error_retry_delay:.1f}s")
                await asyncio.sleep(error_retry_delay)
                # Exponential backoff for preview errors
                error_retry_delay = min(error_retry_delay * 1.5, max_error_retry_delay)
            except Exception as e:
                logger.error(f"Unexpected preview error: {e}")
                await asyncio.sleep(0.1)

    except asyncio.CancelledError:
        logger.info(f"Preview stream cancelled after {frame_count} frames")
        raise  # Re-raise to properly clean up
    finally:
        logger.info("Preview stream ended")


@router.get("/preview")
async def preview(fps: int = 30):
    """
    Stream live camera preview as MJPEG.

    Use in an img tag: <img src="/api/v1/camera/preview" />
    """
    target_fps = max(1, min(fps, 60))

    return StreamingResponse(
        mjpeg_stream(target_fps),
        media_type="multipart/x-mixed-replace; boundary=frame",
        headers={
            "Cache-Control": "no-cache, no-store, must-revalidate",
            "Pragma": "no-cache",
            "Expires": "0",
            "Access-Control-Allow-Origin": "*",
        },
    )


@router.get("/frame")
async def single_frame():
    """
    Get a single preview frame as JPEG.

    Use this for polling-based preview in environments where
    MJPEG streams don't work well (like Tauri WebView).
    """
    from fastapi.responses import Response

    camera = get_camera(settings.camera_backend)

    if not camera.is_connected():
        if not await camera.connect():
            return Response(status_code=503, content=b"Camera not connected")

    try:
        frame = await camera.get_preview_frame()
        return Response(
            content=frame,
            media_type="image/jpeg",
            headers={
                "Cache-Control": "no-cache, no-store",
                "Access-Control-Allow-Origin": "*",
            },
        )
    except CameraError as e:
        return Response(status_code=503, content=str(e).encode())


@router.get("/status")
async def status():
    """Get camera status."""
    camera = get_camera(settings.camera_backend)
    return JSONResponse(
        {
            "connected": camera.is_connected(),
            "previewPaused": is_preview_paused(),
        }
    )


@router.post("/connect")
async def connect():
    """Connect to camera."""
    camera = get_camera(settings.camera_backend)
    connected = await camera.connect()
    return JSONResponse({"connected": connected})


@router.post("/disconnect")
async def disconnect():
    """Disconnect from camera."""
    camera = get_camera(settings.camera_backend)
    await camera.disconnect()
    return JSONResponse({"connected": False})


@router.post("/reset")
async def reset():
    """Reset camera connection."""
    camera = get_camera(settings.camera_backend)

    if camera.is_connected():
        await camera.disconnect()

    connected = await camera.connect()
    return JSONResponse({"connected": connected})
