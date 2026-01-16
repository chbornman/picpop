"""Photo schemas."""

from datetime import datetime

from app.schemas.base import CamelModel


class PhotoResponse(CamelModel):
    """Schema for photo response."""

    id: str
    sessionId: str
    sequence: int
    capturedAt: datetime
    webUrl: str
    thumbnailUrl: str
