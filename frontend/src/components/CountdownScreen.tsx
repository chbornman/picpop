import { motion } from 'framer-motion';
import { useEffect } from 'react';

interface CountdownScreenProps {
  value: number;
  photoNumber?: number;
  totalPhotos?: number;
}

// Vibrate for haptic feedback
function vibrate(pattern: number | number[]) {
  if ('vibrate' in navigator) {
    navigator.vibrate(pattern);
  }
}

const colors: Record<number, string> = {
  3: '#8B5CF6', // Purple
  2: '#FBBF24', // Yellow
  1: '#EC4899', // Pink
};

const messages: Record<number, string> = {
  3: 'Get ready!',
  2: 'Strike a pose!',
  1: 'Smile!',
};

export function CountdownScreen({ value, photoNumber = 1, totalPhotos = 1 }: CountdownScreenProps) {
  // Haptic feedback on each number
  useEffect(() => {
    if (value === 1) {
      vibrate([100, 50, 100]); // Strong pulse for final countdown
    } else {
      vibrate(50);
    }
  }, [value]);

  return (
    <div className="h-full flex flex-col items-center justify-center">
      {/* Photo indicator - show which photo we're on (e.g., 1/3) */}
      {totalPhotos > 1 && (
        <motion.div
          key={`photo-${photoNumber}`}
          initial={{ y: -20, opacity: 0, scale: 0.8 }}
          animate={{ y: 0, opacity: 1, scale: 1 }}
          className="mb-6 px-5 py-2 rounded-full bg-white/10 backdrop-blur-sm border border-white/20"
        >
          <span className="text-white text-xl font-semibold tracking-wide">
            {photoNumber}
            <span className="text-white/50 mx-1">/</span>
            {totalPhotos}
          </span>
        </motion.div>
      )}

      {/* Large number */}
      <motion.div
        key={`${photoNumber}-${value}`}
        initial={{ scale: 2, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.5, opacity: 0 }}
        transition={{ duration: 0.3, ease: 'easeOut' }}
        className="relative"
      >
        {/* Glow effect */}
        <div
          className="absolute inset-0 blur-3xl opacity-50"
          style={{ backgroundColor: colors[value] || '#8B5CF6' }}
        />

        {/* Number */}
        <span
          className="relative text-[200px] font-bold leading-none"
          style={{
            color: colors[value] || '#FFFFFF',
            textShadow: `0 0 60px ${colors[value] || '#8B5CF6'}`,
          }}
        >
          {value}
        </span>
      </motion.div>

      {/* Message */}
      <motion.p
        key={`msg-${photoNumber}-${value}`}
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        className="text-white text-2xl font-medium mt-8"
      >
        {messages[value] || ''}
      </motion.p>

      {/* Progress dots for countdown */}
      <div className="flex gap-3 mt-8">
        {[3, 2, 1].map((n) => (
          <motion.div
            key={n}
            className="w-3 h-3 rounded-full"
            animate={{
              backgroundColor: n >= value ? '#FFFFFF' : 'rgba(255,255,255,0.3)',
              scale: n === value ? 1.2 : 1,
            }}
          />
        ))}
      </div>

      {/* Photo progress indicator */}
      {totalPhotos > 1 && (
        <div className="flex gap-2 mt-6">
          {Array.from({ length: totalPhotos }, (_, i) => i + 1).map((n) => (
            <motion.div
              key={`photo-dot-${n}`}
              className="w-2 h-2 rounded-full"
              animate={{
                backgroundColor:
                  n < photoNumber
                    ? '#8B5CF6'
                    : n === photoNumber
                      ? '#FFFFFF'
                      : 'rgba(255,255,255,0.3)',
                scale: n === photoNumber ? 1.3 : 1,
              }}
            />
          ))}
        </div>
      )}
    </div>
  );
}
