import { Camera } from 'lucide-react';

interface IdleScreenProps {
  onStart: () => void;
  error: string | null;
}

export function IdleScreen({ onStart, error }: IdleScreenProps) {
  return (
    <div className="w-full h-full flex flex-col items-center justify-center">
      {/* Logo and title */}
      <div className="mb-16 text-center">
        <div className="w-32 h-32 mx-auto mb-8 rounded-full bg-gradient-main flex items-center justify-center animate-pulse-slow">
          <Camera className="w-16 h-16 text-white" />
        </div>
        <h1 className="text-7xl font-bold bg-gradient-main bg-clip-text text-transparent">
          PicPop
        </h1>
        <p className="text-2xl text-white/60 mt-4">
          Touch to start your photo session
        </p>
      </div>

      {/* Start button */}
      <button
        onClick={onStart}
        className="group relative px-16 py-8 rounded-full bg-gradient-main text-white text-3xl font-semibold transition-all hover:scale-105 active:scale-95"
      >
        {/* Pulse ring effect */}
        <span className="absolute inset-0 rounded-full bg-gradient-main opacity-50 pulse-ring" />
        <span className="relative z-10">Start Session</span>
      </button>

      {/* Error message */}
      {error && (
        <div className="mt-8 bg-red-500/20 border border-red-500/50 text-red-300 px-8 py-4 rounded-2xl text-xl max-w-lg text-center">
          {error}
        </div>
      )}

      {/* Instructions */}
      <div className="absolute bottom-12 text-center text-white/40 text-lg">
        <p>Scan the QR code with your phone to receive your photos instantly</p>
      </div>
    </div>
  );
}
