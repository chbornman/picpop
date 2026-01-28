import { useState, useEffect, useCallback } from 'react';
import { Camera } from 'lucide-react';

interface WelcomeScreenProps {
  onStart: () => void;
  isLoading: boolean;
  previewUrl: string;
}

export function WelcomeScreen({ onStart, isLoading, previewUrl }: WelcomeScreenProps) {
  const [previewError, setPreviewError] = useState(false);
  const [previewKey, setPreviewKey] = useState(0);

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

  return (
    <div className="w-full h-full relative">
      {/* Camera preview background */}
      <div className="absolute inset-0">
        {!previewError ? (
          <img
            key={previewKey}
            src={`${previewUrl}${previewUrl.includes('?') ? '&' : '?'}_t=${previewKey}`}
            alt="Camera Preview"
            className="w-full h-full object-cover"
            onError={() => setPreviewError(true)}
          />
        ) : (
          <div className="w-full h-full bg-bg-dark" />
        )}
        {/* Gradient overlay disabled for FPS testing */}
        {/* <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-black/60" /> */}
      </div>

      {/* Branding overlay */}
      <div className="relative z-10 w-full h-full flex flex-col items-center justify-center p-12">
        {/* Logo and title */}
        <div className="mb-16 text-center">
          <div className="w-40 h-40 mx-auto mb-8 rounded-full bg-gradient-main flex items-center justify-center shadow-2xl">
            <Camera className="w-20 h-20 text-white" />
          </div>
          <h1 className="text-8xl font-bold bg-gradient-main bg-clip-text text-transparent drop-shadow-lg">
            PicPop
          </h1>
          <p className="text-3xl text-white/80 mt-6 drop-shadow-md">
            Photo Booth
          </p>
        </div>

        {/* Start button */}
        <button
          onClick={onStart}
          disabled={isLoading}
          className={`
            group relative px-20 py-10 rounded-full text-white text-4xl font-semibold
            transition-all shadow-2xl
            ${isLoading
              ? 'bg-white/20 cursor-wait'
              : 'bg-gradient-main hover:scale-105 active:scale-95'
            }
          `}
        >
          {!isLoading && (
            <span className="absolute inset-0 rounded-full bg-gradient-main opacity-50 pulse-ring" />
          )}
          <span className="relative z-10">
            {isLoading ? 'Starting...' : 'Start Session'}
          </span>
        </button>
      </div>
    </div>
  );
}
