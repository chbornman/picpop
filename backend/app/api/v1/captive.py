"""Captive portal endpoints for automatic phone connection."""

from fastapi import APIRouter, Depends, Request
from fastapi.responses import HTMLResponse, PlainTextResponse, RedirectResponse
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.core.config import settings
from app.db.session import get_db
from app.models.session import Session, SessionStatus

router = APIRouter()

# Captive portal landing page HTML - with active session
PORTAL_HTML_ACTIVE = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
    <title>PicPop Photo Booth</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #8B5CF6 0%, #EC4899 100%);
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
        .logo { font-size: 48px; margin-bottom: 10px; }
        h1 { color: #8B5CF6; font-size: 32px; margin-bottom: 10px; }
        p { color: #666; margin-bottom: 20px; font-size: 16px; line-height: 1.5; }
        .button {
            display: block;
            background: linear-gradient(135deg, #8B5CF6 0%, #EC4899 100%);
            color: white;
            text-decoration: none;
            padding: 18px 40px;
            border-radius: 50px;
            font-size: 18px;
            font-weight: 600;
            transition: transform 0.2s;
            border: none;
            cursor: pointer;
            width: 100%;
        }
        .button:active { transform: scale(0.98); }
        .step {
            background: #f8f8f8;
            border-radius: 12px;
            padding: 15px;
            margin-top: 20px;
            text-align: left;
        }
        .step-num {
            display: inline-block;
            width: 24px;
            height: 24px;
            background: #8B5CF6;
            color: white;
            border-radius: 50%;
            text-align: center;
            line-height: 24px;
            font-size: 14px;
            margin-right: 10px;
        }
        .step-text { color: #333; font-size: 14px; }
        .url-box {
            background: #eee;
            padding: 10px;
            border-radius: 8px;
            margin-top: 10px;
            font-family: monospace;
            font-size: 14px;
            word-break: break-all;
            color: #8B5CF6;
        }
        .hint { margin-top: 15px; font-size: 12px; color: #999; }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">📸</div>
        <h1>PicPop</h1>
        <p>You're connected to the photo booth!</p>

        <button class="button" onclick="openSession()">Open Photo Booth</button>

        <div class="step">
            <p class="step-text"><span class="step-num">1</span>Tap the button above</p>
            <p class="step-text" style="margin-top:10px;"><span class="step-num">2</span>If it doesn't open, close this popup and open Safari/Chrome:</p>
            <div class="url-box">SESSION_URL</div>
        </div>

        <p class="hint">This popup may close automatically</p>
    </div>

    <script>
        function openSession() {
            var sessionUrl = "SESSION_URL";
            // Navigate to success endpoint with redirect - this tells iOS "portal complete"
            // and then redirects to the actual session
            window.location.href = "/captive-success?redirect=" + encodeURIComponent(sessionUrl);
        }
    </script>
</body>
</html>
"""

# Captive portal landing page HTML - no active session
PORTAL_HTML_WAITING = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
    <meta http-equiv="refresh" content="5">
    <title>PicPop Photo Booth</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #8B5CF6 0%, #EC4899 100%);
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
        .logo { font-size: 48px; margin-bottom: 10px; }
        h1 { color: #8B5CF6; font-size: 32px; margin-bottom: 10px; }
        p { color: #666; margin-bottom: 20px; font-size: 16px; line-height: 1.5; }
        .spinner {
            width: 40px;
            height: 40px;
            border: 4px solid #f3f3f3;
            border-top: 4px solid #8B5CF6;
            border-radius: 50%;
            animation: spin 1s linear infinite;
            margin: 20px auto;
        }
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        .hint { font-size: 13px; color: #999; }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">📸</div>
        <h1>PicPop</h1>
        <p>Waiting for the photo booth session to start...</p>
        <div class="spinner"></div>
        <p class="hint">This page will refresh automatically</p>
    </div>
</body>
</html>
"""


@router.get("/generate_204")
async def android_captive_check():
    """
    Android captive portal detection.
    Return 204 = "internet works" = no popup, stay connected.
    """
    return PlainTextResponse("", status_code=204)


@router.get("/hotspot-detect.html")
async def apple_captive_check():
    """
    Apple/iOS captive portal detection.
    Return Success = no popup, stay connected.
    """
    return HTMLResponse("<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>")


@router.get("/library/test/success.html")
async def apple_captive_check_alt():
    """Alternative Apple captive portal check URL."""
    return HTMLResponse("<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>")


@router.get("/connecttest.txt")
async def windows_captive_check():
    """
    Windows captive portal detection.
    Return expected response = no popup, stay connected.
    """
    return PlainTextResponse("Microsoft Connect Test")


@router.get("/ncsi.txt")
async def windows_ncsi_check():
    """Windows NCSI check."""
    return PlainTextResponse("Microsoft NCSI")


@router.get("/captive-success")
async def captive_success(redirect: str = None):
    """
    Success endpoint that tells iOS/Android the captive portal is complete.
    If redirect is provided, attempts to open that URL after closing portal.
    """
    if redirect:
        # Return success page that also tries to redirect
        return HTMLResponse(f"""<HTML>
<HEAD>
<TITLE>Success</TITLE>
<meta http-equiv="refresh" content="0;url={redirect}">
</HEAD>
<BODY>
Success
<script>
    // Try multiple methods to open the session
    window.location.href = "{redirect}";
    setTimeout(function() {{ window.open("{redirect}", "_blank"); }}, 100);
</script>
</BODY>
</HTML>""")
    return HTMLResponse("<HTML><HEAD><TITLE>Success</TITLE></HEAD><BODY>Success</BODY></HTML>")


@router.get("/redirect")
async def generic_redirect():
    """Generic redirect endpoint for captive portal."""
    return RedirectResponse(url="/portal", status_code=302)


@router.get("/portal", response_class=HTMLResponse)
async def captive_portal_page(
    request: Request,
    db: AsyncSession = Depends(get_db),
):
    """
    Captive portal landing page.

    This page is shown when users connect to the WiFi.
    It checks for an active session and either shows a join button
    or a waiting message.
    """
    # Look for the current active session
    result = await db.execute(
        select(Session)
        .where(Session.status.in_([
            SessionStatus.ACTIVE.value,
            SessionStatus.CAPTURING.value,
            SessionStatus.COUNTDOWN.value,
        ]))
        .order_by(Session.created_at.desc())
        .limit(1)
    )
    active_session = result.scalar_one_or_none()

    if active_session:
        session_url = f"{settings.public_url}/session/{active_session.id}"
        # Replace all occurrences of SESSION_URL
        html = PORTAL_HTML_ACTIVE.replace("SESSION_URL", session_url)
    else:
        html = PORTAL_HTML_WAITING

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
