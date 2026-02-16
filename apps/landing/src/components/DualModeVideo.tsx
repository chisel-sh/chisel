import React, { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

interface DualModeVideoProps {
  humanVideo: string;
  machineVideo: string;
  humanTitle: string;
  machineTitle: string;
  humanFooter?: React.ReactNode;
}

export const DualModeVideo: React.FC<DualModeVideoProps> = ({
  humanVideo,
  machineVideo,
  humanTitle,
  machineTitle,
  humanFooter
}) => {
  const [mode, setMode] = useState<'human' | 'machine'>('human');
  const videoRef = useRef<HTMLVideoElement>(null);

  // Handle visibility autoplay
  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            video.play().catch(() => {});
          } else {
            video.pause();
          }
        });
      },
      { threshold: 0.5 }
    );

    observer.observe(video);

    return () => {
      observer.disconnect();
    };
  }, [mode]);

  return (
    <div className="w-full rounded-lg overflow-hidden border border-gray-800 shadow-2xl bg-[#0D0D0D] text-[#EDEDED] flex flex-col">
      {/* Header with Tabs */}
      <div className="flex items-center gap-4 px-4 bg-[#1A1A1A] border-b border-gray-800 select-none shrink-0 h-12">
        <div className="flex gap-1.5 shrink-0">
          <div className="w-3 h-3 rounded-full bg-[#EB5757]"></div>
          <div className="w-3 h-3 rounded-full bg-[#F2C94C]"></div>
          <div className="w-3 h-3 rounded-full bg-[#6EEB83]"></div>
        </div>
        
        {/* Tabs */}
        <div className="flex h-full ml-4 space-x-1">
            <button
                onClick={() => setMode('human')}
                className={`px-4 h-full border-b-2 text-xs font-mono font-bold tracking-wider transition-colors flex items-center ${
                    mode === 'human' 
                    ? 'border-[#4DA3FF] text-[#EDEDED] bg-white/5' 
                    : 'border-transparent text-gray-500 hover:text-gray-300 hover:bg-white/5'
                }`}
            >
                HUMAN MODE
            </button>
            <button
                onClick={() => setMode('machine')}
                className={`px-4 h-full border-b-2 text-xs font-mono font-bold tracking-wider transition-colors flex items-center ${
                    mode === 'machine' 
                    ? 'border-[#4DA3FF] text-[#EDEDED] bg-white/5' 
                    : 'border-transparent text-gray-500 hover:text-gray-300 hover:bg-white/5'
                }`}
            >
                MACHINE MODE
            </button>
        </div>
      </div>

      {/* Terminal Body (Video) */}
      <div className="relative w-full aspect-[16/10] bg-[#0D0D0D] overflow-hidden group">
          <AnimatePresence mode="wait">
            <motion.div
              key={mode}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.3 }}
              className="absolute inset-0 w-full h-full"
            >
              <video
                  ref={videoRef}
                  src={mode === 'human' ? humanVideo : machineVideo}
                  autoPlay
                  loop
                  muted
                  playsInline
                  className="w-full h-full object-contain"
              />
            </motion.div>
          </AnimatePresence>
      </div>

      {/* Explainer Context */}
      <div className="bg-[#050505] border-t border-gray-800 px-6 py-4">
        <AnimatePresence mode="wait">
            <motion.div
                key={mode}
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -5 }}
                transition={{ duration: 0.2 }}
                className="flex items-start gap-3"
            >
                <div className={`mt-1 w-1.5 h-1.5 rounded-full shrink-0 ${mode === 'human' ? 'bg-[#4DA3FF]' : 'bg-[#BB6BD9]'}`} />
                <div className="space-y-1">
                    <p className="text-sm font-bold text-[#EDEDED]">
                        {mode === 'human' ? 'Interactive TUI for Developers' : 'Token-Dense YAML for Agents'}
                    </p>
                    <p className="text-xs text-gray-400 font-mono leading-relaxed">
                        {mode === 'human' 
                            ? 'Full terminal UI with colors, borders, and keyboard shortcuts (j/k) for efficient human navigation.' 
                            : 'Deterministic, structured output optimized for LLM context windows. No visual clutter, just data.'}
                    </p>
                </div>
            </motion.div>
        </AnimatePresence>
      </div>

      {/* Legacy Footer (Shortcuts) - Only show in Human Mode */}
      <AnimatePresence>
        {mode === 'human' && humanFooter && (
            <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: 'auto', opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                className="bg-[#1A1A1A] border-t border-gray-800 overflow-hidden"
            >
                <div className="px-4 py-2 flex items-center whitespace-nowrap overflow-x-auto text-[10px] sm:text-xs text-gray-500 font-mono">
                    {humanFooter}
                </div>
            </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};
