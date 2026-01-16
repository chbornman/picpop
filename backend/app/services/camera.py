"""Camera service using gphoto2 for capture control."""

from __future__ import annotations

import asyncio
import logging
from abc import ABC, abstractmethod
from pathlib import Path

logger = logging.getLogger(__name__)


class CameraError(Exception):
    """Base exception for camera errors."""


class CameraNotFoundError(CameraError):
    """No camera detected."""


class CaptureError(CameraError):
    """Failed to capture image."""


class CameraService(ABC):
    """Abstract camera interface."""

    @abstractmethod
    async def connect(self) -> bool:
        """Connect to camera. Returns True if successful. Safe to call multiple times."""

    @abstractmethod
    async def disconnect(self) -> None:
        """Disconnect from camera."""

    @abstractmethod
    async def capture(self, save_path: Path) -> Path:
        """Capture image and save to path. Returns the saved file path."""

    @abstractmethod
    async def capture_preview(self) -> bytes:
        """Capture a single preview frame. Returns JPEG bytes."""

    @abstractmethod
    async def is_connected(self) -> bool:
        """Check if camera is connected."""

    @abstractmethod
    async def ensure_connected(self) -> bool:
        """Ensure camera is connected, connecting if needed. Returns True if connected."""

    @abstractmethod
    def supports_preview(self) -> bool:
        """Check if camera supports live preview."""


class PreviewNotSupportedError(CameraError):
    """Camera doesn't support preview."""


class GPhoto2Camera(CameraService):
    """Camera implementation using gphoto2."""

    def __init__(self) -> None:
        self._camera: "gp.Camera | None" = None
        self._context: "gp.Context | None" = None
        self._supports_preview: bool = False
        self._preview_lock = asyncio.Lock()
        self._connection_lock = asyncio.Lock()

    async def connect(self) -> bool:
        """Connect to the first available camera. Safe to call multiple times."""
        import gphoto2 as gp

        async with self._connection_lock:
            # Already connected - return success
            if self._camera is not None:
                logger.debug("Camera already connected")
                return True

            try:
                self._context = gp.Context()
                self._camera = gp.Camera()
                await asyncio.to_thread(self._camera.init, self._context)

                # Get camera summary to verify connection
                summary = await asyncio.to_thread(
                    self._camera.get_summary, self._context
                )
                logger.info(f"Connected to camera: {summary.text[:100]}...")

                # Test if camera supports preview
                try:
                    await asyncio.to_thread(self._camera.capture_preview)
                    self._supports_preview = True
                    logger.info("Camera supports live preview")
                except gp.GPhoto2Error:
                    self._supports_preview = False
                    logger.info("Camera does not support live preview")

                return True
            except gp.GPhoto2Error as e:
                logger.error(f"Failed to connect to camera: {e}")
                self._camera = None
                self._context = None
                return False

    async def disconnect(self) -> None:
        """Disconnect from camera."""
        async with self._connection_lock:
            if self._camera:
                try:
                    await asyncio.to_thread(self._camera.exit, self._context)
                except Exception as e:
                    logger.warning(f"Error disconnecting camera: {e}")
                finally:
                    self._camera = None
                    self._context = None
                    logger.info("Camera disconnected")

    async def capture(self, save_path: Path) -> Path:
        """Capture image and save to specified path."""
        import gphoto2 as gp

        if not self._camera or not self._context:
            raise CameraNotFoundError("Camera not connected")

        try:
            # Capture image
            logger.info("Triggering capture...")
            file_path = await asyncio.to_thread(
                self._camera.capture, gp.GP_CAPTURE_IMAGE, self._context
            )
            logger.info(f"Captured: {file_path.folder}/{file_path.name}")

            # Download from camera
            camera_file = gp.CameraFile()
            await asyncio.to_thread(
                self._camera.file_get,
                file_path.folder,
                file_path.name,
                gp.GP_FILE_TYPE_NORMAL,
                camera_file,
                self._context,
            )

            # Ensure directory exists
            save_path.parent.mkdir(parents=True, exist_ok=True)

            # Save to disk
            await asyncio.to_thread(camera_file.save, str(save_path))
            logger.info(f"Saved to: {save_path}")

            return save_path

        except gp.GPhoto2Error as e:
            raise CaptureError(f"Capture failed: {e}") from e

    async def is_connected(self) -> bool:
        """Check if camera is connected."""
        return self._camera is not None

    async def ensure_connected(self) -> bool:
        """Ensure camera is connected, connecting if needed. Returns True if connected."""
        if self._camera is not None:
            return True
        return await self.connect()

    async def capture_preview(self) -> bytes:
        """Capture a single preview frame from the camera."""
        import gphoto2 as gp

        if not self._camera or not self._context:
            raise CameraNotFoundError("Camera not connected")

        if not self._supports_preview:
            raise PreviewNotSupportedError("Camera does not support preview")

        async with self._preview_lock:
            try:
                # Capture preview frame
                camera_file = await asyncio.to_thread(
                    self._camera.capture_preview
                )

                # Get the data as bytes
                file_data = await asyncio.to_thread(
                    camera_file.get_data_and_size
                )
                return bytes(file_data)

            except gp.GPhoto2Error as e:
                raise CaptureError(f"Preview capture failed: {e}") from e

    def supports_preview(self) -> bool:
        """Check if camera supports live preview."""
        return self._supports_preview


class MockCamera(CameraService):
    """Mock camera for testing without hardware."""

    def __init__(self) -> None:
        self._connected = False
        self._capture_count = 0
        self._preview_count = 0

    async def connect(self) -> bool:
        """Simulate camera connection. Safe to call multiple times."""
        if self._connected:
            return True
        logger.info("Mock camera connected")
        self._connected = True
        return True

    async def disconnect(self) -> None:
        """Simulate camera disconnection."""
        logger.info("Mock camera disconnected")
        self._connected = False

    async def capture(self, save_path: Path) -> Path:
        """Generate a test image."""
        from PIL import Image, ImageDraw, ImageFont
        import random

        if not self._connected:
            raise CameraNotFoundError("Mock camera not connected")

        self._capture_count += 1

        # Create a test image with random color
        colors = [
            (75, 0, 130),    # Indigo
            (138, 43, 226),  # Blue Violet
            (255, 20, 147),  # Deep Pink
            (0, 191, 255),   # Deep Sky Blue
            (50, 205, 50),   # Lime Green
        ]
        bg_color = random.choice(colors)

        img = Image.new("RGB", (1920, 1280), color=bg_color)
        draw = ImageDraw.Draw(img)

        # Draw some text
        text = f"PicPop #{self._capture_count}"
        try:
            font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 96)
        except OSError:
            try:
                font = ImageFont.truetype("/usr/share/fonts/TTF/DejaVuSans-Bold.ttf", 96)
            except OSError:
                font = ImageFont.load_default()

        # Center the text
        bbox = draw.textbbox((0, 0), text, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        x = (1920 - text_width) // 2
        y = (1280 - text_height) // 2
        draw.text((x, y), text, fill=(255, 255, 255), font=font)

        # Add some decorative elements
        for _ in range(20):
            cx = random.randint(50, 1870)
            cy = random.randint(50, 1230)
            r = random.randint(10, 40)
            draw.ellipse([(cx-r, cy-r), (cx+r, cy+r)], fill=(255, 255, 255, 100))

        # Save
        save_path.parent.mkdir(parents=True, exist_ok=True)
        img.save(save_path, "JPEG", quality=95)
        logger.info(f"Mock capture saved to: {save_path}")

        # Simulate capture delay
        await asyncio.sleep(0.3)

        return save_path

    async def is_connected(self) -> bool:
        """Check mock connection status."""
        return self._connected

    async def ensure_connected(self) -> bool:
        """Ensure mock camera is connected."""
        if self._connected:
            return True
        return await self.connect()

    async def capture_preview(self) -> bytes:
        """Generate a mock preview frame."""
        from PIL import Image, ImageDraw, ImageFont
        from datetime import datetime
        import io

        if not self._connected:
            raise CameraNotFoundError("Mock camera not connected")

        self._preview_count += 1

        # Create a preview image (smaller than capture)
        width, height = 640, 480

        # Cycle through colors for visual feedback
        colors = [
            (75, 0, 130),    # Indigo
            (138, 43, 226),  # Blue Violet
            (255, 20, 147),  # Deep Pink
            (0, 191, 255),   # Deep Sky Blue
            (50, 205, 50),   # Lime Green
        ]
        bg_color = colors[self._preview_count % len(colors)]

        img = Image.new("RGB", (width, height), color=bg_color)
        draw = ImageDraw.Draw(img)

        # Add timestamp for visual feedback that frames are updating
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        text = f"PREVIEW #{self._preview_count}\n{timestamp}"

        try:
            font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 36)
        except OSError:
            try:
                font = ImageFont.truetype("/usr/share/fonts/TTF/DejaVuSans-Bold.ttf", 36)
            except OSError:
                font = ImageFont.load_default()

        # Center the text
        bbox = draw.textbbox((0, 0), text, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        x = (width - text_width) // 2
        y = (height - text_height) // 2
        draw.text((x, y), text, fill=(255, 255, 255), font=font, align="center")

        # Draw border to make it look like a viewfinder
        border_color = (255, 255, 255, 128)
        draw.rectangle([(20, 20), (width - 20, height - 20)], outline=border_color, width=2)

        # Corner markers
        corner_len = 30
        for cx, cy in [(40, 40), (width - 40, 40), (40, height - 40), (width - 40, height - 40)]:
            if cx < width // 2:
                draw.line([(cx, cy), (cx + corner_len, cy)], fill=(255, 255, 255), width=3)
            else:
                draw.line([(cx, cy), (cx - corner_len, cy)], fill=(255, 255, 255), width=3)
            if cy < height // 2:
                draw.line([(cx, cy), (cx, cy + corner_len)], fill=(255, 255, 255), width=3)
            else:
                draw.line([(cx, cy), (cx, cy - corner_len)], fill=(255, 255, 255), width=3)

        # Save to bytes
        output = io.BytesIO()
        img.save(output, format="JPEG", quality=80)
        return output.getvalue()

    def supports_preview(self) -> bool:
        """Mock camera always supports preview."""
        return True


def create_camera(backend: str = "gphoto2") -> CameraService:
    """Factory to create camera service based on backend type."""
    if backend == "mock":
        return MockCamera()
    elif backend == "gphoto2":
        return GPhoto2Camera()
    else:
        raise ValueError(f"Unknown camera backend: {backend}")


# Global shared camera instance and lock
_shared_camera: CameraService | None = None
_capture_lock = asyncio.Lock()


def get_shared_camera(backend: str = "gphoto2") -> CameraService:
    """Get the shared camera singleton instance."""
    global _shared_camera
    if _shared_camera is None:
        _shared_camera = create_camera(backend)
    return _shared_camera


def get_capture_lock() -> asyncio.Lock:
    """Get the capture lock for coordinating preview/capture."""
    return _capture_lock
