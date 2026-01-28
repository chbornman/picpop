"""PicPop Photo Booth Server - FastAPI Application"""

from contextlib import asynccontextmanager
from pathlib import Path
from typing import AsyncGenerator

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse

from app.api.v1 import router as api_router
from app.api.v1.captive import router as captive_router
from app.core.config import settings
from app.core.logging import setup_logging
from app.db.session import init_db


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Application lifespan - startup and shutdown."""
    # Startup
    setup_logging()
    await init_db()

    yield

    # Shutdown - release camera
    from app.services.camera import get_camera
    import logging
    logger = logging.getLogger(__name__)
    try:
        camera = get_camera(settings.camera_backend)
        if camera.is_connected():
            logger.info("Shutting down: disconnecting camera...")
            await camera.disconnect()
            logger.info("Camera disconnected on shutdown")
    except Exception as e:
        logger.warning(f"Error disconnecting camera on shutdown: {e}")


app = FastAPI(
    title="PicPop Photo Booth API",
    description="Offline photo booth server with WebSocket support",
    version="0.1.0",
    lifespan=lifespan,
    response_model_by_alias=True,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# API routes
app.include_router(api_router, prefix="/api/v1")

# Captive portal routes (at root level for device compatibility)
app.include_router(captive_router)

# Static file serving for photos
app.mount("/photos", StaticFiles(directory=settings.photos_dir), name="photos")

# Check if mobile app build exists and serve it
mobile_dist = Path(__file__).parent.parent / "frontend" / "dist"
if mobile_dist.exists():
    # Serve mobile app static files
    app.mount("/assets", StaticFiles(directory=mobile_dist / "assets"), name="mobile-assets")

    @app.get("/session/{session_id}")
    async def serve_mobile_app(session_id: str):
        """Serve mobile app for session view."""
        return FileResponse(mobile_dist / "index.html")

    @app.get("/")
    async def serve_mobile_root():
        """Serve mobile app root."""
        return FileResponse(mobile_dist / "index.html")


@app.get("/health")
async def health_check() -> dict[str, str]:
    """Health check endpoint."""
    return {"status": "healthy", "version": "0.1.0"}


@app.get("/api/health")
async def api_health_check() -> dict[str, str]:
    """API health check endpoint."""
    return {"status": "healthy", "version": "0.1.0"}
