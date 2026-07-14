import { useState, useEffect, useRef, useCallback, useMemo, type MouseEvent as ReactMouseEvent } from 'react';
import { TitleBar } from './components/TitleBar';
import { SecondaryToolbar } from './components/SecondaryToolbar';
import { useWindowMoving } from './hooks/useWindowMoving';
import { LeftSideMenu } from './components/LeftSideMenu';
import { LogPanel } from './components/LogPanel';
import { ConfigPanel } from './components/ConfigPanel';
import { ConfigToggle } from './components/ConfigToggle';
import { ProgressBar } from './components/ProgressBar';
import { StatusBar } from './components/StatusBar';
import { ToastContainer } from './components/Toast';
import { FilterSettingsDialog, type FilterRule } from './components/FilterSettingsDialog';
import { HighlightSettingsDialog, type HighlightRule } from './components/HighlightSettingsDialog';
import { ConfirmDialog } from './components/ConfirmDialog';
import { SearchBar, type SearchScope } from './components/SearchBar';
import { listen, emit } from '@tauri-apps/api/event';
import LC3ToolKitApp from './components/LC3ToolKit/LC3ToolKitApp';
import { USBAudioExtractorDialog } from './components/USBAudioExtractor';
import { performSearch } from './utils/search';

interface ConfigItem {
  entry_type: string;
  label_cn: string;
  name: string;
  tooltip: string;
  var_name: string;
  offset: number;
  bit_offset: number;
  bit_width: number;
  size: number;
  value: any;
  default_value: any;
  min_val: number;
  max_val: number;
  options: { label: string; value: number }[];
  val_type: number;
  str_length: number;
  children: ConfigItem[];
  level_value: number;
  ui_condition_var: string | null;
}

interface LiveImportDiag {
  conn_string: string;
  dll_loaded: boolean;
  init_hr: number;
  init_success: number;
  fts_running: boolean;
}

interface DcfInfo {
  path: string;
  header: any;
  config_tree: ConfigItem[];
  info_offset: number;
  info_len: number;
  info_data: number[];
}

interface FilteredLogEntry {
  text: string;
  idx: number;
}

interface LogWindow {
  start_line: number;
  lines: string[];
}

function generateLogFileName(port: string, date: Date): string {
  const portName = port.replace(/\\/g, '_').replace(/\//g, '_');
  const pad = (n: number) => String(n).padStart(2, '0');
  const dateStr = `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  const timeStr = `${pad(date.getHours())}-${pad(date.getMinutes())}-${pad(date.getSeconds())}`;
  return `${portName}_${dateStr}_${timeStr}.txt`;
}

function getEarliestTimestamp(logs: string[]): Date | null {
  const re = /\((\d{2}):(\d{2}):(\d{2})\.(\d{3})\)/;
  for (const line of logs) {
    const m = re.exec(line);
    if (m) {
      const now = new Date();
      return new Date(
        now.getFullYear(), now.getMonth(), now.getDate(),
        parseInt(m[1]), parseInt(m[2]), parseInt(m[3]), parseInt(m[4])
      );
    }
  }
  return null;
}

function isWindowMoving(): boolean {
  return document.documentElement.getAttribute('data-window-moving') === 'true';
}

function matchesFilterRules(line: string, rules: FilterRule[]): boolean {
  return rules.some(rule => {
    try {
      if (rule.matchType === 'regex') {
        return new RegExp(rule.keyword).test(line);
      }
      return line.includes(rule.keyword);
    } catch {
      return false;
    }
  });
}

function keepTail<T>(items: T[], maxItems: number): T[] {
  return items.length > maxItems ? items.slice(items.length - maxItems) : items;
}

function buildFilteredEntries(lines: string[], baseLine: number, rules: FilterRule[]): FilteredLogEntry[] {
  if (rules.length === 0) return [];
  const out: FilteredLogEntry[] = [];
  lines.forEach((line, idx) => {
    if (matchesFilterRules(line, rules)) {
      out.push({ text: line, idx: baseLine + idx });
    }
  });
  return out;
}

export default function App() {
  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.get('view') === 'lc3') {
    return <LC3ToolKitApp />;
  }

  const [menuOpen, setMenuOpen] = useState(false);
  const [configPanelOpen, setConfigPanelOpen] = useState(false);
  const [dualPanelMode, setDualPanelMode] = useState(false);
  const [dualPanelRatio, setDualPanelRatio] = useState(0.5);
  const [isDark, setIsDark] = useState(false);
  const [selectedPort, setSelectedPort] = useState('');
  const [baudRate, setBaudRate] = useState(1500000);
  const [firmwarePath, setFirmwarePath] = useState('');
  const [dcfInfo, setDcfInfo] = useState<DcfInfo | null>(null);
  const [availablePorts, setAvailablePorts] = useState<string[]>([]);
  const [connectedPorts, setConnectedPorts] = useState<string[]>([]);
  const [isFlashing, setIsFlashing] = useState(false);
  const [flashProgress, setFlashProgress] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const [logsBaseLine, setLogsBaseLine] = useState(0);
  const [filteredLogEntries, setFilteredLogEntries] = useState<FilteredLogEntry[]>([]);
  const [logDisplayPort, setLogDisplayPort] = useState<string>('');
  const [logs2, setLogs2] = useState<string[]>([]);
  const [logs2BaseLine, setLogs2BaseLine] = useState(0);
  const [filteredLogEntries2, setFilteredLogEntries2] = useState<FilteredLogEntry[]>([]);
  const [logDisplayPort2, setLogDisplayPort2] = useState<string>('');
  const [toasts, setToasts] = useState<{ id: string; message: string }[]>([]);
  const [timestampEnabled, setTimestampEnabled] = useState(true);
  const [filterEnabled, setFilterEnabled] = useState(false);
  const [filterRules, setFilterRules] = useState<FilterRule[]>([]);
  const [filterDialogOpen, setFilterDialogOpen] = useState(false);
  const [filterDialogDraft, setFilterDialogDraft] = useState<{ keyword: string; requestId: number } | null>(null);
  const [portTypes, setPortTypes] = useState<Record<string, string>>({});
  const [portDescriptions, setPortDescriptions] = useState<Record<string, string>>({});
  const [highlightEnabled, setHighlightEnabled] = useState(false);
  const [highlightRules, setHighlightRules] = useState<HighlightRule[]>([]);
  const [highlightDialogOpen, setHighlightDialogOpen] = useState(false);
  const [highlightDialogDraft, setHighlightDialogDraft] = useState<{ keyword: string; requestId: number } | null>(null);
  const [fontSize, setFontSize] = useState<'xs' | 'sm' | 'base'>('xs');
  const [maxCacheKb, setMaxCacheKb] = useState(50);
  const maxLines = Math.max(1, maxCacheKb) * 1000;
  const maxFilterLines = maxLines;
  const [pollIntervalMs, setPollIntervalMs] = useState(50);
  const [hciTimezoneOffset, setHciTimezoneOffset] = useState(0);
  const [autoSaveEnabled, setAutoSaveEnabled] = useState(false);
  const [autoSaveEnabled2, setAutoSaveEnabled2] = useState(false);
  const [autoSaveDisplayPath, setAutoSaveDisplayPath] = useState('');
  const [autoSaveDisplayPath2, setAutoSaveDisplayPath2] = useState('');
  const [logSaveDir, setLogSaveDir] = useState('');
  const [liveImportEnabled, setLiveImportEnabled] = useState(false);
  const [liveImportStatus, setLiveImportStatus] = useState<'idle' | 'connecting' | 'ready' | 'active' | 'error'>('idle');
  const [wpsPath, setWpsPath] = useState('');
  const [liveImportReady, setLiveImportReady] = useState(false);
  const [liveImportStats, setLiveImportStats] = useState<{ total: number; ok: number; err: number; last_hr: number; last_err_msg: string }>({ total: 0, ok: 0, err: 0, last_hr: 0, last_err_msg: '' });
  const [liveImportSelectedPort, setLiveImportSelectedPort] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [searchMode, setSearchMode] = useState<'fuzzy' | 'plain' | 'regex'>('plain');
  const [searchCaseSensitive, setSearchCaseSensitive] = useState(false);
  const [searchScope, setSearchScope] = useState<SearchScope>({ view0: true, view1: true, view2: false, view3: false });
  const [searchVisible, setSearchVisible] = useState(false);
  const [searchResult, setSearchResult] = useState<any>(null);
  const [searchCurrentIndex, setSearchCurrentIndex] = useState(0);
  const searchTimerRef = useRef<ReturnType<typeof setTimeout>>();
  const dualPanelContainerRef = useRef<HTMLDivElement>(null);
  // 日志版本号 / 已知行数游标：用 ref 跟踪，避免轮询 effect 随之销毁重建
  // （旧实现用 useState 会堆叠并发轮询链并因闭包陈旧值丢日志）。
  const logsVersionRef = useRef(0);
  const logs2VersionRef = useRef(0);
  const logsLenRef = useRef(0);
  const logs2LenRef = useRef(0);
  const logsBaseLineRef = useRef(0);
  const logs2BaseLineRef = useRef(0);
  const [usbAudioDialogOpen, setUsbAudioDialogOpen] = useState(false);
  const [hciExtractConfirm, setHciExtractConfirm] = useState<{ path: string; fileName: string } | null>(null);
  useWindowMoving(300);

  const autoSavePathRef = useRef('');
  const autoSavePathRef2 = useRef('');
  const autoSaveBaseLineRef = useRef(0);
  const autoSaveBaseLineRef2 = useRef(0);
  const panel0PortConnected = Boolean(logDisplayPort && connectedPorts.includes(logDisplayPort));
  const panel1PortConnected = Boolean(logDisplayPort2 && connectedPorts.includes(logDisplayPort2));

  const handleDualPanelDividerMouseDown = useCallback((e: ReactMouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    const container = dualPanelContainerRef.current;
    if (!container) return;
    const rect = container.getBoundingClientRect();

    const onMove = (e: MouseEvent) => {
      const offsetX = e.clientX - rect.left;
      const ratio = Math.max(0.15, Math.min(0.85, offsetX / rect.width));
      setDualPanelRatio(ratio);
    };

    const onUp = () => {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      document.body.style.cursor = '';
    };

    document.body.style.cursor = 'col-resize';
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }, []);

  const showToast = (message: string, durationMs = 2000) => {
    const id = crypto.randomUUID();
    setToasts(prev => [...prev, { id, message }]);
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, durationMs);
  };

  // 清空日志：同时重置前端显示、行号/版本游标与后端缓冲，保持两端同步
  const handleClearLogs = async (panel: 0 | 1) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('clear_logs', { panel });
    } catch (e) {
      console.error('Clear logs error:', e);
    }
    if (panel === 0) {
      logsLenRef.current = 0;
      logsVersionRef.current = 0;
      logsBaseLineRef.current = 0;
      setLogsBaseLine(0);
      setLogs([]);
      setFilteredLogEntries([]);
    } else {
      logs2LenRef.current = 0;
      logs2VersionRef.current = 0;
      logs2BaseLineRef.current = 0;
      setLogs2BaseLine(0);
      setLogs2([]);
      setFilteredLogEntries2([]);
    }
  };

  useEffect(() => {
    const savedTheme = localStorage.getItem('theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const initialTheme = savedTheme || (prefersDark ? 'dark' : 'light');
    setIsDark(initialTheme === 'dark');
    document.documentElement.classList.toggle('dark', initialTheme === 'dark');
    refreshPorts();

    try {
      const savedFilterRules = localStorage.getItem('logzilla_filterRules');
      if (savedFilterRules) setFilterRules(JSON.parse(savedFilterRules));
      const savedFilterEnabled = localStorage.getItem('logzilla_filterEnabled');
      if (savedFilterEnabled !== null) setFilterEnabled(JSON.parse(savedFilterEnabled));
      const savedTimestamp = localStorage.getItem('logzilla_timestampEnabled');
      if (savedTimestamp !== null) setTimestampEnabled(JSON.parse(savedTimestamp));
      const savedHighlightEnabled = localStorage.getItem('logzilla_highlightEnabled');
      if (savedHighlightEnabled !== null) setHighlightEnabled(JSON.parse(savedHighlightEnabled));
      const savedHighlightRules = localStorage.getItem('logzilla_highlightRules');
      if (savedHighlightRules) {
        const rules = JSON.parse(savedHighlightRules);
        setHighlightRules(rules.map((r: any) => {
          if (r.color && !r.textColor) {
            return { ...r, textColor: r.color, bgColor: 'transparent' };
          }
          return r;
        }));
      }
      const savedLogSaveDir = localStorage.getItem('logzilla_logSaveDir');
      if (savedLogSaveDir) setLogSaveDir(savedLogSaveDir);
      const savedAutoSave = localStorage.getItem('logzilla_autoSaveEnabled');
      if (savedAutoSave !== null) setAutoSaveEnabled(JSON.parse(savedAutoSave));
      const savedFontSize = localStorage.getItem('logzilla_fontSize');
      if (savedFontSize) setFontSize(savedFontSize as 'xs' | 'sm' | 'base');
      const savedMaxCache = localStorage.getItem('logzilla_maxCacheKb');
      if (savedMaxCache !== null) setMaxCacheKb(JSON.parse(savedMaxCache));
      const savedPollMs = localStorage.getItem('logzilla_pollIntervalMs');
      if (savedPollMs !== null) setPollIntervalMs(JSON.parse(savedPollMs));
      const savedHciTz = localStorage.getItem('logzilla_hciTimezoneOffset');
      if (savedHciTz !== null) {
        const tz = JSON.parse(savedHciTz);
        if (typeof tz === 'number' && tz >= -12 && tz <= 14) setHciTimezoneOffset(tz);
      }
      const savedWpsPath = localStorage.getItem('logzilla_wpsPath');
      if (savedWpsPath) setWpsPath(savedWpsPath);
    } catch {}
  }, []);

  useEffect(() => { localStorage.setItem('logzilla_filterRules', JSON.stringify(filterRules)); }, [filterRules]);
  useEffect(() => { localStorage.setItem('logzilla_filterEnabled', JSON.stringify(filterEnabled)); }, [filterEnabled]);
  useEffect(() => { localStorage.setItem('logzilla_highlightRules', JSON.stringify(highlightRules)); }, [highlightRules]);
  useEffect(() => { localStorage.setItem('logzilla_highlightEnabled', JSON.stringify(highlightEnabled)); }, [highlightEnabled]);
  useEffect(() => { localStorage.setItem('logzilla_timestampEnabled', JSON.stringify(timestampEnabled)); }, [timestampEnabled]);
  useEffect(() => { localStorage.setItem('logzilla_logSaveDir', logSaveDir); }, [logSaveDir]);
  useEffect(() => { localStorage.setItem('logzilla_autoSaveEnabled', JSON.stringify(autoSaveEnabled)); }, [autoSaveEnabled]);
  useEffect(() => { localStorage.setItem('logzilla_fontSize', fontSize); }, [fontSize]);
  useEffect(() => { localStorage.setItem('logzilla_maxCacheKb', JSON.stringify(maxCacheKb)); }, [maxCacheKb]);
  useEffect(() => { localStorage.setItem('logzilla_pollIntervalMs', JSON.stringify(pollIntervalMs)); }, [pollIntervalMs]);
  useEffect(() => { localStorage.setItem('logzilla_hciTimezoneOffset', JSON.stringify(hciTimezoneOffset)); }, [hciTimezoneOffset]);
  useEffect(() => { localStorage.setItem('logzilla_wpsPath', wpsPath); }, [wpsPath]);

  useEffect(() => {
    if (!filterEnabled || filterRules.length === 0) {
      setFilteredLogEntries([]);
      setFilteredLogEntries2([]);
      return;
    }
    setFilteredLogEntries(buildFilteredEntries(logs, logsBaseLine, filterRules));
    setFilteredLogEntries2(buildFilteredEntries(logs2, logs2BaseLine, filterRules));
  }, [filterEnabled, filterRules]);

  const refreshAutoSavePath = (port: string) => {
    if (!autoSaveEnabled || !logSaveDir || !port) {
      autoSavePathRef.current = '';
      setAutoSaveDisplayPath('');
      return;
    }
    const fileName = generateLogFileName(port, new Date());
    const path = `${logSaveDir}\\${fileName}`;
    autoSavePathRef.current = path;
    autoSaveBaseLineRef.current = logsLenRef.current;
    setAutoSaveDisplayPath(path);
  };

  const refreshAutoSavePath2 = (port: string) => {
    if (!autoSaveEnabled2 || !logSaveDir || !port) {
      autoSavePathRef2.current = '';
      setAutoSaveDisplayPath2('');
      return;
    }
    const fileName = generateLogFileName(port, new Date());
    const path = `${logSaveDir}\\${fileName}`;
    autoSavePathRef2.current = path;
    autoSaveBaseLineRef2.current = logs2LenRef.current;
    setAutoSaveDisplayPath2(path);
  };

  useEffect(() => {
    if (autoSaveEnabled && logDisplayPort) {
      refreshAutoSavePath(logDisplayPort);
    }
  }, [logDisplayPort, autoSaveEnabled]);

  useEffect(() => {
    if (autoSaveEnabled2 && logDisplayPort2) {
      refreshAutoSavePath2(logDisplayPort2);
    }
  }, [logDisplayPort2, autoSaveEnabled2]);

  // Serial data buffer for proper line handling
  // (不再需要，后端处理时间戳和日志存储)

  // Start/stop backend serial reader for panel 0.
  useEffect(() => {
    if (!logDisplayPort || !panel0PortConnected) return;

    const startReader = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('start_serial_reader', {
          port: logDisplayPort,
          timestampEnabled,
          maxLines,
          panel: 0,
          autoSavePath: autoSaveDisplayPath || null
        });
      } catch (e) {
        console.error('Start serial reader error:', e);
      }
    };
    startReader();

    return () => {
      import('@tauri-apps/api/core')
        .then(({ invoke }) => invoke('stop_serial_reader', { port: logDisplayPort }))
        .catch(() => {});
    };
  }, [logDisplayPort, panel0PortConnected]);

  // Update backend reader config for panel 0 without restarting the thread.
  useEffect(() => {
    if (!logDisplayPort || !panel0PortConnected) return;

    const updateReader = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('start_serial_reader', {
          port: logDisplayPort,
          timestampEnabled,
          maxLines,
          panel: 0,
          autoSavePath: autoSaveDisplayPath || null
        });
      } catch (e) {
        console.error('Update serial reader error:', e);
      }
    };
    updateReader();
  }, [logDisplayPort, panel0PortConnected, timestampEnabled, maxLines, autoSaveDisplayPath]);

  // 从后端获取日志用于显示（面板0）
  useEffect(() => {
    let isActive = true;
    let timer: ReturnType<typeof setTimeout> | null = null;

    const fetchLogs = async () => {
      if (!isActive) return;
      if (isWindowMoving()) {
        timer = setTimeout(fetchLogs, Math.max(120, pollIntervalMs));
        return;
      }
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const [newLogs, newVersion, nextKnownLines] = await invoke<[string[], number, number, number]>('get_logs', {
          panel: 0,
          sinceVersion: logsVersionRef.current,
          knownLines: logsLenRef.current
        });

        if (newVersion > logsVersionRef.current) {
          logsVersionRef.current = newVersion;
          logsLenRef.current = nextKnownLines;
          if (newLogs.length > 0) {
            const newStartLine = nextKnownLines - newLogs.length;
            if (filterEnabled && filterRules.length > 0) {
              const matches = buildFilteredEntries(newLogs, newStartLine, filterRules);
              if (matches.length > 0) {
                setFilteredLogEntries(prev => keepTail([...prev, ...matches], maxFilterLines));
              }
            }
            setLogs(prev => {
              const expectedStart = logsBaseLineRef.current + prev.length;
              const combined = newStartLine === expectedStart ? [...prev, ...newLogs] : [...newLogs];
              let nextBase = newStartLine === expectedStart ? logsBaseLineRef.current : newStartLine;
              const trimmed = keepTail(combined, maxLines);
              nextBase += combined.length - trimmed.length;
              logsBaseLineRef.current = nextBase;
              setLogsBaseLine(nextBase);
              return trimmed;
            });
          }
        }
      } catch (e) {
        console.error('Fetch logs error:', e);
      }

      if (isActive) {
        timer = setTimeout(fetchLogs, pollIntervalMs);
      }
    };

    fetchLogs();
    return () => { isActive = false; if (timer) clearTimeout(timer); };
  }, [pollIntervalMs, maxLines, filterEnabled, filterRules, maxFilterLines]);

  // 从后端获取日志用于显示（面板1）
  useEffect(() => {
    if (!dualPanelMode) return;
    let isActive = true;
    let timer: ReturnType<typeof setTimeout> | null = null;

    const fetchLogs2 = async () => {
      if (!isActive) return;
      if (isWindowMoving()) {
        timer = setTimeout(fetchLogs2, Math.max(120, pollIntervalMs));
        return;
      }
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const [newLogs, newVersion, nextKnownLines] = await invoke<[string[], number, number, number]>('get_logs', {
          panel: 1,
          sinceVersion: logs2VersionRef.current,
          knownLines: logs2LenRef.current
        });

        if (newVersion > logs2VersionRef.current) {
          logs2VersionRef.current = newVersion;
          logs2LenRef.current = nextKnownLines;
          if (newLogs.length > 0) {
            const newStartLine = nextKnownLines - newLogs.length;
            if (filterEnabled && filterRules.length > 0) {
              const matches = buildFilteredEntries(newLogs, newStartLine, filterRules);
              if (matches.length > 0) {
                setFilteredLogEntries2(prev => keepTail([...prev, ...matches], maxFilterLines));
              }
            }
            setLogs2(prev => {
              const expectedStart = logs2BaseLineRef.current + prev.length;
              const combined = newStartLine === expectedStart ? [...prev, ...newLogs] : [...newLogs];
              let nextBase = newStartLine === expectedStart ? logs2BaseLineRef.current : newStartLine;
              const trimmed = keepTail(combined, maxLines);
              nextBase += combined.length - trimmed.length;
              logs2BaseLineRef.current = nextBase;
              setLogs2BaseLine(nextBase);
              return trimmed;
            });
          }
        }
      } catch (e) {
        console.error('Fetch logs2 error:', e);
      }

      if (isActive) {
        timer = setTimeout(fetchLogs2, pollIntervalMs);
      }
    };

    fetchLogs2();
    return () => { isActive = false; if (timer) clearTimeout(timer); };
  }, [dualPanelMode, pollIntervalMs, maxLines, filterEnabled, filterRules, maxFilterLines]);

  // Start/stop backend serial reader for panel 1.
  useEffect(() => {
    if (!logDisplayPort2 || !panel1PortConnected) return;

    const startReader = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('start_serial_reader', {
          port: logDisplayPort2,
          timestampEnabled,
          maxLines,
          panel: 1,
          autoSavePath: autoSaveDisplayPath2 || null
        });
      } catch (e) {
        console.error('Start serial reader2 error:', e);
      }
    };
    startReader();

    return () => {
      import('@tauri-apps/api/core')
        .then(({ invoke }) => invoke('stop_serial_reader', { port: logDisplayPort2 }))
        .catch(() => {});
    };
  }, [logDisplayPort2, panel1PortConnected]);

  // Update backend reader config for panel 1 without restarting the thread.
  useEffect(() => {
    if (!logDisplayPort2 || !panel1PortConnected) return;

    const updateReader = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('start_serial_reader', {
          port: logDisplayPort2,
          timestampEnabled,
          maxLines,
          panel: 1,
          autoSavePath: autoSaveDisplayPath2 || null
        });
      } catch (e) {
        console.error('Update serial reader2 error:', e);
      }
    };
    updateReader();
  }, [logDisplayPort2, panel1PortConnected, timestampEnabled, maxLines, autoSaveDisplayPath2]);

  // Live Import HCI sending
  const liveImportIndexRef = useRef(0);
  const liveImportIndex2Ref = useRef(0);
  const [liveImportErrors, setLiveImportErrors] = useState(0);

  useEffect(() => {
    if (!liveImportReady) return;

    let isActive = true;
    let pollTimer: ReturnType<typeof setTimeout> | null = null;

    const extractFrame = (line: string, sent: boolean): { dataHex: string; h4Type: number; sent: boolean; pcTs: string } | null => {
      const trimmed = line.trim();
      if (!trimmed) return null;
      let pcTs = '';
      let rest = trimmed;
      if (trimmed.startsWith('(')) {
        const close = trimmed.indexOf(')');
        if (close > 12 && close <= 14) { pcTs = trimmed.substring(1, close); rest = trimmed.substring(close + 1).trimStart(); }
      }
      if (!rest.includes('CMD') && !rest.includes('EVT') && !rest.includes('ACL')) return null;
      let h4Type = 0;
      let isSent = sent;
      if (rest.includes('CMD')) { h4Type = 1; isSent = true; }
      else if (rest.includes('EVT')) { h4Type = 4; isSent = false; }
      else if (rest.includes('ACL')) { h4Type = 2; if (rest.includes('=>')) isSent = true; else if (rest.includes('<=')) isSent = false; }
      if (h4Type === 0) return null;
      const arrowIdx = rest.indexOf('=>') !== -1 ? rest.indexOf('=>') + 2 : rest.indexOf('<=') + 2;
      if (arrowIdx < 2) return null;
      const hexPart = rest.substring(arrowIdx).trim();
      if (!hexPart) return null;
      return { dataHex: hexPart, h4Type, sent: isSent, pcTs };
    };

    const poll = async () => {
      if (!isActive) return;
      try {
        const { invoke } = await import('@tauri-apps/api/core');

        const processPanel = async (logLines: string[], baseLine: number, indexRef: { current: number }, sent: boolean) => {
          const absEnd = baseLine + logLines.length;
          if (indexRef.current >= absEnd) return;
          const startLocal = Math.max(0, indexRef.current - baseLine);
          const newLines = logLines.slice(startLocal);
          if (newLines.length === 0) return;
          indexRef.current = absEnd;

          const frames = newLines.map(l => extractFrame(l, sent)).filter(Boolean);
          if (frames.length === 0) return;

          await Promise.all(frames.map(f =>
            invoke('live_import_send_frame', { dataHex: f!.dataHex, h4Type: f!.h4Type, sent: f!.sent, pcTimestamp: f!.pcTs })
              .then(() => true)
              .catch(() => { setLiveImportErrors(prev => prev + 1); return false; })
          ));
        };

        if (logDisplayPort === liveImportSelectedPort) {
          await processPanel(logs, logsBaseLine, liveImportIndexRef, true);
        }
        if (dualPanelMode && logDisplayPort2 === liveImportSelectedPort) {
          await processPanel(logs2, logs2BaseLine, liveImportIndex2Ref, false);
        }
      } catch {}
      if (isActive) pollTimer = setTimeout(poll, 300);
    };

    pollTimer = setTimeout(poll, 300);
    return () => { isActive = false; if (pollTimer) clearTimeout(pollTimer); };
  }, [liveImportReady, logs, logs2, logsBaseLine, logs2BaseLine, dualPanelMode, liveImportSelectedPort, logDisplayPort, logDisplayPort2]);

  // Poll send stats from backend
  useEffect(() => {
    if (!liveImportReady) return;
    let isActive = true;
    let timer: ReturnType<typeof setTimeout> | null = null;

    const pollStats = async () => {
      if (!isActive) return;
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const stats: { total: number; ok: number; err: number; last_hr: number; last_err_msg: string } = await invoke('live_import_stats');
        setLiveImportStats(stats);
        if (stats.err > 0) {
          console.warn(`[HCI STATS] total=${stats.total} ok=${stats.ok} err=${stats.err} last_hr=0x${stats.last_hr.toString(16)} last_err=${stats.last_err_msg}`);
        }
      } catch {}
      if (isActive) timer = setTimeout(pollStats, 5000);
    };

    timer = setTimeout(pollStats, 1000);
    return () => { isActive = false; if (timer) clearTimeout(timer); };
  }, [liveImportReady]);

  // Monitor Fts.exe process – auto-disable if WPS closes
  useEffect(() => {
    if (!liveImportEnabled || !['active', 'ready'].includes(liveImportStatus)) return;
    let isActive = true;
    let timer: ReturnType<typeof setTimeout> | null = null;

    const check = async () => {
      if (!isActive) return;
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const running = await invoke<boolean>('live_import_is_fts_running');
        if (!running && isActive) {
          showToast('WPS 已关闭，HCI 动态加载已自动关闭');
          await invoke('live_import_close').catch(() => {});
          setLiveImportEnabled(false);
          setLiveImportStatus('idle');
          setLiveImportReady(false);
          setLiveImportErrors(0);
          liveImportIndexRef.current = 0;
          liveImportIndex2Ref.current = 0;
          setLiveImportStats({ total: 0, ok: 0, err: 0, last_hr: 0, last_err_msg: '' });
        }
      } catch {}
      if (isActive) timer = setTimeout(check, 1000);
    };

    timer = setTimeout(check, 1000);
    return () => { isActive = false; if (timer) clearTimeout(timer); };
  }, [liveImportEnabled, liveImportStatus]);

  // Auto-select live import port when single port connected
  useEffect(() => {
    if (liveImportEnabled && connectedPorts.length === 1) {
      setLiveImportSelectedPort(connectedPorts[0]);
    }
  }, [liveImportEnabled, connectedPorts]);

  // Toast on error accumulation
  useEffect(() => {
    if (liveImportErrors > 0 && liveImportErrors % 10 === 0) {
      showToast(`⚠ HCI 发送已失败 ${liveImportErrors} 帧`);
    }
  }, [liveImportErrors]);

  const handleThemeToggle = () => {
    const newTheme = isDark ? 'light' : 'dark';
    setIsDark(!isDark);
    localStorage.setItem('theme', newTheme);
    document.documentElement.classList.toggle('dark', newTheme === 'dark');
    emit('theme-changed', { theme: newTheme });
  };

  const refreshPorts = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const ports = await invoke<{ port_name: string; port_type: string; description: string }[]>('list_serial_ports');
      setAvailablePorts(ports.map(p => p.port_name));
      const typeMap: Record<string, string> = {};
      const descMap: Record<string, string> = {};
      ports.forEach(p => { typeMap[p.port_name] = p.port_type; descMap[p.port_name] = p.description; });
      setPortTypes(typeMap);
      setPortDescriptions(descMap);
      showToast(`已刷新串口列表，发现 ${ports.length} 个串口`);
    } catch (e) {
      console.error('Failed to list ports:', e);
      showToast(`刷新串口失败: ${e}`);
    }
  };

  const handlePortChange = async (port: string) => {
    if (!port) return;
    
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      if (connectedPorts.includes(port)) {
        await invoke('close_serial_port', { port });
        setConnectedPorts(prev => prev.filter(p => p !== port));
        if (selectedPort === port) {
          setSelectedPort('');
        }
        showToast(`已断开串口 ${port}`);
      } else {
        await invoke('open_serial_port', { port, baudrate: baudRate });
        setConnectedPorts(prev => [...prev, port]);
        setSelectedPort(port);
        showToast(`已连接串口 ${port}，波特率 ${baudRate}`);
      }
    } catch (e) {
      console.error('Port connection error:', e);
      showToast(`串口连接失败: ${e}`);
    }
  };

  const handleFirmwareSelect = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        defaultPath: firmwarePath || undefined,
        filters: [{
          name: 'DCF',
          extensions: ['dcf']
        }]
      });
      if (selected) {
        const path = selected as string;
        setFirmwarePath(path);
        await loadDcfFile(path);
      }
    } catch (e) {
      console.error('Firmware select error:', e);
      showToast(`选择固件失败: ${e}`);
    }
  };

  const loadDcfFile = async (path: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      showToast(`正在解析DCF文件: ${path}`);
      
      const info = await invoke<DcfInfo>('open_dcf', { path });
      setDcfInfo(info);
      
      showToast(`DCF文件解析成功`);
      showToast(`代码版本: ${info.header.code_version}`);
      showToast(`INFO数据长度: ${info.info_len} 字节`);
      showToast(`配置项数量: ${countConfigItems(info.config_tree)} 个`);
    } catch (e) {
      console.error('DCF parse error:', e);
      showToast(`DCF文件解析失败: ${e}`);
    }
  };

  const countConfigItems = (items: ConfigItem[]): number => {
    let count = 0;
    for (const item of items) {
      if (item.entry_type !== 'SUB' && item.entry_type !== 'LVL') {
        count++;
      }
      if (item.children) {
        count += countConfigItems(item.children);
      }
    }
    return count;
  };

  const handleStartFlash = async () => {
    if (!dcfInfo || connectedPorts.length === 0) {
      showToast('请先选择固件文件并连接串口');
      return;
    }

    const port = connectedPorts[0];
    setIsFlashing(true);
    setFlashProgress(0);
    showToast(`开始烧录到 ${port}...`);
    showToast('烧录功能开发中，暂不可用');
    
    // 模拟烧录进度
    setTimeout(() => {
      setFlashProgress(100);
      setIsFlashing(false);
      showToast('烧录模拟完成');
    }, 2000);
  };

  const handleStopFlash = () => {
    setIsFlashing(false);
    setFlashProgress(0);
    showToast('烧录已停止');
  };

  const handleFilterToggle = () => setFilterEnabled(prev => !prev);

  const handleAddRule = (rule: Omit<FilterRule, 'id'>) => {
    setFilterRules(prev => [...prev, { ...rule, id: crypto.randomUUID() }]);
  };

  const handleDeleteRule = (id: string) => {
    setFilterRules(prev => prev.filter(r => r.id !== id));
  };

  const handleUpdateRule = (id: string, rule: Omit<FilterRule, 'id'>) => {
    setFilterRules(prev => prev.map(r => r.id === id ? { ...r, ...rule } : r));
  };

  const handleHighlightToggle = () => setHighlightEnabled(prev => !prev);

  const handleAddHighlightRule = (rule: Omit<HighlightRule, 'id'>) => {
    setHighlightRules(prev => [...prev, { ...rule, id: crypto.randomUUID() }]);
  };

  const handleDeleteHighlightRule = (id: string) => {
    setHighlightRules(prev => prev.filter(r => r.id !== id));
  };

  const handleUpdateHighlightRule = (id: string, rule: Omit<HighlightRule, 'id'>) => {
    setHighlightRules(prev => prev.map(r => r.id === id ? { ...r, ...rule } : r));
  };

  const handleOpenFilterSettings = () => {
    setFilterDialogDraft({ keyword: '', requestId: Date.now() });
    setFilterDialogOpen(true);
  };

  const handleOpenHighlightSettings = () => {
    setHighlightDialogDraft({ keyword: '', requestId: Date.now() });
    setHighlightDialogOpen(true);
  };

  const handleAddFilterKeywordFromSelection = (keyword: string) => {
    const trimmed = keyword.trim();
    if (!trimmed) return;
    setFilterDialogDraft({ keyword: trimmed, requestId: Date.now() });
    setFilterDialogOpen(true);
  };

  const handleAddHighlightKeywordFromSelection = (keyword: string) => {
    const trimmed = keyword.trim();
    if (!trimmed) return;
    setHighlightDialogDraft({ keyword: trimmed, requestId: Date.now() });
    setHighlightDialogOpen(true);
  };

  const handleSaveConfig = async (changes: [ConfigItem, any][]) => {
    if (!dcfInfo) return;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      const newInfoData = await invoke<number[]>('apply_config_changes', {
        dcfData: dcfInfo.info_data,
        changes: changes,
        infoOffset: dcfInfo.info_offset,
        infoLen: dcfInfo.info_len
      });

      setDcfInfo(prev => prev ? { ...prev, info_data: newInfoData } : null);
      showToast('配置已保存');
    } catch (e) {
      console.error('Save config error:', e);
      showToast(`保存配置失败: ${e}`);
    }
  };

  const handleSelectLogDir = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const dir = await open({ directory: true, multiple: false, defaultPath: logSaveDir || undefined });
      if (dir) {
        const path = dir as string;
        setLogSaveDir(path);
        showToast(`日志保存目录已设置为: ${path}`);
        return path;
      }
    } catch (e) {
      console.error('Select log dir error:', e);
      showToast(`选择目录失败: ${e}`);
    }
    return '';
  };

  const handleSave = async (opts: { panel: 0 | 1; isFilter: boolean }) => {
    const { panel, isFilter } = opts;
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const source = panel === 1 ? logs2 : logs;
      const port = (panel === 1 ? logDisplayPort2 : logDisplayPort) || 'unknown';
      // For the filter view, save only the lines that match the active filter
      // rules — i.e. exactly what the filtered sub-window displays.
      const toSave = isFilter
        ? source.filter(l => matchesFilterRules(l, filterRules))
        : source;
      const timestamp = getEarliestTimestamp(source) || new Date();
      // Add an "_filter" suffix to the port segment of the filename so the
      // filtered-window save is distinguishable from the full-window save.
      const portSegment = isFilter ? `${port}_filter` : port;
      const defaultName = generateLogFileName(portSegment, timestamp);
      const defaultDir = logSaveDir || '';
      const defaultPath = defaultDir ? `${defaultDir}\\${defaultName}` : defaultName;

      const path = await save({
        defaultPath,
        filters: [{
          name: '日志文件',
          extensions: ['txt', 'log']
        }]
      });
      if (!path) return;

      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('write_file', { path, content: toSave.join('\n') });
      showToast(`日志已保存`);
    } catch (e) {
      console.error('Save error:', e);
      showToast(`保存日志失败: ${e}`);
    }
  };

  const handleAutoSaveToggle2 = async () => {
    if (autoSaveEnabled2) {
      autoSavePathRef2.current = '';
      autoSaveBaseLineRef2.current = 0;
      setAutoSaveDisplayPath2('');
      setAutoSaveEnabled2(false);
      showToast('右侧面板自动保存已关闭');
      return;
    }

    let dir = logSaveDir;
    if (!dir) {
      dir = await handleSelectLogDir();
      if (!dir) return;
    }

    const port = logDisplayPort2;
    if (!port) {
      showToast('请先选择串口');
      return;
    }

    const now = new Date();
    const fileName = generateLogFileName(port, now);
    const path = `${dir}\\${fileName}`;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (logs2.length > 0) {
        await invoke('write_file', { path, content: logs2.join('\n') + '\n' });
        autoSaveBaseLineRef2.current = logs2BaseLineRef.current;
      } else {
        await invoke('write_file', { path, content: '' });
        autoSaveBaseLineRef2.current = logs2LenRef.current;
      }
      if (panel1PortConnected) {
        await invoke('start_serial_reader', {
          port,
          timestampEnabled,
          maxLines,
          panel: 1,
          autoSavePath: path,
        });
      }
      autoSavePathRef2.current = path;
      setAutoSaveDisplayPath2(path);
      setAutoSaveEnabled2(true);
      showToast(`右侧面板自动保存已开启: ${fileName}`);
    } catch (e) {
      console.error('Auto-save start error:', e);
      showToast(`开启自动保存失败: ${e}`);
    }
  };

  const handleRevealFile = async (path: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('reveal_in_explorer', { path });
    } catch (e) {
      console.error('Reveal error:', e);
    }
  };

  const handleLoadLogWindow = useCallback(async (panel: 0 | 1, centerLine: number): Promise<boolean> => {
    const path = panel === 0 ? autoSavePathRef.current : autoSavePathRef2.current;
    const fileBaseLine = panel === 0 ? autoSaveBaseLineRef.current : autoSaveBaseLineRef2.current;
    if (!path) {
      showToast('当前日志已被显示缓存裁剪，且未开启自动保存，无法回读定位');
      return false;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (centerLine < fileBaseLine) {
        showToast('目标日志早于当前自动保存文件，无法回读定位');
        return false;
      }
      const before = Math.floor(maxLines / 2);
      const after = Math.max(0, maxLines - before - 1);
      const win = await invoke<LogWindow>('read_log_window', {
        path,
        centerLine: centerLine - fileBaseLine,
        before,
        after,
      });

      if (win.lines.length === 0) {
        showToast('自动保存文件中未找到目标日志行');
        return false;
      }

      if (panel === 0) {
        const absoluteStart = fileBaseLine + win.start_line;
        logsBaseLineRef.current = absoluteStart;
        setLogsBaseLine(absoluteStart);
        setLogs(win.lines);
      } else {
        const absoluteStart = fileBaseLine + win.start_line;
        logs2BaseLineRef.current = absoluteStart;
        setLogs2BaseLine(absoluteStart);
        setLogs2(win.lines);
      }
      return true;
    } catch (e) {
      console.error('Load log window error:', e);
      showToast(`回读日志失败: ${e}`);
      return false;
    }
  }, [maxLines]);

  const handleExportHci = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (autoSaveEnabled && autoSavePathRef.current) {
        const result = await invoke<string>('extract_hci', {
          inputPath: autoSavePathRef.current,
          timezoneOffsetHours: hciTimezoneOffset,
        });
        showToast(result);
      } else {
        const { save } = await import('@tauri-apps/plugin-dialog');
        const timestamp = getEarliestTimestamp(logs) || new Date();
        const defaultName = generateLogFileName(logDisplayPort || 'unknown', timestamp);
        const defaultDir = logSaveDir || '';
        const defaultPath = defaultDir ? `${defaultDir}\\${defaultName}` : defaultName;
        const path = await save({
          defaultPath,
          filters: [{ name: '日志文件', extensions: ['txt', 'log'] }]
        });
        if (!path) return;
        await invoke('write_file', { path, content: logs.join('\n') });
        const result = await invoke<string>('extract_hci', {
          inputPath: path,
          timezoneOffsetHours: hciTimezoneOffset,
        });
        showToast(result);
      }
    } catch (e) {
      console.error('HCI export error:', e);
      showToast(`导出HCI日志失败: ${e}`);
    }
  };

  const handleExportHci2 = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (autoSaveEnabled2 && autoSavePathRef2.current) {
        const result = await invoke<string>('extract_hci', {
          inputPath: autoSavePathRef2.current,
          timezoneOffsetHours: hciTimezoneOffset,
        });
        showToast(result);
      } else {
        const { save } = await import('@tauri-apps/plugin-dialog');
        const timestamp = getEarliestTimestamp(logs2) || new Date();
        const defaultName = generateLogFileName(logDisplayPort2 || 'unknown', timestamp);
        const defaultDir = logSaveDir || '';
        const defaultPath = defaultDir ? `${defaultDir}\\${defaultName}` : defaultName;
        const path = await save({
          defaultPath,
          filters: [{ name: '日志文件', extensions: ['txt', 'log'] }]
        });
        if (!path) return;
        await invoke('write_file', { path, content: logs2.join('\n') });
        const result = await invoke<string>('extract_hci', {
          inputPath: path,
          timezoneOffsetHours: hciTimezoneOffset,
        });
        showToast(result);
      }
    } catch (e) {
      console.error('HCI export error:', e);
      showToast(`导出HCI日志失败: ${e}`);
    }
  };

  const handleSelectWpsDir = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({ directory: true, multiple: false });
      if (selected) setWpsPath(selected as string);
    } catch (e) {
      console.error('Select WPS dir error:', e);
    }
  };

  const runHciExtract = async (inputPath: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke<string>('extract_hci', {
        inputPath,
        timezoneOffsetHours: hciTimezoneOffset,
      });
      showToast(result);
    } catch (e) {
      console.error('HCI extract error:', e);
      showToast(`HCI日志提取失败: ${e}`);
    }
  };

  const confirmAndExtractHci = async (inputPath: string) => {
    const fileName = inputPath.split(/[/\\]/).pop() || inputPath;
    setHciExtractConfirm({ path: inputPath, fileName });
  };

  const handleHciExtractConfirm = async () => {
    const pending = hciExtractConfirm;
    setHciExtractConfirm(null);
    if (!pending) return;
    await runHciExtract(pending.path);
  };

  const handleHciExtractCancel = () => {
    setHciExtractConfirm(null);
  };

  const handleHciExtract = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{
          name: '日志文件',
          extensions: ['txt', 'log']
        }]
      });
      if (!selected) return;
      await confirmAndExtractHci(selected as string);
    } catch (e) {
      console.error('HCI extract error:', e);
      showToast(`HCI日志提取失败: ${e}`);
    }
  };

  const handleHciExtractFromPath = async (inputPath: string) => {
    if (!inputPath) {
      showToast('无法识别拖入的文件路径，请拖入本地 .txt / .log 文件');
      return;
    }
    const lower = inputPath.toLowerCase();
    if (!lower.endsWith('.txt') && !lower.endsWith('.log')) {
      showToast('请拖入 .txt 或 .log 日志文件');
      return;
    }
    await confirmAndExtractHci(inputPath);
  };

  const liveImportBgRef = useRef<number | null>(null);

  const handleLiveImportToggle = async () => {
    if (liveImportEnabled) {
      if (liveImportBgRef.current) {
        clearInterval(liveImportBgRef.current);
        liveImportBgRef.current = null;
      }
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('live_import_close');
      } catch {}
      setLiveImportEnabled(false);
      setLiveImportStatus('idle');
      setLiveImportReady(false);
      setLiveImportErrors(0);
      liveImportIndexRef.current = 0;
      liveImportIndex2Ref.current = 0;
      setLiveImportStats({ total: 0, ok: 0, err: 0, last_hr: 0, last_err_msg: '' });
      showToast('HCI动态加载已关闭');
      return;
    }

    let path = wpsPath;
    if (!path) {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({ directory: true, multiple: false, defaultPath: wpsPath || undefined });
        if (!selected) return;
        path = selected as string;
        setWpsPath(path);
      } catch (e) {
        showToast('选择WPS路径失败');
        return;
      }
    }

    setLiveImportEnabled(true);
    setLiveImportStatus('connecting');
    setLiveImportErrors(0);
    liveImportIndexRef.current = 0;
    liveImportIndex2Ref.current = 0;
    setLiveImportStats({ total: 0, ok: 0, err: 0, last_hr: 0, last_err_msg: '' });
    showToast('正在连接 WPS...');

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Step 1: Initialize Live Import
      const diag = await invoke<LiveImportDiag>('live_import_init', { wpsPath: path });

      if (diag.init_hr !== 0 || diag.init_success === 0) {
        showToast('❌ 初始化失败，请检查WPS路径');
        setLiveImportEnabled(false);
        setLiveImportStatus('error');
        return;
      }

      // Step 2: Launch FTS if not running
      if (!diag.fts_running) {
        await invoke('launch_wps', { wpsPath: path });
      }

      // Step 3: Poll IsAppReady
      let ready = false;
      for (let i = 0; i < 30; i++) {
        await new Promise(r => setTimeout(r, 2000));
        try {
          ready = await invoke<boolean>('live_import_is_ready');
          if (ready) break;
        } catch {}
      }

      if (ready) {
        setLiveImportStatus('active');
        setLiveImportReady(true);
        showToast('已连接就绪，请点击Start Capture', 7000);
      } else {
        setLiveImportStatus('ready');
        showToast('⏳ 捕获软件连接失败');
        // Background poll: check IsAppReady in the background
        liveImportBgRef.current = window.setInterval(async () => {
          try {
            const { invoke } = await import('@tauri-apps/api/core');
            const bgReady = await invoke<boolean>('live_import_is_ready');
            if (bgReady) {
              if (liveImportBgRef.current) clearInterval(liveImportBgRef.current);
              liveImportBgRef.current = null;
              setLiveImportStatus('active');
              setLiveImportReady(true);
              showToast('已连接就绪，请点击Start Capture', 7000);
            }
          } catch {}
        }, 1000);
      }
    } catch (e) {
      setLiveImportEnabled(false);
      setLiveImportStatus('error');
      setLiveImportReady(false);
      showToast(`WPS连接失败: ${e}`);
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('live_import_close');
      } catch {}
    }
  };

  const handleAutoSaveToggle = async () => {
    if (autoSaveEnabled) {
      autoSavePathRef.current = '';
      autoSaveBaseLineRef.current = 0;
      setAutoSaveDisplayPath('');
      setAutoSaveEnabled(false);
      showToast('自动保存已关闭');
      return;
    }

    let dir = logSaveDir;
    if (!dir) {
      dir = await handleSelectLogDir();
      if (!dir) return;
    }

    const port = logDisplayPort;
    if (!port) {
      showToast('请先选择串口');
      return;
    }

    const now = new Date();
    const fileName = generateLogFileName(port, now);
    const path = `${dir}\\${fileName}`;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      if (logs.length > 0) {
        await invoke('write_file', { path, content: logs.join('\n') + '\n' });
        autoSaveBaseLineRef.current = logsBaseLineRef.current;
      } else {
        await invoke('write_file', { path, content: '' });
        autoSaveBaseLineRef.current = logsLenRef.current;
      }
      if (panel0PortConnected) {
        await invoke('start_serial_reader', {
          port,
          timestampEnabled,
          maxLines,
          panel: 0,
          autoSavePath: path,
        });
      }
      autoSavePathRef.current = path;
      setAutoSaveDisplayPath(path);
      setAutoSaveEnabled(true);
      showToast(`自动保存已开启: ${fileName}`);
    } catch (e) {
      console.error('Auto-save start error:', e);
      showToast(`开启自动保存失败: ${e}`);
    }
  };

  // Ctrl+F to open search
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
        e.preventDefault();
        e.stopPropagation();
        
        // Default search scope based on current state
        const hasFilter = filterEnabled && filterRules.length > 0;
        const defaultScope: SearchScope = {
          view0: true,
          view1: hasFilter,
          view2: dualPanelMode,
          view3: dualPanelMode && hasFilter,
        };
        
        setSearchScope(defaultScope);
        setSearchVisible(true);
      }
    };
    document.addEventListener('keydown', handleKeyDown, true);
    return () => document.removeEventListener('keydown', handleKeyDown, true);
  }, [dualPanelMode, filterEnabled, filterRules]);

  const filteredLogTexts = useMemo(() => filteredLogEntries.map(e => e.text), [filteredLogEntries]);
  const filteredLogIndices = useMemo(() => filteredLogEntries.map(e => e.idx), [filteredLogEntries]);
  const filteredLogTexts2 = useMemo(() => filteredLogEntries2.map(e => e.text), [filteredLogEntries2]);
  const filteredLogIndices2 = useMemo(() => filteredLogEntries2.map(e => e.idx), [filteredLogEntries2]);

  // Helper: get lines for a specific view
  const getLinesForView = useCallback((viewId: string): string[] => {
    if (viewId === 'view1') {
      return filterEnabled ? filteredLogTexts : [];
    }
    if (viewId === 'view3') {
      return filterEnabled ? filteredLogTexts2 : [];
    }
    return viewId === 'view2' ? logs2 : logs;
  }, [logs, logs2, filterEnabled, filteredLogTexts, filteredLogTexts2]);

  // Listen for LC3 window port changes (auto-update connected ports in main window)
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<{ port: string; action: string }>('lc3-port-changed', (event) => {
      const { port, action } = event.payload;
      if (action === 'steal' && port) {
        // LC3 stole a port — remove it from main panel's connected list
        setConnectedPorts(prev => prev.filter(p => p !== port));
        if (logDisplayPort === port) setLogDisplayPort('');
        if (logDisplayPort2 === port) setLogDisplayPort2('');
      }
      // Refresh connected ports to sync state
      refreshPorts();
    }).then(fn => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [logDisplayPort, logDisplayPort2]);

  // Listen for backend-driven serial disconnect events (e.g. USB unplugged).
  // Event-driven: no polling timer, so it adds zero main-thread cost and never
  // makes window dragging / serial-port interaction feel laggy. When the
  // backend reader detects a fatal read error, it tears down the port and
  // emits "serial-disconnected"; here we drop it from connectedPorts, which
  // flips the dropdown green dot to "disconnected" automatically. The reader
  // start/stop effects above react to panel*PortConnected going false and
  // stop the (already-exited) backend reader cleanly.
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<string>('serial-disconnected', (event) => {
      const port = event.payload;
      if (!port) return;
      setConnectedPorts(prev => prev.filter(p => p !== port));
      if (port === logDisplayPort || port === logDisplayPort2) {
        showToast(`串口 ${port} 已断开`);
      }
    }).then(fn => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [logDisplayPort, logDisplayPort2]);

  // Search function using frontend WebAssembly
  const handleSearch = useCallback(async (query: string, mode: string, caseSensitive: boolean) => {
    if (!query) {
      setSearchResult(null);
      setSearchCurrentIndex(0);
      return;
    }
    
    try {
      // Get selected views
      const selectedViews: string[] = [];
      if (searchScope.view0) selectedViews.push('view0');
      if (searchScope.view1 && filterEnabled && filterRules.length > 0) selectedViews.push('view1');
      if (searchScope.view2) selectedViews.push('view2');
      if (searchScope.view3 && filterEnabled && filterRules.length > 0) selectedViews.push('view3');
      
      // Build view lines
      const viewLines = selectedViews.map(viewId => ({
        viewId,
        lines: getLinesForView(viewId),
      }));
      
      // Perform search
      const result = await performSearch(query, mode as 'fuzzy' | 'plain' | 'regex', viewLines, caseSensitive, 100);
      setSearchResult(result);
    } catch (e) {
      console.error('Search error:', e);
    }
  }, [searchScope, getLinesForView]);

  // Debounced search
  useEffect(() => {
    if (!searchVisible || !searchQuery) {
      setSearchResult(null);
      return;
    }
    // Fuzzy mode: need 3+ chars to auto-trigger search
    if (searchMode === 'fuzzy' && searchQuery.length < 3) {
      setSearchResult(null);
      return;
    }
    clearTimeout(searchTimerRef.current);
    searchTimerRef.current = setTimeout(() => {
      handleSearch(searchQuery, searchMode, searchCaseSensitive);
    }, 100);
    return () => clearTimeout(searchTimerRef.current);
  }, [searchQuery, searchMode, searchCaseSensitive, searchScope, searchVisible, handleSearch]);

  // Auto-update search results when logs change
  useEffect(() => {
    if (!searchVisible || !searchQuery) return;
    handleSearch(searchQuery, searchMode, searchCaseSensitive);
  }, [logs, logs2, searchVisible, searchQuery, searchMode, searchCaseSensitive, handleSearch]);

  // Use ref so handleSearchNavigate always reads the latest searchResult
  // (avoids stale-closure issues between useCallback and useEffect)
  const searchResultRef = useRef(searchResult);
  searchResultRef.current = searchResult;
  const searchNavigateRetryRef = useRef(0);

  const handleSearchNavigate = useCallback((index: number) => {
    setSearchCurrentIndex(index);

    const result = searchResultRef.current;
    if (!result?.matches?.[index]) return;

    const match = result.matches[index];
    const viewId = match.view_id;
    const lineIndex = match.line_index;

    // DOM structure:
    //   div[data-view-id]          ← scrollableRef (overflow-y-auto)
    //     div                      ← containerRef
    //       div                    ← log line 0
    //       div                    ← log line 1
    const scrollableDiv = document.querySelector(`[data-view-id="${viewId}"]`) as HTMLElement | null;
    if (!scrollableDiv) return;
    const logContainer = scrollableDiv.querySelector(':scope > div');
    if (!logContainer) return;
    const lineDivs = logContainer.querySelectorAll(':scope > div');
    const target = lineDivs[lineIndex] as HTMLElement | undefined;
    if (!target) {
      if (searchNavigateRetryRef.current >= 10) {
        searchNavigateRetryRef.current = 0;
        return;
      }
      searchNavigateRetryRef.current += 1;
      requestAnimationFrame(() => {
        const current = searchResultRef.current;
        if (current?.matches?.[index] === match) handleSearchNavigate(index);
      });
      return;
    }
    searchNavigateRetryRef.current = 0;

    // Highlight
    document.querySelectorAll('.search-current').forEach(el => el.classList.remove('search-current'));
    document.querySelectorAll('mark.search-kw-active').forEach(el => el.classList.remove('search-kw-active'));
    target.classList.add('search-current');

    // Activate the specific keyword <mark> within the line
    const marks = target.querySelectorAll('mark.search-kw');
    if (marks.length > 0) {
      // Find the mark whose text offset matches the first highlight
      const hl = match.highlights?.[0];
      if (hl) {
        // Walk text nodes + marks to find which mark covers offset hl.start
        let charPos = 0;
        let activated = false;
        const walker = document.createTreeWalker(target.querySelector('span') || target, NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT);
        while (walker.nextNode()) {
          const node = walker.currentNode;
          if (node.nodeType === Node.TEXT_NODE) {
            charPos += (node.textContent || '').length;
          } else if (node.nodeType === Node.ELEMENT_NODE && (node as HTMLElement).tagName === 'MARK') {
            const markLen = (node.textContent || '').length;
            if (charPos + markLen > hl.start) {
              (node as HTMLElement).classList.add('search-kw-active');
              activated = true;
              break;
            }
            charPos += markLen;
          }
        }
        if (!activated && marks[0]) {
          marks[0].classList.add('search-kw-active');
        }
      } else {
        marks[0].classList.add('search-kw-active');
      }
    }

    // Scroll the scrollable container to center the target line
    const containerRect = scrollableDiv.getBoundingClientRect();
    const targetRect = target.getBoundingClientRect();
    const targetTopInContainer = targetRect.top - containerRect.top + scrollableDiv.scrollTop;
    const desiredScrollTop = targetTopInContainer - scrollableDiv.clientHeight / 2 + target.offsetHeight / 2;
    scrollableDiv.scrollTo({ top: Math.max(0, desiredScrollTop), behavior: 'smooth' });
  }, []);  // stable — reads from ref, no dependency needed

  // Auto-navigate to first match when search results arrive
  const lastNavigatedQueryRef = useRef('');
  useEffect(() => {
    if (searchResult && searchResult.total_count > 0 && searchResult.query !== lastNavigatedQueryRef.current) {
      lastNavigatedQueryRef.current = searchResult.query;
      handleSearchNavigate(0);
    }
    if (!searchResult || searchResult.total_count === 0) {
      lastNavigatedQueryRef.current = '';
    }
  }, [searchResult, handleSearchNavigate]);

  const handleSearchClose = () => {
    setSearchVisible(false);
    setSearchResult(null);
    setSearchCurrentIndex(0);
    document.querySelectorAll('.search-current').forEach(el => el.classList.remove('search-current'));
    document.querySelectorAll('mark.search-kw-active').forEach(el => el.classList.remove('search-kw-active'));
  };

  const handleSearchHere = (viewId: number, query?: string) => {
    const scope: SearchScope = { view0: false, view1: false, view2: false, view3: false };
    const key = `view${viewId}` as keyof SearchScope;
    if (key in scope) scope[key] = true;
    setSearchScope(scope);
    if (query) setSearchQuery(query);
    setSearchVisible(true);
  };

  const handleOpenLc3ToolKit = async () => {
    setMenuOpen(false);
    try {
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      // Check if LC3 window already exists
      const existing = await WebviewWindow.getByLabel('lc3-toolkit');
      if (existing) {
        await existing.setFocus();
        return;
      }
      const win = new WebviewWindow('lc3-toolkit', {
        url: '/lc3.html',
        title: 'LC3 Tool Kit',
        width: 460,
        height: 460,
        minWidth: 460,
        minHeight: 400,
        maxWidth: 460,
        maxHeight: 800,
        decorations: false,
        resizable: false,
        center: true,
      });
      void win.setIcon('/logo.png').catch((e) => {
        console.error('LC3 window icon error:', e);
      });
      win.once('tauri://error', (e) => {
        console.error('LC3 window error:', e);
        showToast('打开 LC3 窗口失败');
      });
    } catch (e) {
      console.error('Failed to open LC3 window:', e);
      showToast('打开 LC3 窗口失败');
    }
  };

  const handleOpenUsbAudio = () => {
    setUsbAudioDialogOpen(true);
  };

  return (
    <div className="w-screen h-screen flex flex-col bg-background text-foreground overflow-hidden">
      <TitleBar />

      <SecondaryToolbar
        onMenuClick={() => setMenuOpen(true)}
        selectedPort={selectedPort}
        baudRate={baudRate}
        firmwarePath={firmwarePath}
        dualMode={dualPanelMode}
        isFlashing={isFlashing}
        onPortChange={handlePortChange}
        onBaudRateChange={setBaudRate}
        onFirmwareSelect={handleFirmwareSelect}
        onDualModeToggle={() => setDualPanelMode(!dualPanelMode)}
        onThemeToggle={handleThemeToggle}
        onStartFlash={handleStartFlash}
        onStopFlash={handleStopFlash}
        onRefreshPorts={refreshPorts}
        isDark={isDark}
        availablePorts={availablePorts}
        connectedPorts={connectedPorts}
        portTypes={portTypes}
        portDescriptions={portDescriptions}
      />

      {isFlashing && (
        <div className="px-6 py-2 shrink-0">
          <ProgressBar progress={flashProgress} status="烧录中..." />
        </div>
      )}

      <div className={`flex-1 min-h-0 px-6 py-4 flex relative ${dualPanelMode ? 'gap-0' : 'gap-4'}`} ref={dualPanelContainerRef}>
        {searchVisible && (
          <div className="absolute left-1/2 -translate-x-1/2 z-[9999]" style={{ top: -8 }}>
            <SearchBar
              query={searchQuery}
              onQueryChange={setSearchQuery}
              mode={searchMode}
              onModeChange={setSearchMode}
              caseSensitive={searchCaseSensitive}
              onCaseChange={setSearchCaseSensitive}
              scope={searchScope}
              onScopeChange={setSearchScope}
              onClose={handleSearchClose}
              dualPanelMode={dualPanelMode}
              searchResult={searchResult}
              currentMatchIndex={searchCurrentIndex}
              onNavigate={handleSearchNavigate}
              onSearchNow={() => handleSearch(searchQuery, searchMode, searchCaseSensitive)}
            />
          </div>
        )}
        {!dualPanelMode ? (
          <div className="flex-1 min-h-0">
            <LogPanel
              isConnected={connectedPorts.length > 0}
              logs={logs}
              onClear={() => handleClearLogs(0)}
              onSave={handleSave}
              autoSaveEnabled={autoSaveEnabled}
              onAutoSaveToggle={handleAutoSaveToggle}
              autoSaveFilePath={autoSaveDisplayPath}
              onRevealFile={handleRevealFile}
              onExportHci={handleExportHci}
              selectedPort={logDisplayPort}
              availablePorts={availablePorts}
              connectedPorts={connectedPorts}
              disabledPorts={[]}
              onPortChange={setLogDisplayPort}
              onRefreshPorts={refreshPorts}
              filterEnabled={filterEnabled}
              filterRules={filterRules}
              highlightEnabled={highlightEnabled}
              highlightRules={highlightRules}
              portTypes={portTypes}
              portDescriptions={portDescriptions}
              fontSize={fontSize}
              logBaseIndex={logsBaseLine}
              filteredLogsOverride={filteredLogTexts}
              filteredIndicesOverride={filteredLogIndices}
              onLoadLogWindow={handleLoadLogWindow}
              panelIndex={0}
              searchQuery={searchQuery}
              searchMode={searchMode}
              searchCaseSensitive={searchCaseSensitive}
              view0Active={searchScope.view0 && searchVisible}
              view1Active={searchScope.view1 && searchVisible && filterEnabled && filterRules.length > 0}
              onSearchHere={handleSearchHere}
              onAddFilterKeyword={handleAddFilterKeywordFromSelection}
              onAddHighlightKeyword={handleAddHighlightKeywordFromSelection}
            />
          </div>
        ) : (
          <>
            <div className="min-h-0 min-w-0" style={{ flex: `${dualPanelRatio} 1 0` }}>
              <LogPanel
                isConnected={connectedPorts.length > 0}
                logs={logs}
                onClear={() => handleClearLogs(0)}
                onSave={handleSave}
                autoSaveEnabled={autoSaveEnabled}
                onAutoSaveToggle={handleAutoSaveToggle}
                autoSaveFilePath={autoSaveDisplayPath}
                onRevealFile={handleRevealFile}
                onExportHci={handleExportHci}
                selectedPort={logDisplayPort}
                availablePorts={availablePorts}
                connectedPorts={connectedPorts}
              disabledPorts={logDisplayPort2 ? [logDisplayPort2] : []}
              onPortChange={setLogDisplayPort}
              onRefreshPorts={refreshPorts}
              filterEnabled={filterEnabled}
              filterRules={filterRules}
              highlightEnabled={highlightEnabled}
              highlightRules={highlightRules}
              portTypes={portTypes}
              portDescriptions={portDescriptions}
              fontSize={fontSize}
              logBaseIndex={logsBaseLine}
              filteredLogsOverride={filteredLogTexts}
              filteredIndicesOverride={filteredLogIndices}
              onLoadLogWindow={handleLoadLogWindow}
              panelIndex={0}
                searchQuery={searchQuery}
                searchMode={searchMode}
                searchCaseSensitive={searchCaseSensitive}
                view0Active={searchScope.view0 && searchVisible}
                view1Active={searchScope.view1 && searchVisible && filterEnabled && filterRules.length > 0}
                onSearchHere={handleSearchHere}
                onAddFilterKeyword={handleAddFilterKeywordFromSelection}
                onAddHighlightKeyword={handleAddHighlightKeywordFromSelection}
                />
            </div>
            <button
              type="button"
              className="w-4 flex items-center justify-center cursor-col-resize shrink-0 group"
              onMouseDown={handleDualPanelDividerMouseDown}
              title="调整左右窗口宽度"
              aria-label="调整左右窗口宽度"
            >
              <span className="w-1 h-8 rounded-full bg-border dark:bg-border/50 group-hover:bg-accent/70 group-hover:w-1.5 transition-all duration-200" />
            </button>
            <div className="min-h-0 min-w-0" style={{ flex: `${1 - dualPanelRatio} 1 0` }}>
              <LogPanel
                isConnected={connectedPorts.length > 0}
                logs={logs2}
                onClear={() => handleClearLogs(1)}
                onSave={handleSave}
                autoSaveEnabled={autoSaveEnabled2}
                onAutoSaveToggle={handleAutoSaveToggle2}
                autoSaveFilePath={autoSaveDisplayPath2}
                onRevealFile={handleRevealFile}
                onExportHci={handleExportHci2}
                selectedPort={logDisplayPort2}
                availablePorts={availablePorts}
                connectedPorts={connectedPorts}
                disabledPorts={logDisplayPort ? [logDisplayPort] : []}
                onPortChange={setLogDisplayPort2}
                onRefreshPorts={refreshPorts}
                filterEnabled={filterEnabled}
                filterRules={filterRules}
                highlightEnabled={highlightEnabled}
                highlightRules={highlightRules}
                portTypes={portTypes}
                portDescriptions={portDescriptions}
                fontSize={fontSize}
                logBaseIndex={logs2BaseLine}
                filteredLogsOverride={filteredLogTexts2}
                filteredIndicesOverride={filteredLogIndices2}
                onLoadLogWindow={handleLoadLogWindow}
                panelIndex={1}
                searchQuery={searchQuery}
                searchMode={searchMode}
                searchCaseSensitive={searchCaseSensitive}
                view0Active={searchScope.view2 && searchVisible}
                view1Active={searchScope.view3 && searchVisible && filterEnabled && filterRules.length > 0}
                onSearchHere={handleSearchHere}
                onAddFilterKeyword={handleAddFilterKeywordFromSelection}
                onAddHighlightKeyword={handleAddHighlightKeywordFromSelection}
                />
            </div>
          </>
        )}
      </div>

      {!configPanelOpen && <ConfigToggle onClick={() => setConfigPanelOpen(true)} />}

      <LeftSideMenu
        isOpen={menuOpen}
        onClose={() => setMenuOpen(false)}
        timestampEnabled={timestampEnabled}
        onTimestampToggle={() => setTimestampEnabled(prev => !prev)}
        filterEnabled={filterEnabled}
        onFilterToggle={handleFilterToggle}
        onFilterSettings={handleOpenFilterSettings}
        highlightEnabled={highlightEnabled}
        onHighlightToggle={handleHighlightToggle}
        onHighlightSettings={handleOpenHighlightSettings}
        logSaveDir={logSaveDir}
        onSelectLogDir={handleSelectLogDir}
        onHciExtract={handleHciExtract}
        onHciExtractFromPath={handleHciExtractFromPath}
        fontSize={fontSize}
        onFontSizeChange={setFontSize}
        maxCacheKb={maxCacheKb}
        onMaxCacheKbChange={setMaxCacheKb}
        pollIntervalMs={pollIntervalMs}
        onPollIntervalMsChange={setPollIntervalMs}
        hciTimezoneOffset={hciTimezoneOffset}
        onHciTimezoneOffsetChange={setHciTimezoneOffset}
        wpsPath={wpsPath}
        liveImportEnabled={liveImportEnabled}
        liveImportStatus={liveImportStatus}
        liveImportStats={liveImportStats}
        onLiveImportToggle={handleLiveImportToggle}
        onSelectWpsDir={handleSelectWpsDir}
        connectedPorts={connectedPorts}
        liveImportSelectedPort={liveImportSelectedPort}
        onLiveImportSelectPort={setLiveImportSelectedPort}
        onOpenLc3ToolKit={handleOpenLc3ToolKit}
        onOpenUsbAudio={handleOpenUsbAudio}
        portTypes={portTypes}
        portDescriptions={portDescriptions}
      />

      <ConfigPanel
        isOpen={configPanelOpen}
        onClose={() => setConfigPanelOpen(false)}
        configTree={dcfInfo?.config_tree || []}
        infoData={dcfInfo?.info_data || []}
        onSave={handleSaveConfig}
      />

      <ToastContainer toasts={toasts} onDismiss={(id) => setToasts(prev => prev.filter(t => t.id !== id))} />

      <ConfirmDialog
        isOpen={!!hciExtractConfirm}
        title="HCI日志提取"
        message="确认从以下日志提取 HCI 数据包？"
        fileName={hciExtractConfirm?.fileName}
        detail={hciExtractConfirm?.path}
        hint="将在同目录生成 .cfa 文件"
        confirmLabel="提取"
        cancelLabel="取消"
        onConfirm={handleHciExtractConfirm}
        onCancel={handleHciExtractCancel}
      />

      <FilterSettingsDialog
        isOpen={filterDialogOpen}
        rules={filterRules}
        initialKeyword={filterDialogDraft?.keyword}
        initialKeywordRequestId={filterDialogDraft?.requestId}
        onAddRule={handleAddRule}
        onUpdateRule={handleUpdateRule}
        onDeleteRule={handleDeleteRule}
        onClose={() => {
          setFilterDialogOpen(false);
          setFilterDialogDraft(null);
        }}
      />

      <HighlightSettingsDialog
        isOpen={highlightDialogOpen}
        rules={highlightRules}
        initialKeyword={highlightDialogDraft?.keyword}
        initialKeywordRequestId={highlightDialogDraft?.requestId}
        onAddRule={handleAddHighlightRule}
        onUpdateRule={handleUpdateHighlightRule}
        onDeleteRule={handleDeleteHighlightRule}
        onClose={() => {
          setHighlightDialogOpen(false);
          setHighlightDialogDraft(null);
        }}
      />


      <USBAudioExtractorDialog
        isOpen={usbAudioDialogOpen}
        onClose={() => setUsbAudioDialogOpen(false)}
        onToast={showToast}
      />

      <StatusBar
        isConnected={connectedPorts.length > 0}
        ports={connectedPorts}
        device={connectedPorts.length > 0 ? '蓝牙芯片' : undefined}
      />
    </div>
  );
}
