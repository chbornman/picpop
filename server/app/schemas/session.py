"""Session schemas."""

from datetime import datetime

from app.schemas.base import CamelModel
from app.schemas.photo import PhotoResponse


class SessionResponse(CamelModel):
    """Schema for session response."""

    id: str
    createdAt: datetime
    expiresAt: datetime
    status: str
    photoCount: int
    kioskConnected: bool = False
    phoneConnected: bool = False


class CreateSessionResponse(CamelModel):
    """Schema for create session response."""

    id: str
    uploadToken: str
    galleryUrl: str
    qrCodeUrl: str
    wifiQrUrl: str


class SessionGalleryResponse(CamelModel):
    """Schema for session gallery response."""

    session: SessionResponse
    photos: list[PhotoResponse]
    qrCodeUrl: str
    stripUrl: str | None = None
