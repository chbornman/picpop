"""Storage service for handling photo files."""

import io
from abc import ABC, abstractmethod
from datetime import datetime
from pathlib import Path
from uuid import uuid4

from PIL import Image, ImageDraw, ImageFont

from app.core.config import settings
from app.core.logging import get_logger

logger = get_logger(__name__)


class StorageService(ABC):
    """Abstract base class for storage services."""

    @abstractmethod
    async def process_and_store(
        self,
        source_path: Path,
        session_id: str,
        sequence: int,
        save_raw: bool = False,
    ) -> tuple[str, str]:
        """
        Process a captured photo and store web + thumbnail versions.

        Returns:
            Tuple of (web_path, thumbnail_path)
        """
        pass

    @abstractmethod
    async def delete_session_photos(self, session_id: str) -> None:
        """Delete all photos for a session."""
        pass

    @abstractmethod
    def get_photo_url(self, path: str) -> str:
        """Get public URL for a photo path."""
        pass


class LocalStorageService(StorageService):
    """Local filesystem storage service."""

    def __init__(self, base_dir: Path, public_url: str) -> None:
        self.base_dir = base_dir
        self.public_url = public_url.rstrip("/")

    async def process_and_store(
        self,
        source_path: Path,
        session_id: str,
        sequence: int,
        save_raw: bool = False,
    ) -> tuple[str, str]:
        """Process captured photo and store web + thumbnail versions."""
        session_dir = self.base_dir / session_id
        session_dir.mkdir(parents=True, exist_ok=True)

        # Read source image
        image_data = source_path.read_bytes()

        photo_id = str(uuid4())[:8]
        ext = source_path.suffix if save_raw else ".jpg"
        web_filename = f"web_{sequence:03d}_{photo_id}{ext}"
        thumb_filename = f"thumb_{sequence:03d}_{photo_id}.jpg"

        # Save web version
        web_path = session_dir / web_filename
        if save_raw:
            web_path.write_bytes(image_data)
            web_size = len(image_data)
        else:
            web_image = process_image(image_data, max_width=1920, quality=90)
            web_path.write_bytes(web_image)
            web_size = len(web_image)

        # Always create thumbnail
        thumb_image = process_image(image_data, max_width=settings.thumbnail_max_width, quality=80)
        thumb_path = session_dir / thumb_filename
        thumb_path.write_bytes(thumb_image)

        logger.info(
            "Saved photo",
            session_id=session_id,
            sequence=sequence,
            web_path=str(web_path),
            original_size=len(image_data),
            web_size=web_size,
            save_raw=save_raw,
        )

        return f"{session_id}/{web_filename}", f"{session_id}/{thumb_filename}"

    async def delete_session_photos(self, session_id: str) -> None:
        """Delete all photos for a session."""
        session_dir = self.base_dir / session_id
        if session_dir.exists():
            import shutil
            shutil.rmtree(session_dir)
            logger.info("Deleted session photos", session_id=session_id)

    def get_photo_url(self, path: str) -> str:
        """Get URL for a photo (relative for flexibility)."""
        return f"/photos/{path}"


def process_image(
    image_data: bytes,
    max_width: int,
    quality: int = 85,
) -> bytes:
    """Process image - resize and optimize."""
    img = Image.open(io.BytesIO(image_data))

    # Convert to RGB if necessary
    if img.mode in ("RGBA", "P"):
        img = img.convert("RGB")

    # Resize if needed
    if img.width > max_width:
        ratio = max_width / img.width
        new_height = int(img.height * ratio)
        img = img.resize((max_width, new_height), Image.Resampling.LANCZOS)

    # Save to bytes
    output = io.BytesIO()
    img.save(output, format="JPEG", quality=quality, optimize=True)
    return output.getvalue()


def generate_photo_strip(
    photo_paths: list[Path],
    strip_width: int = 1080,
) -> bytes:
    """
    Generate a stylized vertical photo strip.

    Args:
        photo_paths: List of paths to photo files
        strip_width: Width of the final strip

    Returns:
        JPEG bytes of the photo strip
    """
    if not photo_paths:
        raise ValueError("No photos provided")

    # Design parameters
    outer_padding = 54
    inner_padding = 27
    photo_border = 14
    header_height = 150
    footer_height = 108
    corner_radius = 27

    photo_area_width = strip_width - (outer_padding * 2)
    photo_width = photo_area_width - (photo_border * 2)

    # Load and resize images
    images = []
    for path in photo_paths:
        img = Image.open(path)
        if img.mode in ("RGBA", "P"):
            img = img.convert("RGB")

        ratio = photo_width / img.width
        new_height = int(img.height * ratio)
        img = img.resize((photo_width, new_height), Image.Resampling.LANCZOS)
        images.append(img)

    # Calculate total height
    total_photo_height = sum(img.height + (photo_border * 2) for img in images)
    total_inner_padding = inner_padding * (len(images) - 1)
    content_height = header_height + total_photo_height + total_inner_padding + footer_height
    total_height = content_height + (outer_padding * 2)

    # Create strip with gradient background
    strip = Image.new("RGB", (strip_width, total_height), (255, 255, 255))
    draw = ImageDraw.Draw(strip)

    # Draw gradient background
    for y in range(total_height):
        ratio = y / total_height
        r = int(250 + (255 - 250) * ratio)
        g = int(245 + (240 - 245) * ratio)
        b = int(255 + (250 - 255) * ratio)
        draw.line([(0, y), (strip_width, y)], fill=(r, g, b))

    # Draw film strip holes
    hole_radius = 12
    hole_spacing = 72
    hole_color = (220, 215, 230)
    for y in range(outer_padding + 36, total_height - outer_padding, hole_spacing):
        draw.ellipse(
            [(18 - hole_radius, y - hole_radius), (18 + hole_radius, y + hole_radius)],
            fill=hole_color
        )
        draw.ellipse(
            [(strip_width - 18 - hole_radius, y - hole_radius),
             (strip_width - 18 + hole_radius, y + hole_radius)],
            fill=hole_color
        )

    # Load fonts
    title_font = None
    small_font = None
    font_paths = [
        "/usr/share/fonts/liberation/LiberationSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
    ]
    font_paths_regular = [
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ]
    for path in font_paths:
        try:
            title_font = ImageFont.truetype(path, 96)
            break
        except OSError:
            continue
    for path in font_paths_regular:
        try:
            small_font = ImageFont.truetype(path, 36)
            break
        except OSError:
            continue
    if title_font is None:
        title_font = ImageFont.load_default()
    if small_font is None:
        small_font = ImageFont.load_default()

    # Draw title
    title_text = "PicPop"
    title_bbox = draw.textbbox((0, 0), title_text, font=title_font)
    title_width = title_bbox[2] - title_bbox[0]
    title_x = (strip_width - title_width) // 2
    title_y = outer_padding + 24
    draw.text((title_x, title_y), title_text, fill=(139, 92, 246), font=title_font)

    # Sparkles
    sparkle_color = (251, 191, 36)
    sparkle_positions = [
        (title_x - 48, title_y + 20),
        (title_x + title_width + 32, title_y + 16),
    ]
    for sx, sy in sparkle_positions:
        draw.polygon([(sx, sy-12), (sx+3, sy-3), (sx+12, sy), (sx+3, sy+3),
                      (sx, sy+12), (sx-3, sy+3), (sx-12, sy), (sx-3, sy-3)],
                     fill=sparkle_color)

    # Paste photos
    y_offset = outer_padding + header_height
    for img in images:
        bordered_width = img.width + (photo_border * 2)
        bordered_height = img.height + (photo_border * 2)
        bordered_img = Image.new("RGB", (bordered_width, bordered_height), (255, 255, 255))
        bordered_img.paste(img, (photo_border, photo_border))

        mask = Image.new("L", (bordered_width, bordered_height), 0)
        mask_draw = ImageDraw.Draw(mask)
        mask_draw.rounded_rectangle(
            [(0, 0), (bordered_width - 1, bordered_height - 1)],
            radius=corner_radius + photo_border,
            fill=255,
        )

        # Shadow
        shadow_offset = 6
        shadow_color = (200, 195, 210)
        shadow_x = outer_padding + shadow_offset
        shadow_y = y_offset + shadow_offset
        draw.rounded_rectangle(
            [(shadow_x, shadow_y),
             (shadow_x + bordered_width - 1, shadow_y + bordered_height - 1)],
            radius=corner_radius + photo_border,
            fill=shadow_color,
        )

        photo_x = outer_padding
        strip.paste(bordered_img, (photo_x, y_offset), mask)
        y_offset += bordered_height + inner_padding

    # Footer date
    date_text = datetime.now().strftime("%b %d, %Y")
    date_bbox = draw.textbbox((0, 0), date_text, font=small_font)
    date_width = date_bbox[2] - date_bbox[0]
    date_x = (strip_width - date_width) // 2
    date_y = total_height - outer_padding - 54
    draw.text((date_x, date_y), date_text, fill=(120, 120, 140), font=small_font)

    # Hearts
    heart_color = (236, 72, 153)
    for hx in [date_x - 60, date_x + date_width + 36]:
        hy = date_y + 12
        draw.polygon([(hx, hy+6), (hx+9, hy-3), (hx+18, hy+6), (hx+9, hy+15)], fill=heart_color)
        draw.ellipse([(hx, hy-3), (hx+9, hy+6)], fill=heart_color)
        draw.ellipse([(hx+9, hy-3), (hx+18, hy+6)], fill=heart_color)

    # Save
    output = io.BytesIO()
    strip.save(output, format="JPEG", quality=92, optimize=True)
    return output.getvalue()


def get_storage_service() -> StorageService:
    """Get configured storage service."""
    return LocalStorageService(
        base_dir=settings.photos_dir,
        public_url=settings.public_url,
    )
