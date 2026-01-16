"""Photo session model."""

from datetime import datetime
from enum import Enum
from typing import TYPE_CHECKING
from uuid import uuid4

from sqlalchemy import DateTime, String, func
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.base import Base

if TYPE_CHECKING:
    from app.models.photo import Photo


class SessionStatus(str, Enum):
    """Session status enumeration."""

    ACTIVE = "active"          # Ready, waiting for capture command
    COUNTDOWN = "countdown"    # Countdown in progress
    CAPTURING = "capturing"    # Camera is actively capturing
    COMPLETED = "completed"    # Done, photos available for download
    EXPIRED = "expired"        # Past expiry time


class Session(Base):
    """Photo session model - represents a single booth session."""

    __tablename__ = "sessions"

    id: Mapped[str] = mapped_column(
        String(36),
        primary_key=True,
        default=lambda: str(uuid4()),
    )
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        server_default=func.now(),
    )
    expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))
    status: Mapped[str] = mapped_column(
        String(20),
        default=SessionStatus.ACTIVE.value,
    )
    upload_token: Mapped[str] = mapped_column(String(64))
    photo_count: Mapped[int] = mapped_column(default=0)

    # Track connected clients
    kiosk_connected: Mapped[bool] = mapped_column(default=False)
    phone_connected: Mapped[bool] = mapped_column(default=False)

    # Relationships
    photos: Mapped[list["Photo"]] = relationship(
        "Photo",
        back_populates="session",
        cascade="all, delete-orphan",
    )
