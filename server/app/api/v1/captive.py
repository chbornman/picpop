"""Captive portal endpoints for automatic phone connection."""

from fastapi import APIRouter, Request
from fastapi.responses import HTMLResponse, PlainTextResponse, RedirectResponse

from app.core.config import settings

router = APIRouter()

# Captive portal landing page HTML
PORTAL_HTML = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
    <title>PicPop Photo Booth</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .container {
            background: white;
            border-radius: 24px;
            padding: 40px 30px;
            text-align: center;
            max-width: 360px;
            width: 100%;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
        }
        .logo {
            font-size: 48px;
            margin-bottom: 10px;
        }
        h1 {
            color: #667eea;
            font-size: 32px;
            margin-bottom: 10px;
        }
        p {
            color: #666;
            margin-bottom: 30px;
            font-size: 16px;
            line-height: 1.5;
        }
        .button {
            display: block;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            text-decoration: none;
            padding: 18px 40px;
            border-radius: 50px;
            font-size: 18px;
            font-weight: 600;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        .button:active {
            transform: scale(0.98);
        }
        .hint {
            margin-top: 20px;
            font-size: 13px;
            color: #999;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">📸</div>
        <h1>PicPop</h1>
        <p>Welcome to the photo booth!<br>Tap below to view your photos.</p>
        <a href="SESSION_URL" class="button">Open Photo Booth</a>
        <p class="hint">Photos will appear here after capture</p>
    </div>
</body>
</html>
"""


@router.get("/generate_204")
async def android_captive_check():
    """
    Android captive portal detection.

    Android checks this URL and expects a 204 response if online.
    We return a redirect to trigger the captive portal UI.
    """
    return RedirectResponse(url="/portal", status_code=302)


@router.get("/hotspot-detect.html")
async def apple_captive_check():
    """
    Apple/iOS captive portal detection.

    iOS checks this URL and expects "<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>"
    if online. We return the portal page to trigger the captive portal UI.
    """
    return RedirectResponse(url="/portal", status_code=302)


@router.get("/library/test/success.html")
async def apple_captive_check_alt():
    """Alternative Apple captive portal check URL."""
    return RedirectResponse(url="/portal", status_code=302)


@router.get("/connecttest.txt")
async def windows_captive_check():
    """
    Windows captive portal detection.

    Windows checks this URL and expects "Microsoft Connect Test".
    We return a redirect to trigger the captive portal.
    """
    return RedirectResponse(url="/portal", status_code=302)


@router.get("/ncsi.txt")
async def windows_ncsi_check():
    """Windows NCSI (Network Connectivity Status Indicator) check."""
    return RedirectResponse(url="/portal", status_code=302)


@router.get("/redirect")
async def generic_redirect():
    """Generic redirect endpoint for captive portal."""
    return RedirectResponse(url="/portal", status_code=302)


@router.get("/portal", response_class=HTMLResponse)
async def captive_portal_page(request: Request):
    """
    Captive portal landing page.

    This page is shown when users connect to the WiFi.
    It provides a button to open the full photo booth web app.
    """
    # Try to get session ID from query params or use latest
    session_id = request.query_params.get("session")

    if session_id:
        session_url = f"{settings.public_url}/session/{session_id}"
    else:
        # Default to root which will show instructions
        session_url = settings.public_url

    html = PORTAL_HTML.replace("SESSION_URL", session_url)
    return HTMLResponse(content=html)


@router.get("/success.txt")
async def success_check():
    """
    Success check endpoint.

    Some systems check for this to verify internet connectivity.
    We return "success" to prevent constant redirects after initial portal.
    """
    return PlainTextResponse("success")


# Catch-all for unknown captive portal checks
@router.get("/favicon.ico")
async def favicon():
    """Return empty favicon to prevent 404s."""
    return PlainTextResponse("", status_code=204)
