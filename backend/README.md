# PicPop Server

FastAPI backend for the PicPop photo booth system.

## Features

- Session management
- Camera control via gphoto2
- WebSocket communication with kiosk and phones
- QR code generation
- Captive portal endpoints
- Photo storage and strip generation

## Development

```bash
uv sync
uv run uvicorn app.main:app --reload
```
