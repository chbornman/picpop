import { motion } from 'framer-motion';

export function ProcessingScreen() {
  return (
    <div className="h-full flex flex-col items-center justify-center">
      {/* Animated photo stack */}
      <div className="relative w-32 h-32 mb-8">
        {[0, 1, 2].map((i) => (
          <motion.div
            key={i}
            className="absolute inset-0 rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 shadow-lg"
            initial={{ rotate: (i - 1) * 8, scale: 1 - i * 0.05 }}
            animate={{
              rotate: [(i - 1) * 8, (i - 1) * 8 + 5, (i - 1) * 8],
              scale: [1 - i * 0.05, 1 - i * 0.03, 1 - i * 0.05],
              y: [0, -5, 0],
            }}
            transition={{
              duration: 2,
              repeat: Infinity,
              delay: i * 0.15,
              ease: 'easeInOut',
            }}
            style={{
              zIndex: 3 - i,
              opacity: 1 - i * 0.2,
            }}
          >
            {/* Photo icon inside */}
            {i === 0 && (
              <div className="absolute inset-0 flex items-center justify-center">
                <motion.div
                  animate={{ scale: [1, 1.1, 1] }}
                  transition={{ duration: 1.5, repeat: Infinity }}
                >
                  <svg
                    className="w-12 h-12 text-white/80"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={1.5}
                      d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                    />
                  </svg>
                </motion.div>
              </div>
            )}
          </motion.div>
        ))}
      </div>

      {/* Animated dots text */}
      <div className="flex items-baseline gap-1">
        <motion.span
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-white text-2xl font-medium"
        >
          Processing
        </motion.span>
        <div className="flex gap-[3px] ml-1">
          {[0, 1, 2].map((i) => (
            <motion.span
              key={i}
              className="text-white text-2xl font-medium"
              animate={{ y: [0, -8, 0] }}
              transition={{
                duration: 0.6,
                repeat: Infinity,
                delay: i * 0.15,
                ease: 'easeInOut',
              }}
            >
              .
            </motion.span>
          ))}
        </div>
      </div>

      {/* Subtitle */}
      <motion.p
        initial={{ opacity: 0 }}
        animate={{ opacity: [0.4, 0.7, 0.4] }}
        transition={{ duration: 2, repeat: Infinity }}
        className="text-white/60 text-lg mt-3"
      >
        Almost done!
      </motion.p>
    </div>
  );
}
