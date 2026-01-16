import { useState, useEffect, useCallback, useRef } from 'react';
import { IdleScreen } from './components/IdleScreen';
import { SessionScreen } from './components/SessionScreen';
import { CountdownOverlay } from './components/CountdownOverlay';
import { PhotoStripPreview } from './components/PhotoStripPreview';

const API_BASE = 'http://localhost:8000';
const WS_BASE = 'ws://localhost:8000';

type KioskState = 'idle' | 'session' | 'capturing' | 'reviewing';

interface SessionData {
  id: string;
  qrCodeUrl: string;
  wifiQrUrl: string;
  phoneCount: number;
}

interface Photo {
  id: string;
  sequence: number;
  url: string;
  thumbnailUrl: string;
}

export default function App() {
  const [state, setState] = useState<KioskState>('idle');
  const [session, setSession] = useState<SessionData | null>(null);
  const [countdown, setCountdown] = useState<number | null>(null);
  const [photos, setPhotos] = useState<Photo[]>([]);
  const [stripUrl, setStripUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

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
          id: message.data.photoId,
          sequence: message.data.sequence,
          url: message.data.webUrl,
          thumbnailUrl: message.data.thumbnailUrl,
        }]);
        break;

      case 'capture_complete':
        setStripUrl(message.data?.stripUrl ?? null);
        setState('reviewing');
        setCountdown(null);
        break;

      case 'session_ended':
        endSession();
        break;
    }
  };

  // Create new session
  const createSession = async () => {
    try {
      setError(null);
      const response = await fetch(`${API_BASE}/api/v1/sessions`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error('Failed to create session');
      }

      const data = await response.json();

      setSession({
        id: data.id,
        qrCodeUrl: `${API_BASE}${data.qrCodeUrl}`,
        wifiQrUrl: `${API_BASE}/api/v1/sessions/${data.id}/wifi-qr`,
        phoneCount: 0,
      });

      setPhotos([]);
      setStripUrl(null);
      setState('session');

      connectWebSocket(data.id);
    } catch (e) {
      console.error('Failed to create session:', e);
      setError('Failed to create session. Is the server running?');
    }
  };

  // Start capture sequence
  const startCapture = async () => {
    if (!session) return;

    try {
      setPhotos([]);
      const response = await fetch(
        `${API_BASE}/api/v1/sessions/${session.id}/capture`,
        { method: 'POST' }
      );

      if (!response.ok) {
        throw new Error('Failed to start capture');
      }
    } catch (e) {
      console.error('Failed to start capture:', e);
      setError('Failed to start capture');
      setState('session');
    }
  };

  // Take more photos (reset to session state)
  const takeMore = () => {
    setPhotos([]);
    setStripUrl(null);
    setState('session');
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
    setStripUrl(null);
    setCountdown(null);
    setState('idle');
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
        {state === 'idle' && (
          <IdleScreen onStart={createSession} error={error} />
        )}

        {(state === 'session' || state === 'capturing') && session && (
          <SessionScreen
            session={session}
            onCapture={startCapture}
            onEnd={endSession}
            isCapturing={state === 'capturing'}
          />
        )}

        {state === 'reviewing' && session && (
          <PhotoStripPreview
            photos={photos}
            stripUrl={stripUrl}
            onTakeMore={takeMore}
            onEnd={endSession}
          />
        )}
      </div>

      {/* Countdown overlay */}
      {countdown !== null && countdown > 0 && (
        <CountdownOverlay value={countdown} />
      )}

      {/* Error toast */}
      {error && state !== 'idle' && (
        <div className="absolute bottom-8 left-1/2 -translate-x-1/2 bg-red-500/90 text-white px-6 py-3 rounded-full text-lg">
          {error}
        </div>
      )}
    </div>
  );
}
