"""QR code generation service."""

import io
from functools import lru_cache

import qrcode
from qrcode.image.styledpil import StyledPilImage
from qrcode.image.styles.moduledrawers import RoundedModuleDrawer


@lru_cache(maxsize=100)
def generate_qr_code(url: str, size: int = 256) -> bytes:
    """
    Generate a QR code image for the given URL.

    Args:
        url: The URL to encode
        size: The size of the QR code in pixels

    Returns:
        PNG image bytes
    """
    qr = qrcode.QRCode(
        version=1,
        error_correction=qrcode.constants.ERROR_CORRECT_M,
        box_size=10,
        border=2,
    )
    qr.add_data(url)
    qr.make(fit=True)

    img = qr.make_image(
        image_factory=StyledPilImage,
        module_drawer=RoundedModuleDrawer(),
    )

    # Resize to requested size
    img = img.resize((size, size))

    # Convert to bytes
    output = io.BytesIO()
    img.save(output, format="PNG")
    return output.getvalue()


def generate_wifi_qr_code(ssid: str, password: str, size: int = 256) -> bytes:
    """
    Generate a QR code for WiFi connection.

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
        version=1,
        error_correction=qrcode.constants.ERROR_CORRECT_M,
        box_size=10,
        border=2,
    )
    qr.add_data(wifi_string)
    qr.make(fit=True)

    img = qr.make_image(
        image_factory=StyledPilImage,
        module_drawer=RoundedModuleDrawer(),
    )

    # Resize to requested size
    img = img.resize((size, size))

    # Convert to bytes
    output = io.BytesIO()
    img.save(output, format="PNG")
    return output.getvalue()
