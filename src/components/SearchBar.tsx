import { useState, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { Search, ChevronUp, ChevronDown } from 'lucide-react';

export interface SearchScope {
  view0: boolean;
  view1: boolean;
  view2: boolean;
  view3: boolean;
}

export interface SearchResult {
  matches: { line_index: number; score: number; line_text: string; highlights: { start: number; end: number }[] }[];
  total_count: number;
  query: string;
  elapsed_ms: number;
}

interface SearchBarProps {
  query: string;
  onQueryChange: (q: string) => void;
  mode: 'fuzzy' | 'plain' | 'regex';
  onModeChange: (m: 'fuzzy' | 'plain' | 'regex') => void;
  caseSensitive: boolean;
  onCaseChange: (c: boolean) => void;
  scope: SearchScope;
  onScopeChange: (s: SearchScope) => void;
  onClose: () => void;
  dualPanelMode: boolean;
  searchResult?: SearchResult | null;
  currentMatchIndex?: number;
  onNavigate?: (index: number) => void;
  onSearchNow?: () => void;
}

const MODE_CONFIG = {
  plain:  { label: 'NOR', tooltip: '常规模式',     showCase: true  },
  fuzzy:  { label: 'FUZ', tooltip: '模糊搜索，智能大小写', showCase: false },
  regex:  { label: '\\.*', tooltip: '正则模式',     showCase: true  },
} as const;

const MODE_CYCLE: Record<string, 'fuzzy' | 'plain' | 'regex'> = {
  plain: 'fuzzy',
  fuzzy: 'regex',
  regex: 'plain',
};

export function SearchBar({
  query,
  onQueryChange,
  mode,
  onModeChange,
  caseSensitive,
  onCaseChange,
  scope,
  onScopeChange,
  onClose,
  dualPanelMode,
  searchResult,
  currentMatchIndex = 0,
  onNavigate,
  onSearchNow,
}: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const [scopeOpen, setScopeOpen] = useState(false);
  const scopeRef = useRef<HTMLDivElement>(null);
  const scopeBtnRef = useRef<HTMLButtonElement>(null);
  const popupRef = useRef<HTMLDivElement>(null);
  const [scopePopupPos, setScopePopupPos] = useState<{ left: number; top: number } | null>(null);

  const [toastMsg, setToastMsg] = useState<string | null>(null);
  const toastTimerRef = useRef<ReturnType<typeof setTimeout>>();

  const showToast = (msg: string) => {
    clearTimeout(toastTimerRef.current);
    setToastMsg(msg);
    toastTimerRef.current = setTimeout(() => setToastMsg(null), 1000);
  };

  useEffect(() => {
    inputRef.current?.select();
  }, []);

  const matchCount = searchResult?.total_count ?? 0;
  const displayIndex = currentMatchIndex >= 0 ? currentMatchIndex : 0;

  const doNavigate = (toIdx: number) => {
    onNavigate?.(toIdx);
  };

  // Close search bar on outside click (but not when clicking the scope popup)
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (barRef.current?.contains(e.target as Node)) return;
      if (popupRef.current?.contains(e.target as Node)) return;
      onClose();
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [onClose]);

  // Close scope popup on outside click
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (!scopeOpen) return;
      const target = e.target as Node;
      const btn = scopeBtnRef.current;
      const popup = popupRef.current;
      if (btn && !btn.contains(target) && popup && !popup.contains(target)) {
        setScopeOpen(false);
      }
    };
    if (scopeOpen) {
      document.addEventListener('mousedown', handleClick);
      return () => document.removeEventListener('mousedown', handleClick);
    }
  }, [scopeOpen]);

  // Calculate popup fixed position when opening
  useEffect(() => {
    if (scopeOpen && scopeBtnRef.current) {
      const rect = scopeBtnRef.current.getBoundingClientRect();
      setScopePopupPos({ left: rect.left + rect.width / 2, top: rect.bottom });
    }
  }, [scopeOpen]);

  // Keyboard handlers
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
        return;
      }
      if (e.key === 'Enter') {
        e.preventDefault();
        // If fuzzy mode and < 3 chars, force search now
        if (mode === 'fuzzy' && query.length > 0 && query.length < 3 && onSearchNow) {
          onSearchNow();
          return;
        }
        if (matchCount === 0) return;
        if (e.shiftKey) {
          doNavigate(displayIndex <= 0 ? matchCount - 1 : displayIndex - 1);
        } else {
          doNavigate(displayIndex >= matchCount - 1 ? 0 : displayIndex + 1);
        }
        return;
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (matchCount === 0) return;
        const next = Math.min(matchCount - 1, displayIndex + 1);
        if (next === displayIndex) {
          showToast('已到底部');
          return;
        }
        doNavigate(next);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (matchCount === 0) return;
        const next = Math.max(0, displayIndex - 1);
        if (next === displayIndex) {
          showToast('已到顶部');
          return;
        }
        doNavigate(next);
        return;
      }
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [onClose, matchCount, displayIndex, onNavigate, mode, query, onSearchNow]);

  const handlePrev = () => {
    if (matchCount === 0) return;
    doNavigate(displayIndex <= 0 ? matchCount - 1 : displayIndex - 1);
  };

  const handleNext = () => {
    if (matchCount === 0) return;
    doNavigate(displayIndex >= matchCount - 1 ? 0 : displayIndex + 1);
  };

  const toggleScope = (key: keyof SearchScope) => {
    onScopeChange({ ...scope, [key]: !scope[key] });
  };

  const hasFilter = true;
  const isDark = document.documentElement.classList.contains('dark');
  const cfg = MODE_CONFIG[mode];
  const cycleMode = () => onModeChange(MODE_CYCLE[mode]);

  return (
    <div ref={barRef} style={{ minWidth: '720px' }} className="mx-auto">
      <div
        className="flex items-center gap-1.5 px-3 py-2 rounded-xl"
        style={{
          background: isDark ? 'rgba(44,44,44,0.5)' : 'rgba(252,252,252,0.45)',
          backdropFilter: 'blur(60px) saturate(180%)',
          WebkitBackdropFilter: 'blur(60px) saturate(180%)',
          border: '1px solid var(--acrylic-border)',
          boxShadow: 'var(--shadow-lg)'
        }}
      >
        <Search className="w-4 h-4 text-muted-foreground shrink-0" />

        <input
          ref={inputRef}
          value={query}
          onChange={e => onQueryChange(e.target.value)}
          placeholder="搜索日志..."
          className="flex-1 bg-transparent border-none outline-none text-sm text-foreground placeholder:text-muted-foreground/50 min-w-0"
        />

        <div className="flex items-center gap-1 shrink-0">
          {/* Case sensitive toggle — invisible placeholder in fuzzy mode to keep layout stable */}
          <div className="w-[40px] flex justify-center">
            {cfg.showCase && (
              <button
                onClick={() => onCaseChange(!caseSensitive)}
                className={`px-2 py-0.5 rounded-md text-xs font-mono font-bold tracking-wider transition-all ${
                  caseSensitive
                    ? 'bg-accent/40 text-accent-foreground'
                    : 'text-muted-foreground hover:bg-accent/20'
                }`}
                title={caseSensitive ? '大小写匹配' : '忽略大小写'}
              >
                {caseSensitive ? 'ABC' : 'abc'}
              </button>
            )}
          </div>

          {/* Mode toggle button */}
          <button
            onClick={cycleMode}
            className="px-2 py-0.5 rounded-md text-xs font-mono font-bold tracking-wider transition-all bg-accent/40 text-accent-foreground hover:scale-110"
            title={cfg.tooltip}
          >
            {cfg.label}
          </button>

          {/* Scope selector */}
          <div className="relative" ref={scopeRef}>
            <button
              ref={scopeBtnRef}
              onClick={() => setScopeOpen(!scopeOpen)}
              className="p-1.5 rounded-md text-muted-foreground hover:bg-accent/20 hover:scale-110 transition-all"
              title="搜索范围"
            >
              <svg width="14" height="14" viewBox="0 0 14 14">
                <rect x="1" y="1" width="5" height="5" rx="0.8" fill="currentColor" opacity={scope.view0 ? 1 : 0.2} />
                <rect x="8" y="1" width="5" height="5" rx="0.8" fill="currentColor" opacity={dualPanelMode && scope.view2 ? 1 : 0.08} />
                <rect x="1" y="8" width="5" height="5" rx="0.8" fill="currentColor" opacity={scope.view1 ? 1 : 0.2} />
                <rect x="8" y="8" width="5" height="5" rx="0.8" fill="currentColor" opacity={dualPanelMode && scope.view3 ? 1 : 0.08} />
              </svg>
            </button>

            {scopeOpen && scopePopupPos && createPortal(
              <div
                ref={popupRef}
                className="fixed z-[9999] rounded-lg"
                style={{
                  left: scopePopupPos.left,
                  top: scopePopupPos.top,
                  transform: 'translateX(-50%)',
                  marginTop: '4px',
                  background: 'var(--acrylic-tint)',
                  backdropFilter: 'blur(60px) saturate(180%)',
                  WebkitBackdropFilter: 'blur(60px) saturate(180%)',
                  border: '1px solid var(--acrylic-border)',
                  boxShadow: 'var(--shadow-lg)'
                }}
              >
                <div className="grid grid-cols-2 gap-0.5 p-0.5">
                  <ScopeCell active={scope.view0} disabled={false} onClick={() => toggleScope('view0')} />
                  <ScopeCell active={scope.view2} disabled={!dualPanelMode} onClick={() => dualPanelMode && toggleScope('view2')} />
                  <ScopeCell active={scope.view1} disabled={!hasFilter} onClick={() => toggleScope('view1')} />
                  <ScopeCell active={scope.view3} disabled={!dualPanelMode || !hasFilter} onClick={() => dualPanelMode && hasFilter && toggleScope('view3')} />
                </div>
              </div>,
              document.body
            )}
          </div>
        </div>

        {/* Match count & navigation */}
        <div className="flex items-center gap-1 shrink-0 text-xs text-muted-foreground">
          {query && (
            <>
              <span className="min-w-[8ch] text-right tabular-nums">
                {matchCount > 0
                  ? `${displayIndex + 1}/${matchCount}`
                  : searchResult ? '0' : '...'
                }
              </span>
              {searchResult && (
                <span className="text-[10px] text-muted-foreground/50">
                  {searchResult.elapsed_ms.toFixed(1)}ms
                </span>
              )}
              <button
                onClick={handlePrev}
                disabled={matchCount === 0}
                className="p-1 rounded text-muted-foreground hover:bg-accent/20 hover:scale-125 transition-all disabled:opacity-30 disabled:cursor-not-allowed"
                title="上一个匹配 (↑)"
              >
                <ChevronUp className="w-5 h-5" />
              </button>
              <button
                onClick={handleNext}
                disabled={matchCount === 0}
                className="p-1 rounded text-muted-foreground hover:bg-accent/20 hover:scale-125 transition-all disabled:opacity-30 disabled:cursor-not-allowed"
                title="下一个匹配 (↓)"
              >
                <ChevronDown className="w-5 h-5" />
              </button>
            </>
          )}
        </div>
      </div>

      {toastMsg && (
        <div className="mt-1.5 text-center">
          <span className="inline-block px-3 py-1.5 text-sm rounded-md bg-background/80 text-muted-foreground backdrop-blur-sm border border-border/30">
            {toastMsg}
          </span>
        </div>
      )}
    </div>
  );
}

function ScopeCell({ active, disabled, onClick }: {
  active: boolean;
  disabled: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`w-6 h-6 rounded border-2 transition-all ${
        disabled
          ? 'opacity-20 cursor-not-allowed border-border/30'
          : active
            ? 'bg-primary/40 dark:bg-white/[0.28] cursor-pointer'
            : 'bg-black/[0.06] dark:bg-white/[0.06] border-border/50 hover:bg-accent/30 cursor-pointer'
      }`}
    />
  );
}
