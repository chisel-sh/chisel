import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

interface DualModeTerminalProps {
  humanCommand: string;
  machineCommand: string;
  humanTitle: string;
  machineTitle: string;
  humanContent?: React.ReactNode;
  machineContent?: React.ReactNode;
  humanFooter?: React.ReactNode;
}

export const DualModeTerminal: React.FC<DualModeTerminalProps> = ({
  humanCommand,
  machineCommand,
  humanTitle,
  machineTitle,
  humanContent,
  machineContent,
  humanFooter
}) => {
  const [mode, setMode] = useState<'human' | 'machine'>('human');

  return (
    <div className="w-full space-y-4">
      {/* Toggle Switch */}
      <div className="flex justify-center">
        <div className="inline-flex p-1 bg-gray-900/50 rounded-lg border border-gray-800">
          <button
            onClick={() => setMode('human')}
            className={`px-4 py-1.5 rounded-md text-xs font-mono transition-all ${mode === 'human'
                ? 'bg-[#EDEDED] text-[#0D0D0D] font-bold shadow-lg'
                : 'text-gray-500 hover:text-gray-300'
              }`}
          >
            DEVELOPER MODE
          </button>
          <button
            onClick={() => setMode('machine')}
            className={`px-4 py-1.5 rounded-md text-xs font-mono transition-all ${mode === 'machine'
                ? 'bg-[#4DA3FF] text-[#0D0D0D] font-bold shadow-lg'
                : 'text-gray-500 hover:text-gray-300'
              }`}
          >
            AI MACHINE MODE
          </button>
        </div>
      </div>

      {/* Terminal Window */}
      <div className="w-full rounded-lg overflow-hidden border border-gray-800 shadow-2xl bg-[#0D0D0D] text-[#EDEDED] flex flex-col h-[400px]">
        {/* Terminal Header */}
        <div className="flex items-center gap-2 px-4 py-3 bg-[#1A1A1A] border-b border-gray-800 select-none shrink-0">
          <div className="flex gap-1.5 shrink-0">
            <div className="w-3 h-3 rounded-full bg-[#EB5757]"></div>
            <div className="w-3 h-3 rounded-full bg-[#F2C94C]"></div>
            <div className="w-3 h-3 rounded-full bg-[#6EEB83]"></div>
          </div>
          <div className="ml-2 text-[10px] text-gray-500 font-mono truncate uppercase tracking-widest">
            {mode === 'human' ? humanTitle : machineTitle}
          </div>
        </div>

        {/* Terminal Body */}
        <div className="p-6 text-left flex flex-col font-mono text-sm sm:text-base overflow-x-auto flex-grow relative">
          <div className="flex items-center h-6 shrink-0 mb-6">
            <span className="text-[#4DA3FF] mr-2 shrink-0 select-none">$</span>
            <span className="truncate">{mode === 'human' ? humanCommand : machineCommand}</span>
          </div>

          <div className="flex-grow">
            <AnimatePresence mode="wait">
              <motion.div
                key={mode}
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -5 }}
                transition={{ duration: 0.2 }}
              >
                {mode === 'human' ? humanContent : machineContent}
              </motion.div>
            </AnimatePresence>
          </div>
        </div>

        {/* Terminal Footer (Always rendered to maintain height, but content is conditional) */}
        <div className="h-10 bg-[#1A1A1A] border-t border-gray-800 font-mono text-[10px] sm:text-xs overflow-hidden shrink-0">
          <AnimatePresence>
            {mode === 'human' && humanFooter && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="px-4 py-2.5 flex items-center h-full overflow-x-auto whitespace-nowrap select-none"
              >
                {humanFooter}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
};
