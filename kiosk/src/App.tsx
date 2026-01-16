import { useState, useEffect, useCallback, useRef } from 'react';
import { WelcomeScreen } from './components/WelcomeScreen';
import { SessionScreen } from './components/SessionScreen';
import { CountdownOverlay } from './components/CountdownOverlay';

const API_BASE = 'http://localhost:8000';
const WS_BASE = 'ws://localhost:8000';

type KioskState = 'welcome' | 'session' | 'capturing';

interface SessionData {
  id: string;
  phoneCount: number;
}

interface Photo {
  id: string;
  thumbnailUrl: string;
  webUrl: string;
}

export default function App() {
  const [state, setState] = useState<KioskState>('welcome');
  const [session, setSession] = useState<SessionData | null>(null);
  const [countdown, setCountdown] = useState<number | null>(null);
  const [photos, setPhotos] = useState<Photo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isStarting, setIsStarting] = useState(false);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // WiFi QR URL (doesn't require a session)
  const wifiQrUrl = `${API_BASE}/api/v1/sessions/wifi-qr?size=512`;

  // Camera preview URL (MJPEG stream) - 30fps default, use ?fps=60 for higher
  const previewUrl = `${API_BASE}/api/v1/camera/preview?fps=30`;

  // Connect to WebSocket for session
  const connectWebSocket = useCallback((sessionId: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.close();
    }

    const ws = new WebSocket(`${WS_BASE}/api/v1/ws/kiosk/${sessionId}`);

    ws.onopen = () => {
      console.log('Kiosk WebSocket connected');
      setError(null);
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        handleWebSocketMessage(message);
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e);
      }
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      // Reconnect if still in session
      if (session) {
        reconnectTimeoutRef.current = setTimeout(() => {
          connectWebSocket(sessionId);
        }, 2000);
      }
    };

    ws.onerror = (e) => {
      console.error('WebSocket error:', e);
    };

    wsRef.current = ws;
  }, [session]);

  // Handle incoming WebSocket messages
  const handleWebSocketMessage = (message: { type: string; data?: any }) => {
    switch (message.type) {
      case 'phone_connected':
        setSession(prev => prev ? { ...prev, phoneCount: prev.phoneCount + 1 } : null);
        break;

      case 'phone_disconnected':
        setSession(prev => prev ? {
          ...prev,
          phoneCount: Math.max(0, prev.phoneCount - 1)
        } : null);
        break;

      case 'countdown':
        setCountdown(message.data?.value ?? null);
        if (message.data?.value !== null) {
          setState('capturing');
        }
        break;

      case 'photo_ready':
        setPhotos(prev => [...prev, {
          id: message.data.id,
          thumbnailUrl: message.data.thumbnailUrl.startsWith('/')
            ? `${API_BASE}${message.data.thumbnailUrl}`
            : message.data.thumbnailUrl,
          webUrl: message.data.webUrl.startsWith('/')
            ? `${API_BASE}${message.data.webUrl}`
            : message.data.webUrl,
        }]);
        break;

      case 'capture_complete':
        setState('session');
        setCountdown(null);
        break;

      case 'capture_failed':
        setState('session');
        setCountdown(null);
        setError(message.data?.error || 'Capture failed');
        // Auto-clear error after 5 seconds
        setTimeout(() => setError(null), 5000);
        break;

      case 'session_ended':
        endSession();
        break;
    }
  };

  // Start new session
  const startSession = async () => {
    try {
      setError(null);
      setIsStarting(true);

      const response = await fetch(`${API_BASE}/api/v1/sessions`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error('Failed to create session');
      }

      const data = await response.json();

      setSession({
        id: data.id,
        phoneCount: 0,
      });

      setPhotos([]);
      setState('session');

      connectWebSocket(data.id);
    } catch (e) {
      console.error('Failed to create session:', e);
      setError('Failed to start session. Is the server running?');
    } finally {
      setIsStarting(false);
    }
  };

  // Start capture sequence
  const startCapture = async () => {
    if (!session) return;

    try {
      const response = await fetch(
        `${API_BASE}/api/v1/sessions/${session.id}/capture`,
        { method: 'POST' }
      );

      if (!response.ok) {
        const data = await response.json().catch(() => ({}));
        throw new Error(data.detail || 'Failed to start capture');
      }
    } catch (e) {
      console.error('Failed to start capture:', e);
      const errorMsg = e instanceof Error ? e.message : 'Failed to start capture';
      setError(errorMsg);
      setState('session');
      setCountdown(null);
      // Auto-clear error after 5 seconds
      setTimeout(() => setError(null), 5000);
    }
  };

  // End current session
  const endSession = async () => {
    if (session) {
      try {
        await fetch(`${API_BASE}/api/v1/sessions/${session.id}/end`, {
          method: 'POST',
        });
      } catch (e) {
        console.error('Failed to end session:', e);
      }
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    setSession(null);
    setPhotos([]);
    setCountdown(null);
    setState('welcome');
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
    };
  }, []);

  return (
    <div className="w-full h-full bg-bg-dark relative overflow-hidden">
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-primary-purple/20 via-primary-pink/10 to-accent-yellow/5" />

      {/* Main content */}
      <div className="relative z-10 w-full h-full">
        {state === 'welcome' && (
          <WelcomeScreen
            onStart={startSession}
            isLoading={isStarting}
            previewUrl={previewUrl}
          />
        )}

        {(state === 'session' || state === 'capturing') && session && (
          <SessionScreen
            wifiQrUrl={wifiQrUrl}
            sessionQrUrl={`${API_BASE}/api/v1/sessions/${session.id}/qr?size=512`}
            previewUrl={previewUrl}
            phoneCount={session.phoneCount}
            photos={photos}
            isCapturing={state === 'capturing'}
            onCapture={startCapture}
            onEnd={endSession}
          />
        )}
      </div>

      {/* Countdown overlay */}
      {countdown !== null && countdown > 0 && (
        <CountdownOverlay value={countdown} />
      )}

      {/* Error toast */}
      {error && (
        <div className="absolute bottom-8 left-1/2 -translate-x-1/2 bg-red-500/90 text-white px-6 py-3 rounded-full text-lg">
          {error}
        </div>
      )}
    </div>
  );
}
