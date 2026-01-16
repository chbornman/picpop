import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, ChevronLeft, ChevronRight, Image as ImageIcon, Sparkles } from 'lucide-react';
import confetti from 'canvas-confetti';
import type { Photo } from '../types';

interface PhotosScreenProps {
  photos: Photo[];
  stripUrl: string | null;
  isCapturing: boolean;
}

export function PhotosScreen({ photos, stripUrl, isCapturing }: PhotosScreenProps) {
  const [viewingIndex, setViewingIndex] = useState<number | null>(null);
  const [showStrip, setShowStrip] = useState(false);
  const [hasShownConfetti, setHasShownConfetti] = useState(false);

  // Show confetti when first photos arrive
  useEffect(() => {
    if (photos.length > 0 && !hasShownConfetti && !isCapturing) {
      setHasShownConfetti(true);
      confetti({
        particleCount: 100,
        spread: 70,
        origin: { y: 0.6 },
        colors: ['#8B5CF6', '#EC4899', '#FBBF24', '#34D399'],
      });
    }
  }, [photos.length, hasShownConfetti, isCapturing]);

  // Reset confetti flag when capturing starts
  useEffect(() => {
    if (isCapturing) {
      setHasShownConfetti(false);
    }
  }, [isCapturing]);

  const openViewer = (index: number) => {
    setViewingIndex(index);
  };

  const closeViewer = () => {
    setViewingIndex(null);
  };

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
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex-none p-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-2xl">📸</span>
          <span className="text-white font-bold text-lg">PicPop</span>
        </div>
        {isCapturing && (
          <div className="flex items-center gap-2 text-white/70">
            <motion.div
              animate={{ rotate: 360 }}
              transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
              className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full"
            />
            <span className="text-sm">Capturing...</span>
          </div>
        )}
      </div>

      {/* Photo grid */}
      <div className="flex-1 overflow-y-auto p-4 pt-0">
        <div className="grid grid-cols-2 gap-3">
          {photos.map((photo, index) => (
            <motion.button
              key={photo.id}
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: index * 0.1 }}
              onClick={() => openViewer(index)}
              className="aspect-[3/4] rounded-xl overflow-hidden bg-white/10 relative group"
            >
              <img
                src={photo.thumbnailUrl}
                alt={`Photo ${photo.sequence}`}
                className="w-full h-full object-cover"
              />
              <div className="absolute inset-0 bg-black/0 group-active:bg-black/20 transition-colors" />
            </motion.button>
          ))}
        </div>

        {/* Hint text */}
        <p className="text-center text-white/50 text-sm mt-4 mb-2">
          Long-press a photo to save it
        </p>
      </div>

      {/* Bottom actions */}
      {stripUrl && (
        <div className="flex-none p-4 pt-0">
          <button
            onClick={() => setShowStrip(true)}
            className="w-full py-4 rounded-2xl bg-white/20 backdrop-blur-sm text-white font-semibold flex items-center justify-center gap-2 active:scale-[0.98] transition-transform"
          >
            <Sparkles className="w-5 h-5" />
            View Photo Strip
          </button>
        </div>
      )}

      {/* Full photo viewer */}
      <AnimatePresence>
        {viewingIndex !== null && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 bg-black/95 flex flex-col"
          >
            {/* Viewer header */}
            <div className="flex-none p-4 flex items-center justify-between">
              <span className="text-white/70">
                {viewingIndex + 1} / {photos.length}
              </span>
              <button
                onClick={closeViewer}
                className="w-10 h-10 rounded-full bg-white/10 flex items-center justify-center"
              >
                <X className="w-5 h-5 text-white" />
              </button>
            </div>

            {/* Photo */}
            <div className="flex-1 flex items-center justify-center p-4 relative">
              <img
                src={photos[viewingIndex].webUrl}
                alt={`Photo ${photos[viewingIndex].sequence}`}
                className="max-w-full max-h-full object-contain rounded-lg"
              />

              {/* Nav buttons */}
              {viewingIndex > 0 && (
                <button
                  onClick={prevPhoto}
                  className="absolute left-2 top-1/2 -translate-y-1/2 w-10 h-10 rounded-full bg-white/10 flex items-center justify-center"
                >
                  <ChevronLeft className="w-6 h-6 text-white" />
                </button>
              )}
              {viewingIndex < photos.length - 1 && (
                <button
                  onClick={nextPhoto}
                  className="absolute right-2 top-1/2 -translate-y-1/2 w-10 h-10 rounded-full bg-white/10 flex items-center justify-center"
                >
                  <ChevronRight className="w-6 h-6 text-white" />
                </button>
              )}
            </div>

            {/* Hint */}
            <div className="flex-none p-4 text-center">
              <p className="text-white/50 text-sm">
                Long-press the photo to save
              </p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Photo strip viewer */}
      <AnimatePresence>
        {showStrip && stripUrl && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 bg-black/95 flex flex-col"
          >
            {/* Header */}
            <div className="flex-none p-4 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <ImageIcon className="w-5 h-5 text-white/70" />
                <span className="text-white font-medium">Photo Strip</span>
              </div>
              <button
                onClick={() => setShowStrip(false)}
                className="w-10 h-10 rounded-full bg-white/10 flex items-center justify-center"
              >
                <X className="w-5 h-5 text-white" />
              </button>
            </div>

            {/* Strip image */}
            <div className="flex-1 overflow-y-auto p-4 flex items-start justify-center">
              <img
                src={stripUrl}
                alt="Photo Strip"
                className="max-w-full rounded-lg shadow-2xl"
              />
            </div>

            {/* Hint */}
            <div className="flex-none p-4 text-center">
              <p className="text-white/50 text-sm">
                Long-press to save your photo strip
              </p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
