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
python -m venv .venv
source .venv/bin/activate
pip install -e .
uvicorn app.main:app --reload
```
