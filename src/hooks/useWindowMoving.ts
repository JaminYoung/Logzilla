import { useEffect, useRef } from 'react';

const ATTR = 'data-window-moving';

/**
 * Sets document.documentElement[data-window-moving] = "true" during window resize/drag.
 * Components use CSS `[data-window-moving="true"] .acrylic-bar { ... }`
 * to disable expensive backdrop-filter without React re-renders.
 *
 * Two signals feed the flag:
 *  1. A synchronous `pointerdown` on any `[data-tauri-drag-region]` element — this
 *     fires the instant the user presses the title bar, BEFORE Tauri starts the OS
 *     drag, so the acrylic blur is dropped from the very first frame of the drag.
 *     (Tauri's `onMoved` only arrives after the window has already begun moving, so
 *     relying on it alone leaves the first janky frames at full blur.)
 *  2. Tauri `onMoved` / `onResized` events keep the flag alive while the window is
 *     actually moving and, via the debounce, clear it shortly after movement stops.
 */
export function useWindowMoving(debounceMs = 300): void {
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const cleanupRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    let cancelled = false;

    // Mark moving now; (re)arm the debounce that clears the flag once movement stops.
    const markMoving = (holdMs = debounceMs) => {
      if (cancelled) return;
      document.documentElement.setAttribute(ATTR, 'true');
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        if (!cancelled) document.documentElement.removeAttribute(ATTR);
      }, holdMs);
    };

    // --- Signal 1: immediate, latency-free drag-start detection -----------------
    // Capture phase so we run before Tauri's own drag-region handler.
    const onPointerDown = (e: PointerEvent) => {
      const target = e.target as HTMLElement | null;
      if (target && target.closest('[data-tauri-drag-region]')) {
        markMoving();
      }
    };
    // A plain click on the title bar (no drag) still fires pointerdown → clear it
    // promptly on release so the blur returns without a lingering delay.
    const onPointerUp = () => markMoving(120);
    document.addEventListener('pointerdown', onPointerDown, true);
    document.addEventListener('pointerup', onPointerUp, true);

    // --- Signal 2: Tauri window move/resize events ------------------------------
    const init = async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();

        // Window drag fires onMoved; window edge-resize fires onResized.
        const [unlistenResized, unlistenMoved] = await Promise.all([
          appWindow.onResized(() => markMoving()),
          appWindow.onMoved(() => markMoving()),
        ]);

        cleanupRef.current = () => { unlistenResized(); unlistenMoved(); };
      } catch {
        // Not running in Tauri (e.g. plain browser dev) — pointer signals still work.
      }
    };

    init();

    return () => {
      cancelled = true;
      if (timerRef.current) clearTimeout(timerRef.current);
      document.removeEventListener('pointerdown', onPointerDown, true);
      document.removeEventListener('pointerup', onPointerUp, true);
      cleanupRef.current?.();
      document.documentElement.removeAttribute(ATTR);
    };
  }, [debounceMs]);
}
