import { motion } from 'framer-motion';

export function ProcessingScreen() {
  return (
    <div className="h-full flex flex-col items-center justify-center">
      {/* Spinner */}
      <motion.div
        className="w-20 h-20 border-4 border-white/20 border-t-white rounded-full"
        animate={{ rotate: 360 }}
        transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
      />

      {/* Text */}
      <motion.p
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-white text-2xl font-medium mt-8"
      >
        Processing photos...
      </motion.p>

      <motion.p
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.3 }}
        className="text-white/60 text-lg mt-2"
      >
        Almost done!
      </motion.p>
    </div>
  );
}
