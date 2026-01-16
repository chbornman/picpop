"""API v1 router."""

from fastapi import APIRouter

from app.api.v1 import sessions, photos, websocket, captive, preview

router = APIRouter()

router.include_router(sessions.router, prefix="/sessions", tags=["sessions"])
router.include_router(photos.router, prefix="/sessions", tags=["photos"])
router.include_router(websocket.router, prefix="/ws", tags=["websocket"])
router.include_router(preview.router, prefix="/camera", tags=["camera"])
router.include_router(captive.router, tags=["captive-portal"])
