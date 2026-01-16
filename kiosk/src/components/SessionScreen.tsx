import { useState, useEffect, useCallback } from 'react';
import { Camera, Users, Wifi, X, VideoOff, ChevronLeft, ChevronRight, RefreshCw } from 'lucide-react';

interface Photo {
  id: string;
  thumbnailUrl: string;
  webUrl: string;
}

interface SessionScreenProps {
  wifiQrUrl: string;
  sessionQrUrl: string;
  previewUrl: string;
  phoneCount: number;
  photos: Photo[];
  isCapturing: boolean;
  onCapture: () => void;
  onEnd: () => void;
}

export function SessionScreen({
  wifiQrUrl,
  sessionQrUrl,
  previewUrl,
  phoneCount,
  photos,
  isCapturing,
  onCapture,
  onEnd,
}: SessionScreenProps) {
  const [previewError, setPreviewError] = useState(false);
  const [previewKey, setPreviewKey] = useState(0);
  const [viewingIndex, setViewingIndex] = useState<number | null>(null);

  // Retry preview connection
  const retryPreview = useCallback(() => {
    setPreviewError(false);
    setPreviewKey(k => k + 1);
  }, []);

  // Auto-retry preview every 5 seconds when in error state
  useEffect(() => {
    if (!previewError) return;

    const timer = setInterval(() => {
      retryPreview();
    }, 5000);

    return () => clearInterval(timer);
  }, [previewError, retryPreview]);

  const openLightbox = (index: number) => setViewingIndex(index);
  const closeLightbox = () => setViewingIndex(null);
  const prevPhoto = () => {
    if (viewingIndex !== null && viewingIndex > 0) {
      setViewingIndex(viewingIndex - 1);
    }
  };
  const nextPhoto = () => {
    if (viewingIndex !== null && viewingIndex < photos.length - 1) {
      setViewingIndex(viewingIndex + 1);
    }
  };

  return (
    <div className="w-full h-full flex flex-col">
      {/* Top bar */}
      <div className="flex items-center justify-between px-8 py-4">
        {/* Connected phones */}
        <div className="flex items-center gap-3 bg-bg-surface/50 px-6 py-3 rounded-full">
          <Users className="w-6 h-6 text-accent-yellow" />
          <span className="text-xl font-medium">
            {phoneCount} {phoneCount === 1 ? 'phone' : 'phones'} connected
          </span>
        </div>

        {/* End session button */}
        <button
          onClick={onEnd}
          className="flex items-center gap-2 px-6 py-3 rounded-full bg-white/10 hover:bg-red-500/20 hover:text-red-400 transition-colors"
        >
          <X className="w-5 h-5" />
          <span>End Session</span>
        </button>
      </div>

      {/* Main content */}
      <div className="flex-1 flex px-8 pb-4 gap-6 min-h-0">
        {/* Left side - Camera preview & capture */}
        <div className="flex-1 flex flex-col items-center justify-center">
          {/* Camera preview */}
          <div className="w-full max-w-2xl bg-bg-surface/50 rounded-3xl overflow-hidden mb-8 border-2 border-white/10 relative flex items-center justify-center min-h-[300px]">
            {!previewError ? (
              <img
                key={previewKey}
                src={`${previewUrl}${previewUrl.includes('?') ? '&' : '?'}_t=${previewKey}`}
                alt="Camera Preview"
                className="w-full h-auto object-contain"
                onError={() => setPreviewError(true)}
              />
            ) : (
              <div className="w-full py-16 flex items-center justify-center">
                <div className="text-center text-white/40">
                  <VideoOff className="w-16 h-16 mx-auto mb-4 opacity-50" />
                  <p className="text-lg mb-4">Camera not available</p>
                  <button
                    onClick={retryPreview}
                    className="flex items-center gap-2 mx-auto px-4 py-2 rounded-full bg-white/10 hover:bg-white/20 transition-colors text-white/70 hover:text-white"
                  >
                    <RefreshCw className="w-4 h-4" />
                    <span>Retry</span>
                  </button>
                  <p className="text-sm mt-3 text-white/30">Auto-retrying every 5s...</p>
                </div>
              </div>
            )}
            {/* Capturing overlay */}
            {isCapturing && (
              <div className="absolute inset-0 bg-black/50 flex items-center justify-center">
                <div className="text-white text-2xl font-bold animate-pulse">
                  Capturing...
                </div>
              </div>
            )}
          </div>

          {/* Capture button */}
          <button
            onClick={onCapture}
            disabled={isCapturing}
            className={`
              relative w-40 h-40 rounded-full
              flex items-center justify-center
              transition-all duration-300
              ${isCapturing
                ? 'bg-bg-surface/50 cursor-not-allowed'
                : 'bg-gradient-main hover:scale-105 active:scale-95'
              }
            `}
          >
            <span className={`absolute inset-0 rounded-full border-4 ${isCapturing ? 'border-white/20' : 'border-white/40'}`} />
            {!isCapturing && (
              <span className="absolute inset-0 rounded-full bg-white/20 pulse-ring" />
            )}
            <Camera className={`w-16 h-16 ${isCapturing ? 'text-white/40' : 'text-white'}`} />
          </button>

          <p className="mt-6 text-xl text-white/60">
            {isCapturing ? 'Taking photos...' : 'Tap to capture!'}
          </p>
        </div>

        {/* Right side - QR Codes */}
        <div className="w-80 flex flex-col items-center justify-center gap-4 py-4">
          {/* WiFi QR */}
          <div className="bg-bg-surface/30 rounded-2xl p-4 w-full">
            <div className="flex items-center gap-2 mb-3 justify-center">
              <Wifi className="w-5 h-5 text-accent-mint" />
              <h3 className="text-base font-medium">1. Connect to WiFi</h3>
            </div>
            <div className="bg-white p-3 rounded-xl mx-auto w-fit">
              <img
                src={wifiQrUrl}
                alt="WiFi QR Code"
                className="w-36 h-36"
              />
            </div>
            <p className="mt-2 text-xs text-white/40 text-center">
              Scan to join "PicPop" network
            </p>
          </div>

          {/* Session QR */}
          <div className="bg-bg-surface/30 rounded-2xl p-4 w-full">
            <div className="flex items-center gap-2 mb-3 justify-center">
              <Camera className="w-5 h-5 text-primary-pink" />
              <h3 className="text-base font-medium">2. Get Your Photos</h3>
            </div>
            <div className="bg-white p-3 rounded-xl mx-auto w-fit">
              <img
                src={sessionQrUrl}
                alt="Session QR Code"
                className="w-36 h-36"
              />
            </div>
            <p className="mt-2 text-xs text-white/40 text-center">
              Scan to view and download
            </p>
          </div>
        </div>
      </div>

      {/* Photo gallery */}
      {photos.length > 0 && (
        <div className="px-8 pb-6">
          <div className="bg-bg-surface/30 rounded-2xl p-4">
            <div className="flex gap-4 overflow-x-auto pb-2">
              {photos.map((photo, index) => (
                <button
                  key={photo.id}
                  onClick={() => openLightbox(index)}
                  className="flex-shrink-0 h-32 rounded-xl overflow-hidden bg-bg-surface hover:ring-2 hover:ring-white/50 transition-all"
                >
                  <img
                    src={photo.thumbnailUrl}
                    alt="Photo"
                    className="h-full w-auto object-contain"
                  />
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Lightbox */}
      {viewingIndex !== null && (
        <div className="fixed inset-0 z-50 bg-black/95 flex flex-col">
          {/* Lightbox header */}
          <div className="flex-none p-6 flex items-center justify-between">
            <span className="text-white/70 text-xl">
              {viewingIndex + 1} / {photos.length}
            </span>
            <button
              onClick={closeLightbox}
              className="w-12 h-12 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center transition-colors"
            >
              <X className="w-6 h-6 text-white" />
            </button>
          </div>

          {/* Photo */}
          <div className="flex-1 flex items-center justify-center p-8 relative">
            <img
              src={photos[viewingIndex].webUrl}
              alt={`Photo ${viewingIndex + 1}`}
              className="max-w-full max-h-full object-contain rounded-lg"
            />

            {/* Nav buttons */}
            {viewingIndex > 0 && (
              <button
                onClick={prevPhoto}
                className="absolute left-8 top-1/2 -translate-y-1/2 w-14 h-14 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center transition-colors"
              >
                <ChevronLeft className="w-8 h-8 text-white" />
              </button>
            )}
            {viewingIndex < photos.length - 1 && (
              <button
                onClick={nextPhoto}
                className="absolute right-8 top-1/2 -translate-y-1/2 w-14 h-14 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center transition-colors"
              >
                <ChevronRight className="w-8 h-8 text-white" />
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
