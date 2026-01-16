"""Application configuration using Pydantic Settings."""

from pathlib import Path
from typing import Literal

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Application settings loaded from environment variables."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
    )

    # Server
    host: str = "0.0.0.0"
    port: int = 8000
    debug: bool = False

    # URLs - for offline mode on Radxa hotspot
    public_url: str = "http://192.168.4.1:8000"

    # WiFi hotspot settings (for QR code generation)
    wifi_ssid: str = "PicPop"
    wifi_password: str = "photobooth"

    # CORS - allow all origins for local network access
    cors_origins: list[str] = ["*"]

    # Database
    database_url: str = "sqlite+aiosqlite:///./picpop.db"

    # Storage (local only for offline)
    photos_dir: Path = Path("./photos")

    # Session settings
    session_expiry_minutes: int = 60

    # Photo settings
    max_photos_per_session: int = 20
    save_raw_images: bool = False
    thumbnail_max_width: int = 400

    # Camera
    camera_backend: Literal["gphoto2", "mock"] = "gphoto2"
    photos_per_capture: int = 3
    capture_delay_seconds: float = 1.5
    countdown_seconds: int = 3

    def __init__(self, **kwargs: object) -> None:
        super().__init__(**kwargs)
        # Ensure photos directory exists
        self.photos_dir.mkdir(parents=True, exist_ok=True)


settings = Settings()
