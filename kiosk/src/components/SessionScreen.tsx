import { Users, Wifi, Camera, X } from 'lucide-react';

interface SessionScreenProps {
  session: {
    id: string;
    qrCodeUrl: string;
    wifiQrUrl: string;
    phoneCount: number;
  };
  onCapture: () => void;
  onEnd: () => void;
  isCapturing: boolean;
}

export function SessionScreen({
  session,
  onCapture,
  onEnd,
  isCapturing,
}: SessionScreenProps) {
  return (
    <div className="w-full h-full flex">
      {/* Left side - QR codes */}
      <div className="w-1/2 h-full flex flex-col items-center justify-center p-12 bg-bg-surface/30">
        {/* WiFi QR */}
        <div className="mb-8 text-center">
          <div className="flex items-center justify-center gap-3 mb-4">
            <Wifi className="w-8 h-8 text-accent-mint" />
            <h2 className="text-2xl font-semibold text-white">Connect to WiFi</h2>
          </div>
          <div className="bg-white p-4 rounded-2xl">
            <img
              src={session.wifiQrUrl}
              alt="WiFi QR Code"
              className="w-48 h-48"
            />
          </div>
          <p className="mt-3 text-white/60">Scan to connect automatically</p>
        </div>

        {/* Session QR */}
        <div className="text-center">
          <div className="flex items-center justify-center gap-3 mb-4">
            <Camera className="w-8 h-8 text-primary-purple" />
            <h2 className="text-2xl font-semibold text-white">Join Session</h2>
          </div>
          <div className="bg-white p-4 rounded-2xl">
            <img
              src={session.qrCodeUrl}
              alt="Session QR Code"
              className="w-56 h-56"
            />
          </div>
          <p className="mt-3 text-white/60">Then scan this to receive photos</p>
        </div>
      </div>

      {/* Right side - Controls */}
      <div className="w-1/2 h-full flex flex-col items-center justify-center p-12">
        {/* Connected phones indicator */}
        <div className="mb-12 flex items-center gap-4 bg-bg-surface/50 px-8 py-4 rounded-full">
          <Users className="w-8 h-8 text-accent-yellow" />
          <span className="text-3xl font-semibold">
            {session.phoneCount} {session.phoneCount === 1 ? 'phone' : 'phones'} connected
          </span>
        </div>

        {/* Capture button */}
        <button
          onClick={onCapture}
          disabled={isCapturing}
          className={`
            relative w-64 h-64 rounded-full
            flex items-center justify-center
            transition-all duration-300
            ${isCapturing
              ? 'bg-bg-surface/50 cursor-not-allowed'
              : 'bg-gradient-main hover:scale-105 active:scale-95'
            }
          `}
        >
          {/* Outer ring */}
          <span
            className={`
              absolute inset-0 rounded-full border-4
              ${isCapturing ? 'border-white/20' : 'border-white/40'}
            `}
          />
          {/* Pulse effect when ready */}
          {!isCapturing && session.phoneCount > 0 && (
            <span className="absolute inset-0 rounded-full bg-white/20 pulse-ring" />
          )}
          {/* Icon */}
          <Camera
            className={`w-24 h-24 ${isCapturing ? 'text-white/40' : 'text-white'}`}
          />
        </button>

        <p className="mt-8 text-2xl text-white/60">
          {isCapturing
            ? 'Taking photos...'
            : session.phoneCount > 0
              ? 'Tap to take 3 photos!'
              : 'Waiting for phones to connect...'
          }
        </p>

        {/* End session button */}
        <button
          onClick={onEnd}
          className="absolute top-8 right-8 flex items-center gap-2 px-6 py-3 rounded-full bg-white/10 hover:bg-white/20 transition-colors"
        >
          <X className="w-5 h-5" />
          <span>End Session</span>
        </button>
      </div>
    </div>
  );
}
