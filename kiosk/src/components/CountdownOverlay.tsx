import { useEffect, useState } from 'react';

interface CountdownOverlayProps {
  value: number;
}

export function CountdownOverlay({ value }: CountdownOverlayProps) {
  const [animate, setAnimate] = useState(false);

  // Trigger animation on value change
  useEffect(() => {
    setAnimate(true);
    const timer = setTimeout(() => setAnimate(false), 300);
    return () => clearTimeout(timer);
  }, [value]);

  return (
    <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm">
      {/* Countdown number */}
      <div
        className={`
          text-[300px] font-bold leading-none
          bg-gradient-main bg-clip-text text-transparent
          transition-all duration-300
          ${animate ? 'scale-125 opacity-100' : 'scale-100 opacity-80'}
        `}
      >
        {value}
      </div>

      {/* Ring pulse effect */}
      <div
        className={`
          absolute w-96 h-96 rounded-full border-8 border-white/30
          transition-all duration-500
          ${animate ? 'scale-150 opacity-0' : 'scale-100 opacity-30'}
        `}
      />

      {/* Get ready text */}
      <div className="absolute bottom-32 text-3xl text-white/80 font-semibold">
        Get ready!
      </div>
    </div>
  );
}
