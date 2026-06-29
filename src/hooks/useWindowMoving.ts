import { useEffect, useRef } from 'react';

const ATTR = 'data-window-moving';

/**
 * Sets document.documentElement[data-window-moving] = "true" during window resize/drag.
 * Components use CSS `[data-window-moving="true"] .acrylic-bar { ... }`
 * to disable expensive backdrop-filter without React re-renders.
 */
export function useWindowMoving(debounceMs = 300): void {
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const cleanupRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    let cancelled = false;

    const init = async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const window = getCurrentWindow();

        const scheduleReset = () => {
          document.documentElement.setAttribute(ATTR, 'true');
          if (timerRef.current) clearTimeout(timerRef.current);
          timerRef.current = setTimeout(() => {
            if (!cancelled) {
              document.documentElement.removeAttribute(ATTR);
            }
          }, debounceMs);
        };

        // Listen to BOTH resize and move events — window drag fires onMoved,
        // window edge-resize fires onResized.  Both need blur suppression.
        const [unlistenResized, unlistenMoved] = await Promise.all([
          window.onResized(() => { if (!cancelled) scheduleReset(); }),
          window.onMoved(() => { if (!cancelled) scheduleReset(); }),
        ]);

        cleanupRef.current = () => { unlistenResized(); unlistenMoved(); };
      } catch {
        // Not running in Tauri — silently ignore
      }
    };

    init();

    return () => {
      cancelled = true;
      if (timerRef.current) clearTimeout(timerRef.current);
      cleanupRef.current?.();
      document.documentElement.removeAttribute(ATTR);
    };
  }, [debounceMs]);
}
