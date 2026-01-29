"""QR code generation service."""

import io
from functools import lru_cache

import qrcode
from PIL import Image, ImageDraw, ImageFont
from qrcode.image.styledpil import StyledPilImage
from qrcode.image.styles.moduledrawers import RoundedModuleDrawer


# Use a fixed QR version for consistent visual appearance
# Version 6 supports up to 134 alphanumeric chars with H error correction
# This ensures both WiFi and URL QR codes look the same
QR_VERSION = 6


def _add_label_to_qr(img: Image.Image, label: str, size: int) -> Image.Image:
    """
    Add a text label in the center of a QR code.

    Args:
        img: The QR code image
        label: The text to add (e.g., "WIFI", "PHOTOS")
        size: The final image size

    Returns:
        Image with label added
    """
    # Convert to RGBA for transparency support
    img = img.convert("RGBA")

    # Calculate label box size (about 25% of QR code)
    box_size = int(size * 0.28)
    box_x = (size - box_size) // 2
    box_y = (size - box_size) // 2

    # Create a white rounded rectangle for the label background
    draw = ImageDraw.Draw(img)
    corner_radius = box_size // 6
    draw.rounded_rectangle(
        [box_x, box_y, box_x + box_size, box_y + box_size],
        radius=corner_radius,
        fill="white",
    )

    # Calculate font size (scale with box size)
    font_size = box_size // 3
    try:
        # Try to use a bold font if available
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", font_size)
    except (OSError, IOError):
        try:
            font = ImageFont.truetype("/usr/share/fonts/TTF/DejaVuSans-Bold.ttf", font_size)
        except (OSError, IOError):
            # Fall back to default font
            font = ImageFont.load_default()

    # Get text bounding box for centering
    bbox = draw.textbbox((0, 0), label, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]

    # Center text in the box
    text_x = box_x + (box_size - text_width) // 2
    text_y = box_y + (box_size - text_height) // 2 - bbox[1]

    # Draw the text in dark color
    draw.text((text_x, text_y), label, fill="#1E293B", font=font)

    return img


@lru_cache(maxsize=100)
def generate_qr_code(url: str, size: int = 256, label: str = "PHOTOS") -> bytes:
    """
    Generate a QR code image for the given URL with an embedded label.

    Args:
        url: The URL to encode
        size: The size of the QR code in pixels
        label: The text label to embed in the center (default: "PHOTOS")

    Returns:
        PNG image bytes
    """
    qr = qrcode.QRCode(
        version=QR_VERSION,
        # Use HIGH error correction to allow for the center label
        error_correction=qrcode.constants.ERROR_CORRECT_H,
        box_size=10,
        border=2,
    )
    qr.add_data(url)
    qr.make(fit=False)  # Don't auto-fit, use fixed version

    img = qr.make_image(
        image_factory=StyledPilImage,
        module_drawer=RoundedModuleDrawer(),
    )

    # Resize to requested size
    img = img.resize((size, size))

    # Add label to center
    img = _add_label_to_qr(img, label, size)

    # Convert to bytes
    output = io.BytesIO()
    img.save(output, format="PNG")
    return output.getvalue()


def generate_wifi_qr_code(ssid: str, password: str, size: int = 256) -> bytes:
    """
    Generate a QR code for WiFi connection with "WIFI" label.

    Uses the standard WIFI: format that iOS and Android can scan
    to automatically connect to a network.

    Args:
        ssid: WiFi network name
        password: WiFi password
        size: The size of the QR code in pixels

    Returns:
        PNG image bytes
    """
    # WiFi QR code format: WIFI:T:WPA;S:<SSID>;P:<password>;;
    wifi_string = f"WIFI:T:WPA;S:{ssid};P:{password};;"

    qr = qrcode.QRCode(
        version=QR_VERSION,
        # Use HIGH error correction to allow for the center label
        error_correction=qrcode.constants.ERROR_CORRECT_H,
        box_size=10,
        border=2,
    )
    qr.add_data(wifi_string)
    qr.make(fit=False)  # Don't auto-fit, use fixed version

    img = qr.make_image(
        image_factory=StyledPilImage,
        module_drawer=RoundedModuleDrawer(),
    )

    # Resize to requested size
    img = img.resize((size, size))

    # Add "WIFI" label to center
    img = _add_label_to_qr(img, "WIFI", size)

    # Convert to bytes
    output = io.BytesIO()
    img.save(output, format="PNG")
    return output.getvalue()
