import { motion, AnimatePresence } from 'motion/react';
import { ChevronRight, Settings, FolderOpen, Type, HardDrive, Timer, Globe, Bluetooth, Usb, Cable } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';

interface LeftSideMenuProps {
  isOpen: boolean;
  onClose: () => void;
  timestampEnabled: boolean;
  onTimestampToggle: () => void;
  filterEnabled: boolean;
  onFilterToggle: () => void;
  onFilterSettings: () => void;
  highlightEnabled: boolean;
  onHighlightToggle: () => void;
  onHighlightSettings: () => void;
  logSaveDir?: string;
  onSelectLogDir?: () => void;
  onHciExtract?: () => void;
  /** 拖放日志文件到「HCI日志提取」后触发，参数为本地绝对路径 */
  onHciExtractFromPath?: (path: string) => void;
  fontSize?: 'xs' | 'sm' | 'base';
  onFontSizeChange?: (size: 'xs' | 'sm' | 'base') => void;
  maxCacheKb?: number;
  onMaxCacheKbChange?: (kb: number) => void;
  pollIntervalMs?: number;
  onPollIntervalMsChange?: (ms: number) => void;
  hciTimezoneOffset?: number;
  onHciTimezoneOffsetChange?: (offset: number) => void;
  liveImportEnabled?: boolean;
  liveImportStatus?: 'idle' | 'connecting' | 'ready' | 'active' | 'error';
  liveImportStats?: { total: number; ok: number; err: number; last_hr: number; last_err_msg: string };
  onLiveImportToggle?: () => void;
  wpsPath?: string;
  onSelectWpsDir?: () => void;
  connectedPorts?: string[];
  liveImportSelectedPort?: string;
  onLiveImportSelectPort?: (port: string) => void;
  onOpenLc3ToolKit?: () => void;
  onOpenUsbAudio?: () => void;
  portTypes?: Record<string, string>;
  portDescriptions?: Record<string, string>;
}

const menuItems = [
  {
    title: '日志',
    items: ['保存路径', '时间戳', '日志过滤', '关键词高亮', 'HCI日志提取', 'HCI LIVE']
  },
  {
    title: '工具',
    items: ['反汇编导出', '内存统计', '异常分析', 'LC3 Tool Kit', 'USB日志音频提取']
  },
  {
    title: '设置',
    items: ['字体', '缓存行数', '渲染间隔', '日志时区']
  },
  {
    title: '其他',
    items: []
  }
];

export function LeftSideMenu({
  isOpen,
  onClose,
  timestampEnabled,
  onTimestampToggle,
  filterEnabled,
  onFilterToggle,
  onFilterSettings,
  highlightEnabled,
  onHighlightToggle,
  onHighlightSettings,
  logSaveDir,
  onSelectLogDir,
  onHciExtract,
  onHciExtractFromPath,
  fontSize = 'xs',
  onFontSizeChange,
    maxCacheKb = 50,
    onMaxCacheKbChange,
    pollIntervalMs = 50,
    onPollIntervalMsChange,
    hciTimezoneOffset = 0,
    onHciTimezoneOffsetChange,
    liveImportEnabled,
    liveImportStatus = 'idle',
    liveImportStats,
    onLiveImportToggle,
  wpsPath,
  onSelectWpsDir,
  connectedPorts,
  liveImportSelectedPort,
  onLiveImportSelectPort,
  onOpenLc3ToolKit,
  onOpenUsbAudio,
  portTypes = {},
  portDescriptions = {},
}: LeftSideMenuProps) {
  const [expandedSection, setExpandedSection] = useState<string | null>(null);
  /** 日志时区输入草稿：编辑中可为 "8"/"-3"；失焦/回车后规范为 "+8"/"-3"/"0" */
  const [tzDraft, setTzDraft] = useState<string | null>(null);
  /** 是否有文件正拖到「HCI日志提取」上（Tauri 原生拖放） */
  const [hciDropActive, setHciDropActive] = useState(false);
  const hciExtractRef = useRef<HTMLDivElement | null>(null);
  const onHciExtractFromPathRef = useRef(onHciExtractFromPath);
  onHciExtractFromPathRef.current = onHciExtractFromPath;

  // Tauri 默认启用原生 drag-drop，HTML5 onDrop 收不到系统文件；
  // 必须用 getCurrentWindow().onDragDropEvent，并按坐标判断是否落在「HCI日志提取」上。
  useEffect(() => {
    if (!isOpen) {
      setHciDropActive(false);
      return;
    }

    let unlisten: (() => void) | undefined;
    let cancelled = false;

    const pickLogPath = (paths: string[]): string | null => {
      for (const p of paths) {
        const lower = p.toLowerCase();
        if (lower.endsWith('.txt') || lower.endsWith('.log')) return p;
      }
      return paths[0] || null;
    };

    const isOverHciTarget = async (position: { x: number; y: number }) => {
      const el = hciExtractRef.current;
      if (!el) return false;
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const factor = await getCurrentWindow().scaleFactor();
        // Tauri 给出的是物理像素坐标（相对窗口/Webview）
        const x = position.x / factor;
        const y = position.y / factor;
        const rect = el.getBoundingClientRect();
        return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
      } catch {
        // 回退：按 1:1 物理像素（部分环境 scaleFactor 不可用）
        const rect = el.getBoundingClientRect();
        return (
          position.x >= rect.left &&
          position.x <= rect.right &&
          position.y >= rect.top &&
          position.y <= rect.bottom
        );
      }
    };

    (async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        if (cancelled) return;
        unlisten = await getCurrentWindow().onDragDropEvent(async (event) => {
          const payload = event.payload;
          if (payload.type === 'enter' || payload.type === 'over') {
            // 拖入文件时自动展开「日志」分组，方便对准 HCI 提取项
            setExpandedSection((prev) => prev ?? '日志');
            const over = await isOverHciTarget(payload.position);
            setHciDropActive(over);
          } else if (payload.type === 'drop') {
            const over = await isOverHciTarget(payload.position);
            setHciDropActive(false);
            if (!over) return;
            const path = pickLogPath(payload.paths || []);
            onHciExtractFromPathRef.current?.(path || '');
          } else {
            // leave / cancel
            setHciDropActive(false);
          }
        });
      } catch (e) {
        console.error('HCI drag-drop listen failed:', e);
      }
    })();

    return () => {
      cancelled = true;
      setHciDropActive(false);
      if (unlisten) unlisten();
    };
  }, [isOpen]);

  const renderSavePathItem = () => (
    <div className="w-full px-8 py-2 hover:bg-accent/20 transition-colors">
      <div className="flex items-center justify-between">
        <span className="text-base text-muted-foreground">保存路径</span>
        <motion.button
          whileHover={{ scale: 1.15 }}
          whileTap={{ scale: 0.9 }}
          onClick={(e) => { e.stopPropagation(); onSelectLogDir?.(); }}
          className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
        >
          <FolderOpen className="w-4 h-4 text-muted-foreground" />
        </motion.button>
      </div>
      {logSaveDir && (
        <div
          onClick={(e) => { e.stopPropagation(); onSelectLogDir?.(); }}
          className="mt-0.5 text-sm text-muted-foreground/60 truncate cursor-pointer hover:text-foreground/80 transition-colors"
        >
          {logSaveDir}
        </div>
      )}
    </div>
  );

  const renderTimestampItem = () => (
    <div className="w-full px-8 py-2 flex items-center justify-between hover:bg-accent/20 transition-colors">
      <span className="text-base text-muted-foreground">时间戳</span>
      <div
        onClick={(e) => { e.stopPropagation(); onTimestampToggle(); }}
        className={`w-9 h-5 rounded-full cursor-pointer transition-colors relative shrink-0 ${timestampEnabled ? 'bg-primary' : 'bg-muted'}`}
      >
        <div className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-all ${timestampEnabled ? 'left-[18px]' : 'left-0.5'}`} />
      </div>
    </div>
  );

  const renderFilterItem = () => (
    <div className="w-full px-8 py-2 flex items-center justify-between hover:bg-accent/20 transition-colors">
      <span className="text-base text-muted-foreground">日志过滤</span>
      <div className="flex items-center gap-1.5">
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={(e) => { e.stopPropagation(); onFilterSettings(); }}
          className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
        >
          <Settings className="w-4 h-4 text-muted-foreground" />
        </motion.button>
        <div
          onClick={(e) => { e.stopPropagation(); onFilterToggle(); }}
          className={`w-9 h-5 rounded-full cursor-pointer transition-colors relative shrink-0 ${filterEnabled ? 'bg-primary' : 'bg-muted'}`}
        >
          <div className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-all ${filterEnabled ? 'left-[18px]' : 'left-0.5'}`} />
        </div>
      </div>
    </div>
  );

  const renderHighlightItem = () => (
    <div className="w-full px-8 py-2 flex items-center justify-between hover:bg-accent/20 transition-colors">
      <span className="text-base text-muted-foreground">关键词高亮</span>
      <div className="flex items-center gap-1.5">
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={(e) => { e.stopPropagation(); onHighlightSettings(); }}
          className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
        >
          <Settings className="w-4 h-4 text-muted-foreground" />
        </motion.button>
        <div
          onClick={(e) => { e.stopPropagation(); onHighlightToggle(); }}
          className={`w-9 h-5 rounded-full cursor-pointer transition-colors relative shrink-0 ${highlightEnabled ? 'bg-primary' : 'bg-muted'}`}
        >
          <div className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-all ${highlightEnabled ? 'left-[18px]' : 'left-0.5'}`} />
        </div>
      </div>
    </div>
  );

  const fontLevels: { label: string; value: 'xs' | 'sm' | 'base' }[] = [
    { label: '小', value: 'xs' },
    { label: '中', value: 'sm' },
    { label: '大', value: 'base' },
  ];

  const renderFontSizeItem = () => (
    <div className="w-full px-8 py-2 hover:bg-accent/20 transition-colors">
      <div className="flex items-center gap-2 mb-2">
        <Type className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
        <span className="text-base text-muted-foreground">字体</span>
      </div>
      <div className="flex gap-1">
        {fontLevels.map(level => (
          <button
            key={level.value}
            onClick={() => onFontSizeChange?.(level.value)}
            className={`flex-1 py-1 rounded-lg text-sm font-medium transition-colors ${
              fontSize === level.value
                ? 'bg-accent text-accent-foreground border border-accent-foreground/30'
                : 'text-muted-foreground hover:bg-accent/30 border border-transparent'
            }`}
          >
            {level.label}
          </button>
        ))}
      </div>
    </div>
  );

  const renderCacheItem = () => (
    <div className="w-full px-8 py-2 hover:bg-accent/20 transition-colors">
      <div className="flex items-center gap-2 mb-1.5">
        <HardDrive className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
        <span className="text-base text-muted-foreground">缓存行数</span>
      </div>
      <div className="flex items-center gap-2">
        <input
          type="number"
          min={1}
          step={10}
          value={maxCacheKb}
          onChange={e => {
            const val = parseInt(e.target.value);
            if (!isNaN(val) && val >= 1) {
              onMaxCacheKbChange?.(val);
            }
          }}
          className="w-24 px-2 py-1 rounded-lg text-sm bg-input-background border border-border text-foreground focus:outline-none focus:ring-1 focus:ring-primary/50 text-center"
        />
        <span className="text-sm text-muted-foreground">K</span>
      </div>
    </div>
  );

  const pollSpeedMsOptions = [10, 25, 50];

  const renderPollingItem = () => (
    <div className="w-full px-8 py-2 hover:bg-accent/20 transition-colors">
      <div className="flex items-center gap-2 mb-1.5">
        <Timer className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
        <span className="text-base text-muted-foreground">渲染间隔</span>
      </div>
      <div className="flex gap-1">
        {pollSpeedMsOptions.map(ms => (
          <button
            key={ms}
            onClick={() => onPollIntervalMsChange?.(ms)}
            className={`flex-1 py-1 rounded-lg text-sm font-medium transition-colors ${
              pollIntervalMs === ms
                ? 'bg-accent text-accent-foreground border border-accent-foreground/30'
                : 'text-muted-foreground hover:bg-accent/30 border border-transparent'
            }`}
          >
            {`${ms}ms`}
          </button>
        ))}
      </div>
    </div>
  );

  const formatTimezoneOffset = (n: number) => (n > 0 ? `+${n}` : `${n}`);

  const parseTimezoneInput = (raw: string): number | null => {
    const trimmed = raw.trim();
    if (trimmed === '' || trimmed === '+' || trimmed === '-') return null;
    // 无符号或 + 开头 → 正数；显式 - 才为负
    const val = parseInt(trimmed, 10);
    if (isNaN(val) || val < -12 || val > 14) return null;
    return val;
  };

  const commitTimezoneDraft = (raw: string) => {
    const val = parseTimezoneInput(raw);
    if (val !== null) {
      onHciTimezoneOffsetChange?.(val);
    }
    setTzDraft(null);
  };

  const stepTimezone = (delta: number) => {
    const base =
      tzDraft !== null ? (parseTimezoneInput(tzDraft) ?? hciTimezoneOffset) : hciTimezoneOffset;
    const next = Math.min(14, Math.max(-12, base + delta));
    onHciTimezoneOffsetChange?.(next);
    setTzDraft(null);
  };

  const renderHciTimezoneItem = () => {
    const displayValue =
      tzDraft !== null ? tzDraft : formatTimezoneOffset(hciTimezoneOffset);
    return (
      <div className="w-full px-8 py-2 hover:bg-accent/20 transition-colors">
        <div className="flex items-center gap-2 mb-1.5">
          <Globe className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
          <span
            className="text-base text-muted-foreground"
            title="HCI日志导出时区偏移"
          >
            日志时区
          </span>
        </div>
        {/* 布局对齐「缓存行数」：输入框在前，单位在后 */}
        <div className="flex items-center gap-2" title="HCI日志导出时区偏移">
          <div className="group relative w-24">
            <input
              type="text"
              inputMode="numeric"
              value={displayValue}
              onFocus={() => {
                setTzDraft(formatTimezoneOffset(hciTimezoneOffset));
              }}
              onChange={e => {
                const v = e.target.value;
                if (v !== '' && !/^[+-]?\d*$/.test(v)) return;
                setTzDraft(v);
                const val = parseTimezoneInput(v);
                if (val !== null) {
                  onHciTimezoneOffsetChange?.(val);
                }
              }}
              onBlur={e => commitTimezoneDraft(e.target.value)}
              onKeyDown={e => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  commitTimezoneDraft(e.currentTarget.value);
                  e.currentTarget.blur();
                } else if (e.key === 'ArrowUp') {
                  e.preventDefault();
                  stepTimezone(1);
                } else if (e.key === 'ArrowDown') {
                  e.preventDefault();
                  stepTimezone(-1);
                }
              }}
              className="w-full px-2 py-1 pr-5 rounded-lg text-sm bg-input-background border border-border text-foreground focus:outline-none focus:ring-1 focus:ring-primary/50 text-center"
            />
            {/* 对齐 number 输入：默认隐藏，悬停/聚焦输入框时再显示上下箭头 */}
            <div className="absolute right-0.5 top-0 bottom-0 flex flex-col justify-center opacity-0 pointer-events-none group-hover:opacity-100 group-hover:pointer-events-auto group-focus-within:opacity-100 group-focus-within:pointer-events-auto transition-opacity">
              <button
                type="button"
                tabIndex={-1}
                onMouseDown={e => e.preventDefault()}
                onClick={() => stepTimezone(1)}
                className="h-3 w-4 flex items-center justify-center text-muted-foreground hover:text-foreground leading-none"
                aria-label="增加时区"
              >
                <span className="text-[9px]">▲</span>
              </button>
              <button
                type="button"
                tabIndex={-1}
                onMouseDown={e => e.preventDefault()}
                onClick={() => stepTimezone(-1)}
                className="h-3 w-4 flex items-center justify-center text-muted-foreground hover:text-foreground leading-none"
                aria-label="减少时区"
              >
                <span className="text-[9px]">▼</span>
              </button>
            </div>
          </div>
          <span className="text-sm text-muted-foreground">UTC</span>
        </div>
      </div>
    );
  };

  const middleEllipsis = (str: string, maxLen: number) => {
    if (str.length <= maxLen) return str;
    const half = Math.floor((maxLen - 3) / 2);
    return str.slice(0, half) + '...' + str.slice(str.length - half);
  };

  const renderLiveImportItem = () => {
    const isOn = liveImportEnabled;
    let toggleBg = 'bg-muted';
    if (isOn) {
      switch (liveImportStatus) {
        case 'connecting': toggleBg = 'bg-yellow-500 animate-pulse'; break;
        case 'ready': toggleBg = 'bg-green-500'; break;
        case 'active': toggleBg = 'bg-green-500'; break;
        case 'error': toggleBg = 'bg-red-500'; break;
      }
    }

    return (
      <div className="w-full px-8 py-2 hover:bg-accent/20 transition-colors">
        <div className="flex items-center justify-between">
          <span className="text-base text-muted-foreground">HCI LIVE</span>
          <div className="flex items-center gap-1.5">
            <motion.button
              whileHover={{ scale: 1.15 }}
              whileTap={{ scale: 0.9 }}
              onClick={(e) => { e.stopPropagation(); onSelectWpsDir?.(); }}
              className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
              title="选择捕获软件安装路径"
            >
              <FolderOpen className="w-4 h-4 text-muted-foreground" />
            </motion.button>
            <div
              onClick={(e) => { e.stopPropagation(); onLiveImportToggle?.(); }}
              className={`w-9 h-5 rounded-full cursor-pointer transition-colors relative shrink-0 ${toggleBg}`}
            >
              <div className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-all ${isOn ? 'left-[18px]' : 'left-0.5'}`} />
            </div>
          </div>
        </div>
        {wpsPath && (
          <div
            onClick={(e) => { e.stopPropagation(); onSelectWpsDir?.(); }}
            className="mt-0.5 text-sm text-muted-foreground/60 truncate cursor-pointer hover:text-foreground/80 transition-colors"
            title="选择捕获软件安装路径"
          >
            {middleEllipsis(wpsPath, 40)}
          </div>
        )}
        {isOn && connectedPorts && connectedPorts.length > 1 && (
          <div className="mt-1 flex gap-1">
            {connectedPorts.map(p => {
              const type = portTypes[p] || 'unknown';
              const desc = (portDescriptions[p] || '').toLowerCase();
              const icon = (type === 'bluetooth' || desc.includes('bluetooth') || desc.includes('bth'))
                ? <Bluetooth className="w-3 h-3 text-blue-500 shrink-0" />
                : type === 'usb'
                  ? <Usb className="w-3 h-3 text-muted-foreground shrink-0" />
                  : <Cable className="w-3 h-3 text-muted-foreground shrink-0" />;
              return (
                <motion.button
                  key={p}
                  whileHover={{ scale: 1.08 }}
                  whileTap={{ scale: 0.95 }}
                  onClick={(e) => { e.stopPropagation(); onLiveImportSelectPort?.(p); }}
                  className={`flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium transition-colors ${
                    p === liveImportSelectedPort
                      ? 'bg-primary/20 text-primary'
                      : 'bg-accent/30 text-muted-foreground hover:bg-accent/50'
                  }`}
                >
                  {icon}
                  {p}
                </motion.button>
              );
            })}
          </div>
        )}
        {liveImportStatus === 'active' && liveImportStats && (
          <div className="mt-1.5 flex items-center gap-2 text-xs text-muted-foreground/60">
            <span>已发:{liveImportStats.total}</span>
            <span className="text-green-500/60">✓{liveImportStats.ok}</span>
            {liveImportStats.err > 0 && (
              <span className="text-red-500/60">✗{liveImportStats.err}</span>
            )}
            {liveImportStats.last_hr !== 0 && (
              <span className="text-red-500/60">HR=0x{liveImportStats.last_hr.toString(16)}</span>
            )}
          </div>
        )}
      </div>
    );
  };

  const renderPlainItem = (item: string) => (
    <motion.button
      key={item}
      whileHover={{ x: 8 }}
      className="w-full px-8 py-2 text-base text-left hover:bg-accent/20 transition-colors text-muted-foreground hover:text-foreground"
    >
      {item}
    </motion.button>
  );

  const renderSubItem = (item: string) => {
    if (item === '保存路径') return renderSavePathItem();
    if (item === '时间戳') return renderTimestampItem();
    if (item === '日志过滤') return renderFilterItem();
    if (item === '关键词高亮') return renderHighlightItem();
    if (item === '字体') return renderFontSizeItem();
    if (item === '缓存行数') return renderCacheItem();
    if (item === '渲染间隔') return renderPollingItem();
    if (item === 'HCI日志默认时区' || item === '日志时区') return renderHciTimezoneItem();
    if (item === 'HCI日志提取') {
      return (
        <motion.div
          ref={hciExtractRef}
          whileHover={{ x: hciDropActive ? 0 : 8 }}
          onClick={onHciExtract}
          className={`w-full px-8 py-2 text-base text-left transition-colors cursor-pointer select-none ${
            hciDropActive
              ? 'bg-primary/20 text-primary ring-1 ring-inset ring-primary/40'
              : 'hover:bg-accent/20 text-muted-foreground hover:text-foreground'
          }`}
          title="点击选择日志，或将 .txt/.log 拖放到此处提取 HCI"
        >
          <div className="flex items-center justify-between gap-2">
            <span>HCI日志提取</span>
            {hciDropActive && (
              <span className="text-xs text-primary/80 shrink-0">松开以提取</span>
            )}
          </div>
        </motion.div>
      );
    }
    if (item === 'HCI LIVE') return renderLiveImportItem();
    if (item === 'LC3 Tool Kit') {
      return (
        <motion.button
          whileHover={{ x: 8 }}
          onClick={onOpenLc3ToolKit}
          className="w-full px-8 py-2 text-base text-left hover:bg-accent/20 transition-colors text-muted-foreground hover:text-foreground"
        >
          LC3 Tool Kit
        </motion.button>
      );
    }
    if (item === 'USB日志音频提取') {
      return (
        <motion.button
          whileHover={{ x: 8 }}
          onClick={onOpenUsbAudio}
          className="w-full px-8 py-2 text-base text-left hover:bg-accent/20 transition-colors text-muted-foreground hover:text-foreground"
        >
          USB日志音频提取
        </motion.button>
      );
    }
    return renderPlainItem(item);
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed left-0 right-0 top-12 bottom-0 bg-black/20 backdrop-blur-sm z-[350]"
            onClick={onClose}
          />

          <motion.div
            initial={{ x: '-100%' }}
            animate={{ x: 0 }}
            exit={{ x: '-100%' }}
            transition={{ duration: 0.25, ease: [0.2, 0.8, 0.2, 1] }}
            className="fixed left-0 top-12 bottom-0 w-64 z-[360] flex flex-col"
            style={{
              background: 'var(--acrylic-tint)',
              backdropFilter: 'blur(60px) saturate(180%)',
              WebkitBackdropFilter: 'blur(60px) saturate(180%)',
              borderRight: '1px solid var(--acrylic-border)',
              boxShadow: 'var(--shadow-lg)'
            }}
          >
            <div className="px-4 py-3 border-b border-border/50" />

            <div className="flex-1 overflow-y-auto py-2 scrollbar-none">
              {menuItems.map((section) => (
                <div key={section.title} className="mb-1">
                  <motion.button
                    whileHover={{ x: 4 }}
                    onClick={() => {
                      if (section.items.length > 0) {
                        setExpandedSection(expandedSection === section.title ? null : section.title);
                      }
                    }}
                    className="w-full px-4 py-2.5 flex items-center justify-between hover:bg-accent/20 transition-colors text-left"
                  >
                    <span className="text-base font-medium">{section.title}</span>
                    {section.items.length > 0 && (
                      <ChevronRight
                        className={`w-4 h-4 transition-transform ${
                          expandedSection === section.title ? 'rotate-90' : ''
                        }`}
                      />
                    )}
                  </motion.button>

                  <AnimatePresence>
                    {expandedSection === section.title && section.items.length > 0 && (
                      <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: 'auto', opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        transition={{ duration: 0.2 }}
                        className="overflow-hidden bg-muted/10"
                      >
                        {section.items.map((item) => (
                          <div key={item}>
                            {renderSubItem(item)}
                          </div>
                        ))}
                      </motion.div>
                    )}
                  </AnimatePresence>
                </div>
              ))}
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
