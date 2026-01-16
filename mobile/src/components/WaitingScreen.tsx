import { motion } from 'framer-motion';
import { Wifi, WifiOff, Camera } from 'lucide-react';

interface WaitingScreenProps {
  isConnected: boolean;
  kioskConnected: boolean;
  isCapturing: boolean;
}

export function WaitingScreen({ isConnected, kioskConnected, isCapturing }: WaitingScreenProps) {
  return (
    <div className="h-full flex flex-col items-center justify-center p-6 text-center">
      {/* Logo */}
      <motion.div
        animate={{ scale: [1, 1.05, 1] }}
        transition={{ duration: 2, repeat: Infinity }}
        className="text-7xl mb-6"
      >
        📸
      </motion.div>

      <h1 className="text-3xl font-bold text-white mb-2">PicPop</h1>

      {/* Connection status */}
      <div className="flex items-center gap-2 mb-8">
        {isConnected ? (
          <>
            <Wifi className="w-5 h-5 text-green-400" />
            <span className="text-green-400 font-medium">Connected</span>
          </>
        ) : (
          <>
            <WifiOff className="w-5 h-5 text-yellow-400 animate-pulse" />
            <span className="text-yellow-400 font-medium">Connecting...</span>
          </>
        )}
      </div>

      {/* Status message */}
      <div className="bg-white/10 backdrop-blur-sm rounded-2xl p-6 max-w-sm">
        {isCapturing ? (
          <div className="flex flex-col items-center gap-3">
            <motion.div
              animate={{ rotate: 360 }}
              transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
            >
              <Camera className="w-8 h-8 text-white" />
            </motion.div>
            <p className="text-white font-medium">Capturing photos...</p>
            <p className="text-white/60 text-sm">Photos will appear here shortly</p>
          </div>
        ) : kioskConnected ? (
          <div className="flex flex-col items-center gap-3">
            <div className="w-12 h-12 rounded-full bg-green-500/20 flex items-center justify-center">
              <Camera className="w-6 h-6 text-green-400" />
            </div>
            <p className="text-white font-medium">Ready!</p>
            <p className="text-white/60 text-sm">
              Tap the button on the touchscreen to take photos
            </p>
          </div>
        ) : (
          <div className="flex flex-col items-center gap-3">
            <motion.div
              animate={{ opacity: [0.5, 1, 0.5] }}
              transition={{ duration: 2, repeat: Infinity }}
              className="w-12 h-12 rounded-full bg-white/10 flex items-center justify-center"
            >
              <Camera className="w-6 h-6 text-white/60" />
            </motion.div>
            <p className="text-white font-medium">Waiting for booth...</p>
            <p className="text-white/60 text-sm">
              The photo booth screen will show when ready
            </p>
          </div>
        )}
      </div>

      {/* Hint */}
      <p className="mt-8 text-white/40 text-sm">
        Photos will appear here automatically
      </p>
    </div>
  );
}
