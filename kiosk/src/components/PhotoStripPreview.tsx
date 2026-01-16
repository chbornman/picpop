import { Camera, Check, RotateCcw } from 'lucide-react';

interface Photo {
  id: string;
  sequence: number;
  url: string;
  thumbnailUrl: string;
}

interface PhotoStripPreviewProps {
  photos: Photo[];
  stripUrl: string | null;
  onTakeMore: () => void;
  onEnd: () => void;
}

export function PhotoStripPreview({
  photos,
  stripUrl,
  onTakeMore,
  onEnd,
}: PhotoStripPreviewProps) {
  const sortedPhotos = [...photos].sort((a, b) => a.sequence - b.sequence);

  return (
    <div className="w-full h-full flex flex-col items-center justify-center p-12">
      {/* Success header */}
      <div className="mb-8 flex items-center gap-4">
        <div className="w-16 h-16 rounded-full bg-accent-mint flex items-center justify-center">
          <Check className="w-10 h-10 text-white" />
        </div>
        <h1 className="text-5xl font-bold text-white">Photos Captured!</h1>
      </div>

      {/* Photo preview */}
      <div className="flex gap-6 mb-12">
        {sortedPhotos.map((photo) => (
          <div
            key={photo.id}
            className="w-64 h-80 rounded-2xl overflow-hidden bg-bg-surface shadow-2xl"
          >
            <img
              src={photo.thumbnailUrl}
              alt={`Photo ${photo.sequence}`}
              className="w-full h-full object-cover"
            />
          </div>
        ))}
      </div>

      {/* Photo strip preview if available */}
      {stripUrl && (
        <div className="mb-12 text-center">
          <p className="text-xl text-white/60 mb-4">Photo Strip</p>
          <div className="h-96 rounded-2xl overflow-hidden shadow-2xl">
            <img
              src={stripUrl}
              alt="Photo Strip"
              className="h-full object-contain"
            />
          </div>
        </div>
      )}

      {/* Sent to phones indicator */}
      <p className="text-2xl text-accent-mint mb-12">
        Photos sent to connected phones!
      </p>

      {/* Action buttons */}
      <div className="flex gap-8">
        <button
          onClick={onTakeMore}
          className="flex items-center gap-3 px-10 py-5 rounded-full bg-gradient-main text-white text-2xl font-semibold hover:scale-105 active:scale-95 transition-transform"
        >
          <Camera className="w-8 h-8" />
          <span>Take More</span>
        </button>

        <button
          onClick={onEnd}
          className="flex items-center gap-3 px-10 py-5 rounded-full bg-white/10 text-white text-2xl font-semibold hover:bg-white/20 active:scale-95 transition-all"
        >
          <RotateCcw className="w-8 h-8" />
          <span>End Session</span>
        </button>
      </div>
    </div>
  );
}
