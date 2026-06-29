import { Minus, X } from 'lucide-react';
import { motion } from 'motion/react';
import { useState, useEffect, useRef } from 'react';

function RestoreIcon({ className }: { className?: string }) {
  // Standard Windows 11 restore icon: two overlapping rectangles
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 10 10"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1"
    >
      {/* Back rectangle */}
      <path d="M2.5 1 h5 a1 1 0 0 1 1 1 v5" />
      {/* Front rectangle */}
      <rect x="1" y="3" width="6" height="6" rx="0.5" />
    </svg>
  );
}

export function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);
  const cleanupRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    let cancelled = false;

    const init = async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const window = getCurrentWindow();
        const maximized = await window.isMaximized();
        if (!cancelled) setIsMaximized(maximized);

        // Debounced resize → maximize check (IPC is expensive, don't fire every frame)
        let timer: ReturnType<typeof setTimeout> | null = null;
        const unlisten = await window.onResized(() => {
          if (timer) clearTimeout(timer);
          timer = setTimeout(() => {
            window.isMaximized().then(m => {
              if (!cancelled) setIsMaximized(m);
            });
          }, 200);
        });
        cleanupRef.current = unlisten;
      } catch (e) {
        console.error('TitleBar init error:', e);
      }
    };

    init();

    return () => {
      cancelled = true;
      cleanupRef.current?.();
    };
  }, []);

  const handleMinimize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      await window.minimize();
    } catch (e) {
      console.error('Minimize error:', e);
    }
  };

  const handleMaximize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      const maximized = await window.isMaximized();
      if (maximized) {
        await window.unmaximize();
      } else {
        await window.maximize();
      }
    } catch (e) {
      console.error('Maximize error:', e);
    }
  };

  const handleClose = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      await window.close();
    } catch (e) {
      console.error('Close error:', e);
    }
  };

  return (
    <div
      className="acrylic-bar h-12 flex items-center justify-between select-none border-b shrink-0"
      style={{
        background: 'var(--acrylic-tint)',
        backdropFilter: 'blur(40px) saturate(150%)',
        WebkitBackdropFilter: 'blur(40px) saturate(150%)',
        borderColor: 'var(--acrylic-border)'
      }}
      data-tauri-drag-region
    >
      {/* Left - App Name & Icon */}
      <div className="flex items-center gap-2 pl-4" data-tauri-drag-region>
        <img src="/logo.svg" alt="Logzilla" className="w-8 h-8 select-none pointer-events-none" data-tauri-drag-region />
        <span className="font-semibold" data-tauri-drag-region>Logzilla</span>
      </div>

      {/* Right - Window Controls - flush right with no gap */}
      <div className="flex h-full">
        <motion.button
          whileHover={{ backgroundColor: 'rgba(128,128,128,0.1)' }}
          className="w-12 h-full flex items-center justify-center transition-colors"
          onClick={handleMinimize}
        >
          <Minus className="w-4 h-4" />
        </motion.button>
        <motion.button
          whileHover={{ backgroundColor: 'rgba(128,128,128,0.1)' }}
          className="w-12 h-full flex items-center justify-center transition-colors"
          onClick={handleMaximize}
        >
          {isMaximized ? (
            <RestoreIcon className="w-3.5 h-3.5" />
          ) : (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 16 16"
              className="w-3.5 h-3.5"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <rect x="2.5" y="2.5" width="11" height="11" rx="1" />
            </svg>
          )}
        </motion.button>
        <motion.button
          whileHover={{ backgroundColor: '#c42b1c' }}
          className="w-12 h-full flex items-center justify-center transition-colors hover:text-white"
          onClick={handleClose}
        >
          <X className="w-4 h-4" />
        </motion.button>
      </div>
    </div>
  );
}
