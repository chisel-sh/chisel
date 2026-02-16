import React, { useState, useEffect } from 'react';

export interface DemoStep {
  command: string;
  output: React.ReactNode;
  duration?: number;
}

interface TerminalDemoProps {
  steps: DemoStep[];
  title?: string;
  loop?: boolean;
  footer?: React.ReactNode;
}

export const TerminalDemo: React.FC<TerminalDemoProps> = ({ 
  steps, 
  title = "chisel — demo", 
  loop = true,
  footer
}) => {
  const [currentStepIndex, setCurrentStepIndex] = useState(0);
  const [displayedCommand, setDisplayedCommand] = useState("");
  const [isTyping, setIsTyping] = useState(true);
  const [showOutput, setShowOutput] = useState(false);

  useEffect(() => {
    const step = steps[currentStepIndex];
    if (!step) return;

    let timeout: ReturnType<typeof setTimeout>;
    
    if (isTyping) {
      if (displayedCommand.length < step.command.length) {
        timeout = setTimeout(() => {
          setDisplayedCommand(step.command.substring(0, displayedCommand.length + 1));
        }, 30 + Math.random() * 40);
      } else {
        timeout = setTimeout(() => {
          setIsTyping(false);
          setShowOutput(true);
        }, 400);
      }
    } else {
      timeout = setTimeout(() => {
        const nextIndex = currentStepIndex + 1;
        if (nextIndex < steps.length || loop) {
          setShowOutput(false);
          setDisplayedCommand("");
          setIsTyping(true);
          setCurrentStepIndex(nextIndex % steps.length);
        }
      }, step.duration || 4000);
    }

    return () => clearTimeout(timeout);
  }, [currentStepIndex, displayedCommand, isTyping, steps, loop]);

  const step = steps[currentStepIndex];

  return (
    <div className="w-full rounded-lg overflow-hidden border border-gray-800 shadow-2xl bg-[#0D0D0D] text-[#EDEDED]">
      {/* Terminal Header */}
      <div className="flex items-center gap-2 px-4 py-3 bg-[#1A1A1A] border-b border-gray-800 select-none">
        <div className="flex gap-1.5 shrink-0">
          <div className="w-3 h-3 rounded-full bg-[#EB5757]"></div>
          <div className="w-3 h-3 rounded-full bg-[#F2C94C]"></div>
          <div className="w-3 h-3 rounded-full bg-[#6EEB83]"></div>
        </div>
        <div className="ml-2 text-xs text-gray-500 font-mono truncate uppercase tracking-widest">{title}</div>
      </div>

      {/* Terminal Body */}
      <div 
        className="p-6 text-left min-h-[420px] flex flex-col terminal-container"
      >
        <div className="flex items-center h-6 shrink-0 terminal-text text-sm sm:text-base">
          <span className="text-[#4DA3FF] mr-2 shrink-0 select-none">$</span>
          <span className="truncate">{displayedCommand}</span>
          {isTyping && (
            <span className="ml-1 w-2 h-5 bg-[#EDEDED] inline-block shrink-0 animate-pulse" />
          )}
        </div>
        
        <div className="mt-6 flex-grow overflow-x-auto overflow-y-hidden">
          {showOutput && (
            <div className="opacity-0 animate-[fadeIn_0.3s_ease-out_forwards] terminal-text">
              {step.output}
            </div>
          )}
        </div>
      </div>

      {/* Terminal Footer */}
      {footer && (
        <div className="px-4 py-2 bg-[#1A1A1A] border-t border-gray-800 font-mono text-[10px] sm:text-xs overflow-x-auto whitespace-nowrap select-none">
          {footer}
        </div>
      )}

      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&display=swap');

        .terminal-container {
          font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Ubuntu Mono', monospace !important;
          font-variant-ligatures: none;
          letter-spacing: 0px;
          -webkit-font-smoothing: antialiased;
        }

        .terminal-text {
          font-size: 14px;
          line-height: 1.25;
        }

        @media (min-width: 640px) {
          .terminal-text {
            font-size: 16px;
          }
        }

        .terminal-text pre {
          font-family: inherit !important;
          margin: 0;
          line-height: inherit;
          white-space: pre;
        }

        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(4px); }
          to { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </div>
  );
};
