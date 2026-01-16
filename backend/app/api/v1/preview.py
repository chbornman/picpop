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

    logger.info(f"Starting preview stream at {fps} fps")

    while True:
        # Wait if capture is happening
        await wait_if_paused()

        # Connect if needed
        if not camera.is_connected():
            connected = await camera.connect()
            if not connected:
                await asyncio.sleep(1.0)
                continue

        try:
            frame = await camera.get_preview_frame()
            frame_count += 1

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
            logger.warning(f"Preview frame error: {e}")
            await asyncio.sleep(0.5)
        except Exception as e:
            logger.error(f"Unexpected preview error: {e}")
            await asyncio.sleep(0.1)


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
    return JSONResponse({
        "connected": camera.is_connected(),
        "previewPaused": is_preview_paused(),
    })


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
