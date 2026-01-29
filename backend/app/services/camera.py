"""Simple camera service for photo booth."""

import asyncio
import logging
import time
from abc import ABC, abstractmethod
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional

logger = logging.getLogger(__name__)


@dataclass
class CameraStats:
    """Track camera statistics for debugging."""

    connect_count: int = 0
    disconnect_count: int = 0
    preview_frame_count: int = 0
    preview_error_count: int = 0
    capture_count: int = 0
    capture_error_count: int = 0
    last_preview_time: Optional[float] = None
    last_capture_time: Optional[float] = None
    last_error: Optional[str] = None
    last_error_time: Optional[float] = None

    def log_summary(self) -> None:
        """Log a summary of camera stats."""
        preview_age = ""
        if self.last_preview_time:
            age_ms = (time.time() - self.last_preview_time) * 1000
            preview_age = f", last_preview={age_ms:.0f}ms ago"

        logger.info(
            f"[CAMERA STATS] connects={self.connect_count}, disconnects={self.disconnect_count}, "
            f"previews={self.preview_frame_count}, preview_errors={self.preview_error_count}, "
            f"captures={self.capture_count}, capture_errors={self.capture_error_count}"
            f"{preview_age}"
        )


# Global stats instance
_camera_stats = CameraStats()


class CameraError(Exception):
    """Base camera error."""


class CameraNotConnected(CameraError):
    """Camera is not connected."""


class CaptureError(CameraError):
    """Failed to capture."""


class Camera(ABC):
    """Abstract camera interface."""

    @abstractmethod
    async def connect(self) -> bool:
        """Connect to camera. Returns True if successful."""

    @abstractmethod
    async def disconnect(self) -> None:
        """Disconnect from camera."""

    @abstractmethod
    async def capture(self, save_path: Path) -> Path:
        """Capture photo and save to path. Returns the path."""

    @abstractmethod
    async def get_preview_frame(self) -> bytes:
        """Get a single preview frame as JPEG bytes."""

    @abstractmethod
    def is_connected(self) -> bool:
        """Check if camera is connected."""


class GPhoto2Camera(Camera):
    """Camera using gphoto2 library."""

    def __init__(self) -> None:
        self._camera = None
        self._context = None
        self._lock = asyncio.Lock()

    async def connect(self) -> bool:
        import gphoto2 as gp

        async with self._lock:
            if self._camera is not None:
                logger.debug("[CAMERA] Already connected")
                return True

            try:
                logger.info("[CAMERA] Connecting to camera...")
                self._context = gp.Context()
                self._camera = gp.Camera()
                await asyncio.to_thread(self._camera.init, self._context)

                summary = await asyncio.to_thread(self._camera.get_summary, self._context)
                _camera_stats.connect_count += 1
                logger.info(f"[CAMERA] Connected successfully: {summary.text[:80]}...")
                _camera_stats.log_summary()
                return True

            except gp.GPhoto2Error as e:
                _camera_stats.last_error = str(e)
                _camera_stats.last_error_time = time.time()
                logger.error(f"[CAMERA] Failed to connect: {e}")
                self._camera = None
                self._context = None
                return False

    async def disconnect(self) -> None:
        async with self._lock:
            if self._camera:
                try:
                    logger.info("[CAMERA] Disconnecting...")
                    await asyncio.to_thread(self._camera.exit, self._context)
                except Exception as e:
                    logger.warning(f"[CAMERA] Disconnect error: {e}")
                finally:
                    self._camera = None
                    self._context = None
                    _camera_stats.disconnect_count += 1
                    logger.info("[CAMERA] Disconnected")
                    _camera_stats.log_summary()

    async def capture(self, save_path: Path) -> Path:
        import gphoto2 as gp

        async with self._lock:
            if not self._camera:
                raise CameraNotConnected("Camera not connected")

            try:
                logger.info("[CAMERA] Capturing photo...")
                start_time = time.time()

                file_path = await asyncio.to_thread(
                    self._camera.capture, gp.GP_CAPTURE_IMAGE, self._context
                )
                capture_time = time.time() - start_time
                logger.info(
                    f"[CAMERA] Captured in {capture_time:.2f}s: {file_path.folder}/{file_path.name}"
                )

                # Download from camera
                download_start = time.time()
                camera_file = gp.CameraFile()
                await asyncio.to_thread(
                    self._camera.file_get,
                    file_path.folder,
                    file_path.name,
                    gp.GP_FILE_TYPE_NORMAL,
                    camera_file,
                    self._context,
                )

                save_path.parent.mkdir(parents=True, exist_ok=True)
                await asyncio.to_thread(camera_file.save, str(save_path))
                download_time = time.time() - download_start

                _camera_stats.capture_count += 1
                _camera_stats.last_capture_time = time.time()
                logger.info(f"[CAMERA] Saved in {download_time:.2f}s: {save_path}")
                return save_path

            except gp.GPhoto2Error as e:
                # Reset on any gphoto2 error - camera may be disconnected
                _camera_stats.capture_error_count += 1
                _camera_stats.last_error = str(e)
                _camera_stats.last_error_time = time.time()
                logger.error(f"[CAMERA] Capture failed, resetting camera: {e}")
                _camera_stats.log_summary()
                self._camera = None
                self._context = None
                raise CaptureError(f"Capture failed: {e}")

    async def get_preview_frame(self) -> bytes:
        import gphoto2 as gp

        async with self._lock:
            if not self._camera:
                raise CameraNotConnected("Camera not connected")

            try:
                camera_file = await asyncio.to_thread(self._camera.capture_preview)
                data = await asyncio.to_thread(camera_file.get_data_and_size)

                _camera_stats.preview_frame_count += 1
                _camera_stats.last_preview_time = time.time()

                # Log every 100 frames
                if _camera_stats.preview_frame_count % 100 == 0:
                    logger.debug(f"[CAMERA] Preview frames: {_camera_stats.preview_frame_count}")

                return bytes(data)

            except gp.GPhoto2Error as e:
                # Reset on any gphoto2 error - camera may be disconnected
                _camera_stats.preview_error_count += 1
                _camera_stats.last_error = str(e)
                _camera_stats.last_error_time = time.time()
                logger.warning(f"[CAMERA] Preview error, resetting camera: {e}")
                _camera_stats.log_summary()
                self._camera = None
                self._context = None
                raise CaptureError(f"Preview failed: {e}")

    def is_connected(self) -> bool:
        return self._camera is not None


class MockCamera(Camera):
    """Mock camera for testing."""

    def __init__(self) -> None:
        self._connected = False
        self._capture_count = 0
        self._preview_count = 0

    async def connect(self) -> bool:
        self._connected = True
        logger.info("Mock camera connected")
        return True

    async def disconnect(self) -> None:
        self._connected = False
        logger.info("Mock camera disconnected")

    async def capture(self, save_path: Path) -> Path:
        from PIL import Image, ImageDraw, ImageFont
        import random

        if not self._connected:
            raise CameraNotConnected("Mock camera not connected")

        self._capture_count += 1

        # Create test image
        colors = [(75, 0, 130), (138, 43, 226), (255, 20, 147), (0, 191, 255)]
        img = Image.new("RGB", (1920, 1280), random.choice(colors))
        draw = ImageDraw.Draw(img)

        text = f"PicPop #{self._capture_count}"
        try:
            font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 96)
        except OSError:
            font = ImageFont.load_default()

        bbox = draw.textbbox((0, 0), text, font=font)
        x = (1920 - (bbox[2] - bbox[0])) // 2
        y = (1280 - (bbox[3] - bbox[1])) // 2
        draw.text((x, y), text, fill=(255, 255, 255), font=font)

        save_path.parent.mkdir(parents=True, exist_ok=True)
        img.save(save_path, "JPEG", quality=95)
        logger.info(f"Mock capture: {save_path}")

        await asyncio.sleep(0.2)
        return save_path

    async def get_preview_frame(self) -> bytes:
        from PIL import Image, ImageDraw, ImageFont
        from datetime import datetime
        import io

        if not self._connected:
            raise CameraNotConnected("Mock camera not connected")

        self._preview_count += 1

        # Create preview frame
        colors = [(75, 0, 130), (138, 43, 226), (255, 20, 147), (0, 191, 255)]
        img = Image.new("RGB", (640, 480), colors[self._preview_count % len(colors)])
        draw = ImageDraw.Draw(img)

        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        text = f"PREVIEW\n{timestamp}"

        try:
            font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 36)
        except OSError:
            font = ImageFont.load_default()

        bbox = draw.textbbox((0, 0), text, font=font)
        x = (640 - (bbox[2] - bbox[0])) // 2
        y = (480 - (bbox[3] - bbox[1])) // 2
        draw.text((x, y), text, fill=(255, 255, 255), font=font, align="center")

        # Viewfinder corners
        for cx, cy in [(40, 40), (600, 40), (40, 440), (600, 440)]:
            draw.line([(cx - 20, cy), (cx + 20, cy)], fill=(255, 255, 255), width=2)
            draw.line([(cx, cy - 20), (cx, cy + 20)], fill=(255, 255, 255), width=2)

        output = io.BytesIO()
        img.save(output, format="JPEG", quality=80)
        return output.getvalue()

    def is_connected(self) -> bool:
        return self._connected


# Global camera instance and state
_camera: Camera | None = None
_preview_paused = asyncio.Event()
_preview_paused.set()  # Not paused by default


def get_camera(backend: str = "gphoto2") -> Camera:
    """Get the shared camera instance."""
    global _camera
    if _camera is None:
        if backend == "mock":
            _camera = MockCamera()
        else:
            _camera = GPhoto2Camera()
    return _camera


def pause_preview() -> None:
    """Pause preview streaming (call before capture)."""
    _preview_paused.clear()
    logger.debug("Preview paused")


def resume_preview() -> None:
    """Resume preview streaming (call after capture)."""
    _preview_paused.set()
    logger.debug("Preview resumed")


async def wait_if_paused() -> None:
    """Wait if preview is paused. Used by preview stream."""
    await _preview_paused.wait()


def is_preview_paused() -> bool:
    """Check if preview is currently paused."""
    return not _preview_paused.is_set()
