import { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { WaitingScreen } from './components/WaitingScreen';
import { CountdownScreen } from './components/CountdownScreen';
import { ProcessingScreen } from './components/ProcessingScreen';
import { PhotosScreen } from './components/PhotosScreen';
import { useWebSocket } from './hooks/useWebSocket';
import type { Photo } from './types';

export default function App() {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [photos, setPhotos] = useState<Photo[]>([]);
  const [countdown, setCountdown] = useState<number | null>(null);
  const [photoNumber, setPhotoNumber] = useState<number>(1);
  const [totalPhotos, setTotalPhotos] = useState<number>(1);
  const [isCapturing, setIsCapturing] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [kioskConnected, setKioskConnected] = useState(false);
  const [stripUrl, setStripUrl] = useState<string | null>(null);

  // Extract session ID from URL
  useEffect(() => {
    const path = window.location.pathname;
    const match = path.match(/\/session\/([a-f0-9-]+)/i);
    if (match) {
      setSessionId(match[1]);
    }
  }, []);

  // Handle WebSocket messages
  const handleMessage = (message: { type: string; data?: Record<string, unknown> }) => {
    switch (message.type) {
      case 'session_state':
        if (message.data) {
          setKioskConnected(message.data.kioskConnected as boolean);
          if (Array.isArray(message.data.photos)) {
            setPhotos(message.data.photos as Photo[]);
          }
        }
        break;

      case 'kiosk_connected':
        setKioskConnected(true);
        break;

      case 'countdown':
        if (message.data) {
          setCountdown(message.data.value as number);
          setPhotoNumber((message.data.photoNumber as number) || 1);
          setTotalPhotos((message.data.totalPhotos as number) || 1);
          setIsCapturing(true);
          setIsProcessing(false);
        }
        break;

      case 'capture_start':
        // Don't clear countdown - let it transition naturally to next photo
        break;

      case 'processing':
        // All photos captured, now processing
        setCountdown(null);
        setIsCapturing(true);
        setIsProcessing(true);
        break;

      case 'photo_ready':
        if (message.data) {
          const newPhoto: Photo = {
            id: message.data.id as string,
            sessionId: message.data.sessionId as string,
            sequence: message.data.sequence as number,
            webUrl: message.data.webUrl as string,
            thumbnailUrl: message.data.thumbnailUrl as string,
          };
          setPhotos((prev) => [...prev, newPhoto]);
        }
        break;

      case 'capture_complete':
        setIsCapturing(false);
        setIsProcessing(false);
        setCountdown(null);
        if (message.data?.stripUrl) {
          setStripUrl(message.data.stripUrl as string);
        }
        break;

      case 'session_ended':
        // Session ended - could redirect or show message
        setPhotos([]);
        setIsCapturing(false);
        setCountdown(null);
        setKioskConnected(false);
        break;
    }
  };

  const { isConnected } = useWebSocket(sessionId, handleMessage);

  // No session ID in URL
  if (!sessionId) {
    return (
      <div className="h-full flex items-center justify-center p-6">
        <div className="text-center text-white">
          <div className="text-6xl mb-4">📸</div>
          <h1 className="text-2xl font-bold mb-2">PicPop</h1>
          <p className="text-white/70">Scan the QR code on the photo booth to get started!</p>
        </div>
      </div>
    );
  }

  // Determine which screen to show
  const showCountdown = countdown !== null && countdown > 0;
  const showProcessing = isProcessing && !showCountdown;
  const showPhotos = photos.length > 0 && !showCountdown && !showProcessing;
  const showWaiting = !showCountdown && !showProcessing && !showPhotos;

  return (
    <div className="h-full relative overflow-hidden">
      <AnimatePresence mode="wait">
        {showCountdown && (
          <motion.div
            key="countdown"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="h-full"
          >
            <CountdownScreen
              value={countdown}
              photoNumber={photoNumber}
              totalPhotos={totalPhotos}
            />
          </motion.div>
        )}

        {showProcessing && (
          <motion.div
            key="processing"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="h-full"
          >
            <ProcessingScreen />
          </motion.div>
        )}

        {showWaiting && (
          <motion.div
            key="waiting"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="h-full"
          >
            <WaitingScreen
              isConnected={isConnected}
              kioskConnected={kioskConnected}
              isCapturing={isCapturing}
            />
          </motion.div>
        )}

        {showPhotos && (
          <motion.div
            key="photos"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="h-full"
          >
            <PhotosScreen photos={photos} stripUrl={stripUrl} isCapturing={isCapturing} />
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
