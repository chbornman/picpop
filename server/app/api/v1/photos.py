"""Photo endpoints for viewing and downloading."""

from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.responses import FileResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import settings
from app.db.session import get_db
from app.models.session import Session
from app.models.photo import Photo
from app.schemas import PhotoResponse
from app.services import get_storage_service

router = APIRouter()


@router.get("/{session_id}/photos", response_model=list[PhotoResponse])
async def list_session_photos(
    session_id: str,
    db: AsyncSession = Depends(get_db),
) -> list[PhotoResponse]:
    """List all photos for a session."""
    result = await db.execute(select(Session).where(Session.id == session_id))
    session = result.scalar_one_or_none()

    if not session:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Session not found",
        )

    photos_result = await db.execute(
        select(Photo)
        .where(Photo.session_id == session_id)
        .order_by(Photo.sequence)
    )
    photos = photos_result.scalars().all()

    storage = get_storage_service()

    return [
        PhotoResponse(
            id=photo.id,
            sessionId=photo.session_id,
            sequence=photo.sequence,
            capturedAt=photo.captured_at,
            webUrl=storage.get_photo_url(photo.web_path),
            thumbnailUrl=storage.get_photo_url(photo.thumbnail_path),
        )
        for photo in photos
    ]


@router.get("/{session_id}/photos/{photo_id}/download")
async def download_photo(
    session_id: str,
    photo_id: str,
    db: AsyncSession = Depends(get_db),
) -> FileResponse:
    """Download full-resolution photo."""
    result = await db.execute(
        select(Photo)
        .where(Photo.session_id == session_id)
        .where(Photo.id == photo_id)
    )
    photo = result.scalar_one_or_none()

    if not photo:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Photo not found",
        )

    file_path = settings.photos_dir / photo.web_path

    if not file_path.exists():
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Photo file not found",
        )

    filename = f"picpop_{session_id[:8]}_{photo.sequence:02d}.jpg"

    return FileResponse(
        path=file_path,
        filename=filename,
        media_type="image/jpeg",
    )
