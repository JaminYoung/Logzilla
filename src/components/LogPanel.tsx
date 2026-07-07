import { useRef, useEffect, useState, useMemo, useCallback } from 'react';
import { motion } from 'motion/react';
import { Trash2, SaveAll, Copy, Scan, Search as SearchIcon, Circle, ChevronDown, Bluetooth, Usb, Cable, HelpCircle, LocateFixed } from 'lucide-react';

import { type FilterRule } from './FilterSettingsDialog';
import { type HighlightRule } from './HighlightSettingsDialog';
import { ProtocolTooltip, type ParsedProtocol } from './ProtocolTooltip';

function matchesText(text: string, query: string, mode: 'fuzzy' | 'plain' | 'regex', caseSensitive: boolean): boolean {
  if (!query) return false;
  try {
    if (mode === 'regex') {
      return new RegExp(query, caseSensitive ? '' : 'i').test(text);
    }
    // For fuzzy and plain, use simple includes matching
    const t = caseSensitive ? text : text.toLowerCase();
    const q = caseSensitive ? query : query.toLowerCase();
    return t.includes(q);
  } catch { return false; }
}

/**
 * Wrap matched substrings in <mark> elements for keyword-level highlighting.
 * Returns a DocumentFragment with <mark> around each match, or null if no match.
 */
function highlightSearchKeywords(text: string, query: string, mode: 'fuzzy' | 'plain' | 'regex', caseSensitive: boolean): DocumentFragment | null {
  if (!query) return null;
  try {
    const flags = caseSensitive ? 'g' : 'gi';
    let re: RegExp;
    if (mode === 'regex') {
      re = new RegExp(query, flags);
    } else {
      // plain and fuzzy both do substring matching
      const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      re = new RegExp(escaped, flags);
    }
    const frag = document.createDocumentFragment();
    let lastIdx = 0;
    let m: RegExpExecArray | null;
    let found = false;
    while ((m = re.exec(text)) !== null) {
      found = true;
      if (m.index > lastIdx) {
        frag.appendChild(document.createTextNode(text.slice(lastIdx, m.index)));
      }
      const mark = document.createElement('mark');
      mark.className = 'search-kw';
      mark.textContent = m[0];
      frag.appendChild(mark);
      lastIdx = m.index + m[0].length;
      // Prevent infinite loop for zero-length matches
      if (m[0].length === 0) { re.lastIndex++; }
    }
    if (!found) return null;
    if (lastIdx < text.length) {
      frag.appendChild(document.createTextNode(text.slice(lastIdx)));
    }
    return frag;
  } catch { return null; }
}

function createHighlightedFragment(text: string, rules: HighlightRule[]): DocumentFragment | null {
  interface Match { start: number; end: number; textColor: string; bgColor: string; }
  const matches: Match[] = [];

  for (const rule of rules) {
    try {
      if (rule.matchType === 'regex') {
        const re = new RegExp(rule.keyword, 'gi');
        let m;
        while ((m = re.exec(text)) !== null) {
          matches.push({ start: m.index, end: m.index + m[0].length, textColor: rule.textColor, bgColor: rule.bgColor });
        }
      } else {
        const lower = text.toLowerCase();
        const kw = rule.keyword.toLowerCase();
        let idx = lower.indexOf(kw);
        while (idx !== -1) {
          matches.push({ start: idx, end: idx + kw.length, textColor: rule.textColor, bgColor: rule.bgColor });
          idx = lower.indexOf(kw, idx + 1);
        }
      }
    } catch {}
  }

  if (matches.length === 0) return null;

  matches.sort((a, b) => a.start - b.start || b.end - a.end);

  const merged: Match[] = [];
  for (const m of matches) {
    if (merged.length > 0 && m.start < merged[merged.length - 1].end) {
      if (m.end > merged[merged.length - 1].end) {
        merged[merged.length - 1].end = m.end;
        merged[merged.length - 1].textColor = m.textColor;
        merged[merged.length - 1].bgColor = m.bgColor;
      }
    } else {
      merged.push({ ...m });
    }
  }

  const frag = document.createDocumentFragment();
  let cursor = 0;
  for (const m of merged) {
    if (m.start > cursor) {
      frag.appendChild(document.createTextNode(text.slice(cursor, m.start)));
    }
    const hl = document.createElement('span');
    hl.style.color = m.textColor;
    if (m.bgColor !== 'transparent') hl.style.backgroundColor = m.bgColor;
    hl.style.borderRadius = '2px';
    hl.style.fontWeight = '600';
    hl.textContent = text.slice(m.start, m.end);
    frag.appendChild(hl);
    cursor = m.end;
  }
  if (cursor < text.length) {
    frag.appendChild(document.createTextNode(text.slice(cursor)));
  }
  return frag;
}

interface DomLogListProps {
  lines: string[];
  emptyMessage: string;
  onContextMenu?: (e: React.MouseEvent) => void;
  highlightEnabled: boolean;
  highlightRules: HighlightRule[];
  fontSize: 'xs' | 'sm' | 'base';
  viewId: string;
  searchQuery: string;
  searchMode: 'fuzzy' | 'plain' | 'regex';
  searchCaseSensitive: boolean;
  searchActive: boolean;
  /** Parallel to `lines`: original index into the source buffer for each row.
   *  Only the filter view passes this so its rows carry data-original-index,
   *  enabling "定位日志" to scroll the main view to the matching source row. */
  originalIndices?: number[];
  /** When true, the rAF auto-follow-to-bottom is skipped (used while locating
   *  a row so smooth-scroll isn't yanked back to the bottom). */
  pauseAutoFollowRef?: React.RefObject<boolean>;
}

function DomLogList({
  lines,
  emptyMessage,
  onContextMenu,
  highlightEnabled: hlEnabled,
  highlightRules: hlRules,
  fontSize,
  viewId,
  searchQuery,
  searchMode,
  searchCaseSensitive,
  searchActive,
  originalIndices,
  pauseAutoFollowRef,
}: DomLogListProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollableRef = useRef<HTMLDivElement>(null);
  const pendingRef = useRef<{ text: string; idx?: number }[]>([]);
  const prevLenRef = useRef(0);
  const prevLinesRef = useRef<string[]>([]);
  const isAtBottomRef = useRef(true);
  const hlEnabledRef = useRef(hlEnabled);
  const hlRulesRef = useRef(hlRules);
  const fontSizeRef = useRef(fontSize);
  const searchActiveRef = useRef(searchActive);
  const searchQueryRef = useRef(searchQuery);
  const searchModeRef = useRef(searchMode);
  const searchCaseSensitiveRef = useRef(searchCaseSensitive);
  const originalIndicesRef = useRef<number[] | undefined>(originalIndices);
  hlEnabledRef.current = hlEnabled;
  hlRulesRef.current = hlRules;
  fontSizeRef.current = fontSize;
  searchActiveRef.current = searchActive;
  searchQueryRef.current = searchQuery;
  searchModeRef.current = searchMode;
  searchCaseSensitiveRef.current = searchCaseSensitive;
  originalIndicesRef.current = originalIndices;

  // Perpetual rAF flush loop — reads data-window-moving from DOM to avoid
  // React re-renders. During window resize, skips scrollTop (the expensive
  // forced-layout operation) and DOM append, but keeps the loop alive.
  useEffect(() => {
    let raf: number;
    const flush = () => {
      try {
        const isMoving = document.documentElement.getAttribute('data-window-moving') === 'true';

        if (pendingRef.current.length > 0 && containerRef.current && !isMoving) {
          const isDark = document.documentElement.classList.contains('dark');
          const batch = pendingRef.current.splice(0);
          const doHighlight = hlEnabledRef.current && hlRulesRef.current.length > 0;
          const doSearch = searchActiveRef.current && searchQueryRef.current;
          const q = searchQueryRef.current;
          const mode = searchModeRef.current;
          const cs = searchCaseSensitiveRef.current;
          const frag = document.createDocumentFragment();
          for (const { text, idx } of batch) {
            const div = document.createElement('div');
            div.className = isDark
              ? 'py-0.5 hover:bg-white/5 px-2 -mx-2 rounded transition-colors'
              : 'py-0.5 hover:bg-accent/60 px-2 -mx-2 rounded transition-colors';
            div.dataset.originalText = text;
            if (idx !== undefined) div.dataset.originalIndex = String(idx);
            const span = document.createElement('span');
            span.className = 'select-text break-all';
            const fs = fontSizeRef.current;
            span.style.fontSize = fs === 'xs' ? '0.75rem' : fs === 'sm' ? '0.875rem' : '1rem';
            if (doHighlight) {
              const fragHL = createHighlightedFragment(text, hlRulesRef.current);
              if (fragHL) {
                span.appendChild(fragHL);
              } else {
                span.textContent = text;
              }
            } else {
              span.textContent = text;
            }
            if (doSearch) {
              const isMatch = matchesText(text, q, mode, cs);
              if (isMatch) {
                div.classList.add('search-match');
                div.dataset.searchMatch = '';
                const kwFrag = highlightSearchKeywords(text, q, mode, cs);
                if (kwFrag) {
                  span.textContent = '';
                  span.appendChild(kwFrag);
                }
              }
            }
            div.appendChild(span);
            frag.appendChild(div);
          }
          containerRef.current.appendChild(frag);
        }

        // Auto-scroll: skip during window resize to avoid forced layout of 1000s of lines
        if (isAtBottomRef.current && !searchActiveRef.current && scrollableRef.current && !isMoving && !pauseAutoFollowRef?.current) {
          scrollableRef.current.scrollTop = scrollableRef.current.scrollHeight;
        }
      } catch (e) {
        console.error('[DomLogList] flush error:', e);
      }
      raf = requestAnimationFrame(flush);
    };
    raf = requestAnimationFrame(flush);
    return () => cancelAnimationFrame(raf);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Push new data to pending queue — light React touch only
  useEffect(() => {
    if (lines.length > prevLenRef.current) {
      const start = prevLenRef.current;
      const slice = lines.slice(start);
      const idxArr = originalIndicesRef.current;
      const mapped = slice.map((text, i) => ({
        text,
        idx: idxArr ? idxArr[start + i] : undefined,
      }));
      pendingRef.current.push(...mapped);
      prevLenRef.current = lines.length;
      prevLinesRef.current = lines;
    } else if (lines.length < prevLenRef.current) {
      // Logs were cleared (or trimmed) — reset DOM and refs so new logs render correctly
      if (containerRef.current) containerRef.current.innerHTML = '';
      pendingRef.current = [];
      prevLenRef.current = 0;
      isAtBottomRef.current = true;
      if (lines.length > 0) {
        const idxArr = originalIndicesRef.current;
        pendingRef.current.push(...lines.map((text, i) => ({
          text,
          idx: idxArr ? idxArr[i] : undefined,
        })));
        prevLenRef.current = lines.length;
      }
      prevLinesRef.current = lines;
    }
  }, [lines]);

  useEffect(() => {
    if (lines === prevLinesRef.current || lines.length !== prevLenRef.current) return;
    if (containerRef.current) containerRef.current.innerHTML = '';
    pendingRef.current = [];
    const idxArr = originalIndicesRef.current;
    pendingRef.current.push(...lines.map((text, i) => ({
      text,
      idx: idxArr ? idxArr[i] : undefined,
    })));
    prevLinesRef.current = lines;
    isAtBottomRef.current = true;
  }, [lines]);

  // Re-scan existing DOM for search match highlighting (batched for performance)
  useEffect(() => {
    if (!containerRef.current) return;
    const container = containerRef.current;
    const divs = Array.from(container.querySelectorAll<HTMLDivElement>(':scope > div'));
    const total = divs.length;
    if (total === 0) return;

    let index = 0;
    const batchSize = 200;
    let cancelled = false;

    const processBatch = () => {
      if (cancelled) return;
      const end = Math.min(index + batchSize, total);
      for (let i = index; i < end; i++) {
        const div = divs[i];
        const text = div.dataset.originalText || div.textContent || '';
        const isMatch = searchActive && searchQuery && matchesText(text, searchQuery, searchMode, searchCaseSensitive);
        const span = div.querySelector(':scope > span') as HTMLElement | null;
        if (isMatch) {
          div.classList.add('search-match');
          div.dataset.searchMatch = '';
          // Rebuild keyword-level <mark> highlighting
          if (span) {
            const kwFrag = highlightSearchKeywords(text, searchQuery, searchMode, searchCaseSensitive);
            if (kwFrag) {
              span.textContent = '';
              span.appendChild(kwFrag);
            }
          }
        } else {
          div.classList.remove('search-match');
          delete div.dataset.searchMatch;
          // Restore plain text (remove <mark> tags from previous search)
          if (span && span.querySelector('mark.search-kw')) {
            span.textContent = text;
          }
        }
      }
      index = end;
      if (index < total && !cancelled) {
        requestAnimationFrame(processBatch);
      }
    };

    requestAnimationFrame(processBatch);
    return () => { cancelled = true; };
  }, [searchQuery, searchMode, searchCaseSensitive, searchActive]);

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const el = e.currentTarget;
    const atBottom = el.scrollTop + el.clientHeight >= el.scrollHeight - 4;
    isAtBottomRef.current = atBottom;
  }, []);

  // Restore auto-scroll when search is closed
  useEffect(() => {
    if (!searchActive && scrollableRef.current) {
      const el = scrollableRef.current;
      const atBottom = el.scrollTop + el.clientHeight >= el.scrollHeight - 50;
      if (atBottom || isAtBottomRef.current) {
        isAtBottomRef.current = true;
        requestAnimationFrame(() => {
          if (scrollableRef.current) {
            scrollableRef.current.scrollTop = scrollableRef.current.scrollHeight;
          }
        });
      }
    }
  }, [searchActive]);

  if (lines.length === 0) {
    return <div ref={scrollableRef} onContextMenu={onContextMenu} className="h-full" data-view-id={viewId}><div className="text-muted-foreground text-center py-8">{emptyMessage}</div></div>;
  }

  return (
    <div ref={scrollableRef} data-view-id={viewId} className="h-full overflow-y-auto overflow-x-hidden font-mono leading-relaxed" onScroll={handleScroll} onContextMenu={onContextMenu}>
      <div ref={containerRef} />
    </div>
  );
}

interface LogPanelProps {
  isConnected: boolean;
  logs: string[];
  onClear: () => void;
  onSave: () => void;
  autoSaveEnabled?: boolean;
  onAutoSaveToggle?: () => void;
  selectedPort?: string;
  availablePorts?: string[];
  connectedPorts?: string[];
  disabledPorts?: string[];
  onPortChange?: (port: string) => void;
  onRefreshPorts?: () => void;
  filterEnabled?: boolean;
  filterRules?: FilterRule[];
  highlightEnabled?: boolean;
  highlightRules?: HighlightRule[];
  portTypes?: Record<string, string>;
  portDescriptions?: Record<string, string>;
  autoSaveFilePath?: string;
  onRevealFile?: (path: string) => void;
  onExportHci?: () => void;
  fontSize?: 'xs' | 'sm' | 'base';
  logBaseIndex?: number;
  filteredLogsOverride?: string[];
  filteredIndicesOverride?: number[];
  onLoadLogWindow?: (panelIndex: 0 | 1, centerLine: number) => Promise<boolean>;
  // Search props
  panelIndex?: 0 | 1;
  searchQuery: string;
  searchMode: 'fuzzy' | 'plain' | 'regex';
  searchCaseSensitive: boolean;
  view0Active: boolean;
  view1Active: boolean;
  onSearchHere?: (viewId: number, query?: string) => void;
}

export function LogPanel({
  logs,
  onClear,
  onSave,
  autoSaveEnabled = false,
  onAutoSaveToggle,
  selectedPort = '',
  availablePorts = [],
  connectedPorts = [],
  disabledPorts = [],
  onPortChange,
  filterEnabled = false,
  filterRules = [],
  highlightEnabled = false,
  highlightRules = [],
  portTypes = {},
  portDescriptions = {},
  autoSaveFilePath,
  onRevealFile,
  onExportHci,
  fontSize = 'xs',
  logBaseIndex = 0,
  filteredLogsOverride,
  filteredIndicesOverride,
  onLoadLogWindow,
  panelIndex = 0,
  searchQuery,
  searchMode,
  searchCaseSensitive,
  view0Active,
  view1Active,
  onSearchHere,
}: LogPanelProps) {
  const logEndRef = useRef<HTMLDivElement>(null);
  const topEndRef = useRef<HTMLDivElement>(null);
  const bottomEndRef = useRef<HTMLDivElement>(null);
  const topScrollRef = useRef<HTMLDivElement>(null);
  const bottomScrollRef = useRef<HTMLDivElement>(null);
  const splitContainerRef = useRef<HTMLDivElement>(null);
  const isSyncingRef = useRef(false);
  const [displayPort, setDisplayPort] = useState(selectedPort);
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const [splitRatio, setSplitRatio] = useState(0.5);
  const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number } | null>(null);
  const ctxContainerRef = useRef<HTMLElement | null>(null);
  const ctxLineRef = useRef('');
  const ctxViewIdRef = useRef(0);
  const ctxSavedSelectionRef = useRef('');
  const ctxOriginalIndexRef = useRef<string | null>(null);
  // Pauses rAF auto-follow-to-bottom on the main view while a "定位日志" scroll+flash is in flight.
  const pauseAutoFollowRef = useRef(false);
  const locateTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [protocolTooltip, setProtocolTooltip] = useState<{
    x: number;
    y: number;
    data: ParsedProtocol;
  } | null>(null);

  useEffect(() => {
    const handleClick = () => setCtxMenu(null);
    if (ctxMenu) {
      document.addEventListener('click', handleClick);
      return () => document.removeEventListener('click', handleClick);
    }
  }, [ctxMenu]);

  const makeContextMenuHandler = (viewId: number) => (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    ctxSavedSelectionRef.current = window.getSelection()?.toString() || '';
    ctxContainerRef.current = e.currentTarget as HTMLElement;
    const lineEl = (e.target as HTMLElement).closest('[data-original-text]') as HTMLElement | null;
    ctxLineRef.current = lineEl?.textContent || '';
    ctxOriginalIndexRef.current = lineEl?.dataset.originalIndex ?? null;
    ctxViewIdRef.current = viewId;
    const menuW = 180;
    const menuH = 160;
    let x = e.clientX;
    let y = e.clientY;
    if (y + menuH > window.innerHeight) {
      y = e.clientY - menuH;
    }
    if (y < 0) y = 0;
    if (x + menuW > window.innerWidth) {
      x = window.innerWidth - menuW;
    }
    setCtxMenu({ x, y });
  };

  const handleCtxCopy = () => {
    setCtxMenu(null);
    const selText = ctxSavedSelectionRef.current;
    if (selText) {
      navigator.clipboard.writeText(selText).catch(() => {});
    } else if (ctxLineRef.current) {
      navigator.clipboard.writeText(ctxLineRef.current).catch(() => {});
    }
  };

  const handleCtxSelectAll = () => {
    const container = ctxContainerRef.current;
    if (!container) return;
    const range = document.createRange();
    range.selectNodeContents(container);
    const sel = window.getSelection();
    if (sel) {
      sel.removeAllRanges();
      sel.addRange(range);
    }
  };

  const handleCtxSearchHere = () => {
    setCtxMenu(null);
    const selectedText = ctxSavedSelectionRef.current;
    onSearchHere?.(ctxViewIdRef.current, selectedText || undefined);
  };

  // Scroll the main (upper) view to the row whose original buffer index == `index`,
  // then flash it twice. Mirrors the DOM pattern in App.tsx handleSearchNavigate.
  const locateLogInMainView = useCallback((viewId: string, index: number) => {
    const scrollableDiv = document.querySelector(`[data-view-id="${viewId}"]`) as HTMLElement | null;
    if (!scrollableDiv) return;
    const logContainer = scrollableDiv.querySelector(':scope > div');
    if (!logContainer) return;

    const findTarget = () => {
      return logContainer.querySelector(`[data-original-index="${index}"]`) as HTMLElement | null;
    };

    const tryLocate = async (attempt: number, loadedHistory: boolean) => {
      const target = findTarget();
      if (!target) {
        // rAF flush may lag behind the just-grown logs buffer; retry a few frames.
        if (attempt < 10) {
          requestAnimationFrame(() => { void tryLocate(attempt + 1, loadedHistory); });
          return;
        }
        if (!loadedHistory && onLoadLogWindow) {
          const mainPanelIndex = viewId === 'view2' ? 1 : 0;
          const loaded = await onLoadLogWindow(mainPanelIndex, index);
          if (loaded) {
            requestAnimationFrame(() => { void tryLocate(0, true); });
          }
        }
        return;
      }

      // Cancel any in-flight resume so a rapid re-locate doesn't prematurely
      // re-enable auto-follow (and snap to bottom) mid-flash.
      if (locateTimeoutRef.current) clearTimeout(locateTimeoutRef.current);
      // Pause auto-follow BEFORE smooth-scrolling: scrollTo is async, so the rAF
      // loop would otherwise see the stale isAtBottomRef=true and jump to bottom.
      pauseAutoFollowRef.current = true;

      // Clear prior flash, then restart the animation (reflow trick for re-locate).
      document.querySelectorAll('.locate-flash').forEach(el => el.classList.remove('locate-flash'));
      target.classList.remove('locate-flash');
      void target.offsetWidth;

      // Center the target row in the scroll viewport.
      const containerRect = scrollableDiv.getBoundingClientRect();
      const targetRect = target.getBoundingClientRect();
      const targetTopInContainer = targetRect.top - containerRect.top + scrollableDiv.scrollTop;
      const desiredScrollTop = targetTopInContainer - scrollableDiv.clientHeight / 2 + target.offsetHeight / 2;
      scrollableDiv.scrollTo({ top: Math.max(0, desiredScrollTop), behavior: 'smooth' });

      target.classList.add('locate-flash');
      const onEnd = () => target.classList.remove('locate-flash');
      target.addEventListener('animationend', onEnd, { once: true });

      // Safety: resume auto-follow once the flash is done (~1.3s anim + buffer).
      locateTimeoutRef.current = setTimeout(() => {
        pauseAutoFollowRef.current = false;
        target.classList.remove('locate-flash');
        target.removeEventListener('animationend', onEnd);
      }, 1500);
    };

    void tryLocate(0, false);
  }, [onLoadLogWindow]);

  const handleCtxLocateLog = () => {
    setCtxMenu(null);
    const filterViewId = ctxViewIdRef.current;
    if (filterViewId % 2 !== 1) return; // only the filter (lower) view offers locate
    const idxStr = ctxOriginalIndexRef.current;
    if (idxStr == null) return;
    const targetIndex = parseInt(idxStr, 10);
    if (Number.isNaN(targetIndex)) return;
    const mainViewId = filterViewId - 1; // view1→view0, view3→view2
    locateLogInMainView(`view${mainViewId}`, targetIndex);
  };

  const handleCtxSave = () => {
    setCtxMenu(null);
    onSave();
  };

  const handleCtxExportHci = () => {
    setCtxMenu(null);
    onExportHci?.();
  };

  const handleCtxWhatIsThis = async () => {
    const line = ctxLineRef.current;
    const menuPos = ctxMenu ? { ...ctxMenu } : { x: 0, y: 0 };
    setCtxMenu(null);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const parsed = await invoke<ParsedProtocol>('parse_protocol_line', { line });
      setProtocolTooltip({ x: menuPos.x, y: menuPos.y, data: parsed });
    } catch (e) {
      console.error('Parse protocol error:', e);
      setProtocolTooltip({
        x: menuPos.x,
        y: menuPos.y,
        data: {
          protocol: '???',
          name: '???',
          opcode_info: '',
          recognized: false,
          fields: [],
        },
      });
    }
  };

  const getPortIcon = (port: string) => {
    const type = portTypes[port] || 'unknown';
    const desc = (portDescriptions[port] || '').toLowerCase();
    if (type === 'bluetooth' || desc.includes('bluetooth') || desc.includes('bth')) {
      return <Bluetooth className="w-3.5 h-3.5 text-blue-500 shrink-0" />;
    }
    if (type === 'usb') {
      return <Usb className="w-3.5 h-3.5 text-muted-foreground shrink-0" />;
    }
    return <Cable className="w-3.5 h-3.5 text-muted-foreground shrink-0" />;
  };

  const hasSplit = filterEnabled && filterRules.length > 0;

  const filteredEntries = useMemo(() => {
    if (filteredLogsOverride && filteredIndicesOverride) {
      return filteredLogsOverride.map((text, i) => ({
        text,
        idx: filteredIndicesOverride[i],
      }));
    }
    if (!hasSplit) return [];
    const out: { text: string; idx: number }[] = [];
    logs.forEach((line, idx) => {
      if (filterRules.some(rule => {
        try {
          if (rule.matchType === 'regex') {
            return new RegExp(rule.keyword).test(line);
          }
          return line.includes(rule.keyword);
        } catch {
          return false;
        }
      })) {
        out.push({ text: line, idx: logBaseIndex + idx });
      }
    });
    return out;
  }, [logs, hasSplit, filterRules, logBaseIndex, filteredLogsOverride, filteredIndicesOverride]);
  const filteredLogs = useMemo(() => filteredEntries.map(e => e.text), [filteredEntries]);
  const filteredIndices = useMemo(() => filteredEntries.map(e => e.idx), [filteredEntries]);
  const mainOriginalIndices = useMemo(
    () => logs.map((_, idx) => logBaseIndex + idx),
    [logs, logBaseIndex]
  );

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  useEffect(() => {
    if (hasSplit) {
      topEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, hasSplit]);

  useEffect(() => {
    if (hasSplit) {
      bottomEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [filteredLogs, hasSplit]);

  useEffect(() => {
    setDisplayPort(selectedPort);
  }, [selectedPort]);

  // @ts-ignore - kept for reference
  const handleSyncScroll = useCallback((source: 'top' | 'bottom') => {
    if (isSyncingRef.current) return;
    const top = topScrollRef.current;
    const bottom = bottomScrollRef.current;
    if (!top || !bottom) return;
    isSyncingRef.current = true;

    const src = source === 'top' ? top : bottom;
    const dst = source === 'top' ? bottom : top;
    const ratio = src.scrollTop / Math.max(1, src.scrollHeight - src.clientHeight);
    dst.scrollTop = ratio * (dst.scrollHeight - dst.clientHeight);

    requestAnimationFrame(() => { isSyncingRef.current = false; });
  }, []);

  const handleDividerMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    const container = splitContainerRef.current;
    if (!container) return;
    const rect = container.getBoundingClientRect();

    const onMove = (e: MouseEvent) => {
      const offsetY = e.clientY - rect.top;
      const ratio = Math.max(0.15, Math.min(0.85, offsetY / rect.height));
      setSplitRatio(ratio);
    };

    const onUp = () => {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      document.body.style.cursor = '';
    };

    document.body.style.cursor = 'row-resize';
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }, []);

  const isPortConnected = displayPort && connectedPorts.includes(displayPort);

  const handleSelectPort = (port: string) => {
    if (selectedPort === port) {
      // Click already-selected port → deselect
      setDisplayPort('');
      onPortChange?.('');
      setIsOpen(false);
      return;
    }
    setDisplayPort(port);
    onPortChange?.(port);
    setIsOpen(false);
  };

  const viewIdBase = panelIndex * 2;

  return (<>
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="acrylic-bar relative h-full flex flex-col rounded-xl overflow-hidden"
      style={{
        background: 'var(--acrylic-tint)',
        backdropFilter: 'blur(40px) saturate(150%)',
        WebkitBackdropFilter: 'blur(40px) saturate(150%)',
        boxShadow: 'var(--shadow-md)'
      }}
    >
      
      {/* Toolbar */}
      <div className="flex items-center gap-2 p-3 pb-2 relative z-10">
        {/* Port selector */}
        <div className="relative" ref={dropdownRef}>
          <button
            onClick={() => setIsOpen(!isOpen)}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium bg-accent/30 hover:bg-accent/50 transition-colors border border-border/30"
          >
            {displayPort ? (
              <>
                <span className="flex items-center gap-1.5">
                  {getPortIcon(displayPort)}
                  <span>{displayPort}</span>
                </span>
                {isPortConnected && <Circle className="w-2 h-2 fill-green-500 text-green-500" />}
              </>
            ) : (
              <span className="text-muted-foreground">选择端口</span>
            )}
            <ChevronDown className={`w-3 h-3 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
          </button>
          {isOpen && (
            <div className="absolute top-full left-0 mt-1 min-w-[180px] rounded-xl overflow-hidden z-50"
              style={{
                background: 'var(--acrylic-tint)',
                backdropFilter: 'blur(40px) saturate(150%)',
                WebkitBackdropFilter: 'blur(40px) saturate(150%)',
                border: '1px solid var(--acrylic-border)',
                boxShadow: 'var(--shadow-lg)'
              }}
            >
              {availablePorts.length === 0 ? (
                <div className="px-3 py-2 text-xs text-muted-foreground">无可用端口</div>
              ) : (
                availablePorts.map(port => (
                  <button
                    key={port}
                    onClick={() => handleSelectPort(port)}
                    disabled={disabledPorts.includes(port)}
                    className={`w-full flex items-center gap-2 px-3 py-2 text-xs transition-colors
                      ${selectedPort === port ? 'bg-accent/40 text-foreground' : 'text-muted-foreground hover:bg-accent/20'}
                      ${disabledPorts.includes(port) ? 'opacity-40 cursor-not-allowed' : 'cursor-pointer'}`}
                  >
                    {getPortIcon(port)}
                    <span className="flex-1 text-left">{port}</span>
                    {connectedPorts.includes(port) && <Circle className="w-2 h-2 fill-green-500 text-green-500 shrink-0" />}
                    {disabledPorts.includes(port) && <span className="text-[10px] text-muted-foreground">使用中</span>}
                  </button>
                ))
              )}
            </div>
          )}
        </div>

        {autoSaveEnabled && autoSaveFilePath ? (
          <span
            className="flex-1 text-center text-xs text-accent-foreground cursor-pointer hover:underline truncate px-4"
            onClick={() => onRevealFile?.(autoSaveFilePath)}
            title={autoSaveFilePath}
          >
            {autoSaveFilePath.split('\\').pop()}
          </span>
        ) : (
          <div className="flex-1" />
        )}

        <button
          onClick={onClear}
          title="清空日志"
          className="p-2 rounded-lg text-muted-foreground hover:bg-muted/50 transition-colors flex items-center gap-1.5"
        >
          <Trash2 className="w-4 h-4" />
          <span>清空</span>
        </button>

        {onAutoSaveToggle && (
          <button
            onClick={onAutoSaveToggle}
            className={`px-2.5 py-1.5 rounded-lg text-xs font-medium transition-colors flex items-center gap-1.5
              ${autoSaveEnabled
                ? 'bg-accent text-accent-foreground border border-accent-foreground/30'
                : 'text-muted-foreground hover:bg-muted/50'}`}
            title={autoSaveEnabled ? '关闭自动保存' : '开启自动保存'}
          >
            <SaveAll className="w-3.5 h-3.5" />
            自动保存
          </button>
        )}
      </div>

      {hasSplit ? (
        <div className="flex-1 min-h-0 flex flex-col p-3 pt-2" ref={splitContainerRef}>
          <div
            className="min-h-0 rounded-lg bg-white dark:bg-white/[0.03]"
            style={{ flex: splitRatio }}
          >
            <DomLogList lines={logs} emptyMessage="等待日志..." onContextMenu={makeContextMenuHandler(viewIdBase)} highlightEnabled={highlightEnabled} highlightRules={highlightRules} fontSize={fontSize} viewId={`view${viewIdBase}`} searchQuery={searchQuery} searchMode={searchMode} searchCaseSensitive={searchCaseSensitive} searchActive={searchQuery !== '' && view0Active} originalIndices={mainOriginalIndices} pauseAutoFollowRef={pauseAutoFollowRef} />
          </div>
          <div
            className="flex items-center justify-center cursor-row-resize shrink-0 py-1 group"
            onMouseDown={handleDividerMouseDown}
          >
            <div className="h-1 w-8 rounded-full bg-border dark:bg-border/50 group-hover:bg-accent/70 group-hover:h-1.5 transition-all duration-200" />
          </div>
          <div
            className="min-h-0 rounded-lg bg-white dark:bg-white/[0.03]"
            style={{ flex: 1 - splitRatio }}
          >
            <DomLogList lines={filteredLogs} emptyMessage={filterRules.length > 0 ? '无匹配日志' : '请添加过滤规则'} onContextMenu={makeContextMenuHandler(viewIdBase + 1)} highlightEnabled={highlightEnabled} highlightRules={highlightRules} fontSize={fontSize} viewId={`view${viewIdBase + 1}`} searchQuery={searchQuery} searchMode={searchMode} searchCaseSensitive={searchCaseSensitive} searchActive={searchQuery !== '' && view1Active && hasSplit} originalIndices={filteredIndices} pauseAutoFollowRef={pauseAutoFollowRef} />
          </div>
        </div>
      ) : (
        <div className="flex-1 min-h-0">
          <DomLogList lines={logs} emptyMessage="等待日志..." onContextMenu={makeContextMenuHandler(viewIdBase)} highlightEnabled={highlightEnabled} highlightRules={highlightRules} fontSize={fontSize} viewId={`view${viewIdBase}`} searchQuery={searchQuery} searchMode={searchMode} searchCaseSensitive={searchCaseSensitive} searchActive={searchQuery !== '' && view0Active} originalIndices={mainOriginalIndices} pauseAutoFollowRef={pauseAutoFollowRef} />
        </div>
      )}

    </motion.div>

    {/* Right-click context menu */}
    {ctxMenu && (() => {
      const isDark = document.documentElement.classList.contains('dark');
      return (
      <div
        className="fixed z-[9999] min-w-[160px] rounded-xl overflow-hidden"
        style={{
          left: ctxMenu.x,
          top: ctxMenu.y,
          background: isDark ? 'rgba(44, 44, 44, 0.7)' : 'rgba(252, 252, 252, 0.65)',
          backdropFilter: 'blur(60px) saturate(180%)',
          WebkitBackdropFilter: 'blur(60px) saturate(180%)',
          border: isDark ? '1px solid rgba(255, 255, 255, 0.08)' : '1px solid rgba(255, 255, 255, 0.7)',
          boxShadow: isDark ? '0 8px 32px rgba(0, 0, 0, 0.4)' : '0 8px 32px rgba(0, 0, 0, 0.12)'
        }}
        onContextMenu={e => e.preventDefault()}
      >
        <div
          onClick={handleCtxCopy}
          className="px-4 py-2.5 text-sm cursor-pointer hover:bg-black/10 dark:hover:bg-white/10 transition-colors flex items-center gap-2"
        >
          <Copy className="w-4 h-4" />
          复制
        </div>
        <div
          onClick={handleCtxSelectAll}
          className="px-4 py-2.5 text-sm cursor-pointer hover:bg-black/10 dark:hover:bg-white/10 transition-colors flex items-center gap-2"
        >
          <Scan className="w-4 h-4" />
          全选
        </div>
        <div
          onClick={handleCtxSearchHere}
          className="px-4 py-2.5 text-sm cursor-pointer hover:bg-black/10 dark:hover:bg-white/10 transition-colors flex items-center gap-2"
        >
          <SearchIcon className="w-4 h-4" />
          搜索
        </div>
        {ctxViewIdRef.current % 2 === 1 && ctxOriginalIndexRef.current != null && (
          <div
            onClick={handleCtxLocateLog}
            className="px-4 py-2.5 text-sm cursor-pointer hover:bg-black/10 dark:hover:bg-white/10 transition-colors flex items-center gap-2"
          >
            <LocateFixed className="w-4 h-4" />
            定位日志
          </div>
        )}
        <div
          onClick={handleCtxSave}
          className="px-4 py-2.5 text-sm cursor-pointer hover:bg-black/10 dark:hover:bg-white/10 transition-colors flex items-center gap-2"
        >
          <SaveAll className="w-4 h-4" />
          保存日志
        </div>
        <div
          onClick={handleCtxExportHci}
          className="px-4 py-2.5 text-sm cursor-pointer hover:bg-black/10 dark:hover:bg-white/10 transition-colors flex items-center gap-2"
        >
          <Bluetooth className="w-4 h-4" />
          导出HCI日志
        </div>
        <div className="h-px bg-border/20 mx-2 my-1" />
        <div
          onClick={handleCtxWhatIsThis}
          className="px-4 py-2.5 text-sm cursor-pointer hover:bg-black/10 dark:hover:bg-white/10 transition-colors flex items-center gap-2"
        >
          <HelpCircle className="w-4 h-4" />
          这是什么
        </div>
      </div>
      );
    })()}

    {protocolTooltip && (
      <ProtocolTooltip
        data={protocolTooltip.data}
        x={protocolTooltip.x}
        y={protocolTooltip.y}
        onClose={() => setProtocolTooltip(null)}
      />
    )}
  </>);
}
