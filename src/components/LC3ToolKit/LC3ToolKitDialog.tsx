import { useState, useEffect, useRef, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { motion, AnimatePresence } from 'motion/react';
import { X, FolderOpen, ChevronDown, Circle, Bluetooth, Usb, Cable, RefreshCw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

interface Lc3DecodeResult { success: boolean; frame_count: number; frame_samples: number; duration_secs: number; leftover_bytes: number; saved_files: string[]; error: string | null; }
interface Lc3CaptureStatus { port: string; byte_count: number; active: boolean; }

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onToast: (msg: string) => void;
  portTypes: Record<string, string>;
  portDescriptions: Record<string, string>;
  onPortSelect: (port: string) => void;
  availablePorts: string[];
  connectedPorts: string[];
  onRefreshPorts: () => void;
  onStealPort: (port: string) => void;
}

function getPortIcon(port: string, portTypes: Record<string,string>, portDescriptions: Record<string,string>) {
  const type = portTypes[port] || 'unknown';
  const desc = (portDescriptions[port] || '').toLowerCase();
  if (type === 'bluetooth' || desc.includes('bluetooth') || desc.includes('bth'))
    return <Bluetooth className="w-3.5 h-3.5 text-blue-500 shrink-0" />;
  if (type === 'usb') return <Usb className="w-3.5 h-3.5 text-muted-foreground shrink-0" />;
  return <Cable className="w-3.5 h-3.5 text-muted-foreground shrink-0" />;
}

const MENU_STYLE = {
  background: 'var(--acrylic-tint)',
  backdropFilter: 'blur(40px) saturate(150%)',
  WebkitBackdropFilter: 'blur(40px) saturate(150%)',
  border: '1px solid var(--acrylic-border)',
  boxShadow: 'var(--shadow-lg)',
} as const;

const ITEM_BG = 'bg-black/5 dark:bg-white/5';

function ParamDropdown({ label, value, options, unit, onChange }: {
  label: string; value: string; options: string[]; unit?: string; onChange: (v: string) => void;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const [customMode, setCustomMode] = useState(false);
  const [customInput, setCustomInput] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen) { setCustomMode(false); setCustomInput(''); return; }
    if (customMode) setTimeout(() => inputRef.current?.focus(), 50);
  }, [isOpen, customMode]);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        if (customMode && customInput.trim()) onChange(`${customInput.trim()}${unit || ''}`);
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [customMode, customInput, unit, onChange]);

  const handleSelect = (opt: string) => {
    if (opt === '其他') { setCustomMode(true); return; }
    onChange(opt);
    setIsOpen(false);
  };

  const handleCustomKey = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleCustomConfirm();
    if (e.key === 'Escape') { setCustomMode(false); setCustomInput(''); }
  };

  const handleCustomConfirm = () => {
    if (customInput.trim()) onChange(`${customInput.trim()}${unit || ''}`);
    else onChange(options[0]);
    setIsOpen(false);
  };

  return (
    <div className="flex-1 min-w-0" ref={ref}>
      <label className="text-[10px] text-muted-foreground mb-0.5 block">{label}</label>
      <div className="relative">
        <button onClick={() => setIsOpen(!isOpen)}
          className={`w-full flex items-center justify-between px-2 py-1.5 rounded-lg text-[11px] transition-colors border border-border/30 ${ITEM_BG} hover:bg-black/10 dark:hover:bg-white/10`}>
          <span className={value ? 'text-foreground' : 'text-muted-foreground'}>{value || `选择${label}`}</span>
          <ChevronDown className={`w-3 h-3 transition-transform shrink-0 ${isOpen ? 'rotate-180' : ''}`} />
        </button>
        {isOpen && (
          <div className="absolute top-full left-0 mt-1 w-full min-w-[120px] rounded-xl overflow-hidden z-50" style={MENU_STYLE}>
            {options.map(opt => {
              if (opt === '其他' && customMode) {
                return (
                  <div key="custom" className="px-2 py-1">
                    <input ref={inputRef} placeholder={unit} value={customInput}
                      onChange={e => setCustomInput(e.target.value)} onKeyDown={handleCustomKey} onBlur={handleCustomConfirm}
                      className={`w-full px-2 py-1 rounded text-[11px] border border-border/50 focus:outline-none focus:ring-1 focus:ring-primary/50 ${ITEM_BG}`} />
                  </div>
                );
              }
              return (
                <button key={opt} onClick={() => handleSelect(opt)}
                  className={`w-full flex items-center gap-2 px-3 py-1.5 text-xs text-left hover:bg-accent/20 transition-colors cursor-pointer ${value === opt ? 'bg-accent/40 text-foreground' : 'text-muted-foreground'}`}>
                  {opt}
                </button>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

const INITIAL_PARAMS = {
  selectedPort: '', capturing: false, capturedData: [] as number[],
  byteCount: 0, capturedCount: 0, format: 'LC3', channels: '1',
  frameMs: '10ms', sampleRate: '48KHz', bitrate: '80Kbps',
  outputDir: '', saveRaw: true, saveWav: true, decoding: false, logs: [] as string[],
};

const DEFAULT_BAUD_RATES = [9600, 115200, 1500000, 6000000];

export function LC3ToolKitDialog({ isOpen, onClose, onToast, portTypes, portDescriptions, onPortSelect, availablePorts, connectedPorts: mainConnectedPorts, onRefreshPorts, onStealPort }: Props) {
  const [lc3ConnectedPorts, setLc3ConnectedPorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState(INITIAL_PARAMS.selectedPort);
  const [baudRate, setBaudRate] = useState(6000000);
  const [capturing, setCapturing] = useState(INITIAL_PARAMS.capturing);
  const [capturedData, setCapturedData] = useState<number[]>(INITIAL_PARAMS.capturedData);
  const [byteCount, setByteCount] = useState(INITIAL_PARAMS.byteCount);
  const [capturedCount, setCapturedCount] = useState(INITIAL_PARAMS.capturedCount);

  const [format, setFormat] = useState(INITIAL_PARAMS.format);
  const [channels, setChannels] = useState(INITIAL_PARAMS.channels);
  const [frameMs, setFrameMs] = useState(INITIAL_PARAMS.frameMs);
  const [sampleRate, setSampleRate] = useState(INITIAL_PARAMS.sampleRate);
  const [bitrate, setBitrate] = useState(INITIAL_PARAMS.bitrate);
  const [outputDir, setOutputDir] = useState(INITIAL_PARAMS.outputDir);
  const [saveRaw, setSaveRaw] = useState(INITIAL_PARAMS.saveRaw);
  const [saveWav, setSaveWav] = useState(INITIAL_PARAMS.saveWav);
  const [decoding, setDecoding] = useState(INITIAL_PARAMS.decoding);

  const [portMenuOpen, setPortMenuOpen] = useState(false);
  const [baudMenuOpen, setBaudMenuOpen] = useState(false);
  const portRef = useRef<HTMLDivElement>(null);
  const baudRef = useRef<HTMLDivElement>(null);
  const [logs, setLogs] = useState<string[]>(INITIAL_PARAMS.logs);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const logEndRef = useRef<HTMLDivElement>(null);
  const prevOpenRef = useRef(isOpen);

  // Drag state
  const [dragPos, setDragPos] = useState({ x: 0, y: 0 });
  const [dragging, setDragging] = useState(false);
  const dragOffset = useRef({ x: 0, y: 0 });
  const [initialized, setInitialized] = useState(false);

  // Center on first open
  useEffect(() => {
    if (isOpen && !initialized) {
      setDragPos({ x: Math.max(0, (window.innerWidth - 460) / 2), y: Math.max(60, (window.innerHeight - 600) / 3) });
      setInitialized(true);
    }
  }, [isOpen, initialized]);

  // Drag handlers
  const handleDragStart = useCallback((e: React.PointerEvent) => {
    e.preventDefault();
    setDragging(true);
    dragOffset.current = { x: e.clientX - dragPos.x, y: e.clientY - dragPos.y };
    // Capture pointer for reliable drag tracking even when mouse leaves window
    (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
  }, [dragPos]);

  useEffect(() => {
    if (!dragging) return;
    const onMove = (e: MouseEvent) => {
      setDragPos({ x: e.clientX - dragOffset.current.x, y: e.clientY - dragOffset.current.y });
    };
    const onUp = () => setDragging(false);
    // Use both mouse and pointer events for reliability across Tauri webview
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
    document.addEventListener('pointermove', onMove as EventListener);
    document.addEventListener('pointerup', onUp);
    window.addEventListener('blur', onUp);
    return () => {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      document.removeEventListener('pointermove', onMove as EventListener);
      document.removeEventListener('pointerup', onUp);
      window.removeEventListener('blur', onUp);
    };
  }, [dragging]);

  const resetAll = useCallback(() => {
    if (pollRef.current) { clearInterval(pollRef.current); pollRef.current = null; }
    if (capturing) invoke('lc3_stop_capture', { port: selectedPort }).catch(() => {});
    setSelectedPort(INITIAL_PARAMS.selectedPort);
    setCapturing(INITIAL_PARAMS.capturing);
    setCapturedData(INITIAL_PARAMS.capturedData);
    setByteCount(INITIAL_PARAMS.byteCount);
    setCapturedCount(INITIAL_PARAMS.capturedCount);
    setFormat(INITIAL_PARAMS.format);
    setChannels(INITIAL_PARAMS.channels);
    setFrameMs(INITIAL_PARAMS.frameMs);
    setSampleRate(INITIAL_PARAMS.sampleRate);
    setBitrate(INITIAL_PARAMS.bitrate);
    setOutputDir(INITIAL_PARAMS.outputDir);
    setSaveRaw(INITIAL_PARAMS.saveRaw);
    setSaveWav(INITIAL_PARAMS.saveWav);
    setDecoding(INITIAL_PARAMS.decoding);
    setLogs(INITIAL_PARAMS.logs);
    setPortMenuOpen(false);
    setBaudMenuOpen(false);
    onPortSelect('');
  }, [capturing, selectedPort, onPortSelect]);

  useEffect(() => {
    if (prevOpenRef.current && !isOpen) resetAll();
    prevOpenRef.current = isOpen;
  }, [isOpen, resetAll]);

  const addLog = useCallback((msg: string) => {
    const t = new Date().toLocaleTimeString('zh-CN', { hour12: false });
    setLogs(prev => [...prev.slice(-300), `[${t}] ${msg}`]);
  }, []);

  useEffect(() => { logEndRef.current?.scrollIntoView({ behavior: 'smooth' }); }, [logs]);

  // Close dropdowns on outside click
  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (portRef.current && !portRef.current.contains(e.target as Node)) setPortMenuOpen(false);
      if (baudRef.current && !baudRef.current.contains(e.target as Node)) setBaudMenuOpen(false);
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  const refreshLc3Ports = useCallback(async () => {
    try { setLc3ConnectedPorts(await invoke<string[]>('lc3_get_connected_ports')); } catch {}
  }, []);

  useEffect(() => { if (isOpen) refreshLc3Ports(); }, [isOpen, refreshLc3Ports]);

  const handleConnectPort = (port: string) => {
    // If port is used by main panel, steal it
    if (mainConnectedPorts.includes(port)) {
      onStealPort(port);
      addLog(`已从主面板抢占串口 ${port}`);
    }
    setSelectedPort(port);
    onPortSelect(port);
    setPortMenuOpen(false);
  };

  const handleDisconnectPort = () => {
    if (capturing) {
      onToast('请先停止捕获');
      return;
    }
    setSelectedPort('');
    onPortSelect('');
  };

  const handleBaudSelect = (rate: number) => {
    setBaudRate(rate);
    setBaudMenuOpen(false);
  };

  const handleCaptureToggle = async () => {
    if (capturing) {
      if (pollRef.current) { clearInterval(pollRef.current); pollRef.current = null; }
      try {
        const data = await invoke<number[]>('lc3_stop_capture', { port: selectedPort });
        setCapturedData(data); setCapturedCount(data.length);
        addLog(`停止捕获, 共收到 ${data.length.toLocaleString()} 字节`);
      } catch (e) { addLog(`停止捕获失败: ${e}`); }
      setCapturing(false);
    } else {
      if (!selectedPort) { onToast('请先选择串口'); return; }
      try {
        await invoke('lc3_start_capture', { port: selectedPort });
        setCapturing(true); setByteCount(0);
        addLog(`开始捕获: ${selectedPort}`);
        pollRef.current = setInterval(async () => {
          try {
            const status = await invoke<Lc3CaptureStatus>('lc3_get_capture_status', { port: selectedPort });
            setByteCount(status.byte_count);
          } catch {}
        }, 200);
      } catch (e) { addLog(`开始捕获失败: ${e}`); }
    }
  };

  const handleImportFile = async () => {
    try {
      const path = await openDialog({ multiple: false, filters: [{ name: 'LC3 数据文件', extensions: ['bin', 'txt'] }] });
      if (!path) return;
      const filePath = path as string;
      const data = await invoke<number[]>('lc3_import_file', { path: filePath });
      setCapturedData(data); setCapturedCount(data.length);
      const fileDir = filePath.substring(0, Math.max(0, filePath.lastIndexOf('\\')));
      if (fileDir) setOutputDir(fileDir);
      addLog(`已导入: ${filePath.split(/[/\\]/).pop()} (${data.length.toLocaleString()} 字节)`);
    } catch (e) { addLog(`导入文件失败: ${e}`); }
  };

  const handleSelectOutputDir = async () => {
    try {
      const dir = await openDialog({ directory: true, multiple: false, defaultPath: outputDir || undefined });
      if (dir) setOutputDir(dir as string);
    } catch {}
  };

  useEffect(() => {
    if (!selectedPort || !isOpen || capturing) {
      if (pollRef.current) { clearInterval(pollRef.current); pollRef.current = null; }
      return;
    }
    pollRef.current = setInterval(async () => {
      try { await invoke<number[]>('read_serial_data', { port: selectedPort, timestampEnabled: false, maxLines: 1000, panel: 0 }); } catch {}
    }, 100);
    return () => { if (pollRef.current) clearInterval(pollRef.current); };
  }, [selectedPort, isOpen, capturing]);

  useEffect(() => {
    return () => { if (pollRef.current) clearInterval(pollRef.current); };
  }, []);

  const handleDecode = async () => {
    if (capturedData.length === 0) { onToast('没有可解码的数据'); return; }
    if (!outputDir) { onToast('请选择输出目录'); return; }

    const sr = parseInt(sampleRate.replace(/KHz|kHz/i, '')) * 1000;
    const br = parseInt(bitrate.replace(/Kbps|kbps/i, '')) * 1000;
    const fms = parseFloat(frameMs.replace('ms', ''));
    const nch = parseInt(channels);
    const hrmode = format === 'LC3 Plus HR';
    if (isNaN(sr) || sr <= 0) { onToast('无效的采样率'); return; }
    if (isNaN(br) || br <= 0) { onToast('无效的比特率'); return; }
    if (isNaN(fms) || fms <= 0) { onToast('无效的帧时长'); return; }

    setDecoding(true);
    const now = new Date();
    const pad = (n: number) => String(n).padStart(2, '0');
    const ts = `${now.getFullYear()}${pad(now.getMonth()+1)}${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
    const baseName = `LC3_${ts}`;
    addLog(`开始解码: ${fms}ms, ${sr/1000}KHz, ${br/1000}Kbps, ${nch}ch${hrmode?' HR':''}`);

    try {
      const result = await invoke<Lc3DecodeResult>('lc3_decode_and_export', {
        data: capturedData, sampleRate: sr, numChannels: nch, bitrate: br,
        frameDurationMs: fms, hrmode, outputDir, baseName, saveRaw, saveWav,
      });
      if (result.success) {
        addLog(`解码完成: ${result.frame_count} 帧, ${result.duration_secs.toFixed(2)}s`);
        if (result.leftover_bytes > 0) addLog(`尾部剩余 ${result.leftover_bytes} 字节`);
        if (result.saved_files.length > 0) addLog(`已保存: ${result.saved_files.join(', ')}`);
        onToast(`LC3 解码完成: ${result.frame_count} 帧`);
      } else { addLog(`解码失败: ${result.error || '未知错误'}`); }
    } catch (e) { addLog(`解码失败: ${e}`); }
    setDecoding(false);
  };

  const isPortLc3Connected = lc3ConnectedPorts.includes(selectedPort);

  const extractFrameBytes = (): number => {
    try {
      const br = parseInt(bitrate.replace(/Kbps|kbps/i, '')) * 1000;
      const fms = parseFloat(frameMs.replace('ms', ''));
      const nch = parseInt(channels);
      if (br > 0 && fms > 0) return Math.ceil((br * fms / 8000) * nch);
    } catch {}
    return 0;
  };

  const frameBytesVal = extractFrameBytes();
  const estimatedFrames = frameBytesVal > 0 ? Math.floor(capturedCount / frameBytesVal) : 0;

  if (!isOpen) return null;

  return createPortal(
    <div
      className="fixed z-[401] pointer-events-none"
      style={{ left: dragPos.x, top: dragPos.y, width: 460 }}
    >
      <div
        className="pointer-events-auto rounded-xl overflow-hidden"
        style={{
          background: 'var(--acrylic-tint)',
          backdropFilter: 'blur(60px) saturate(180%)',
          WebkitBackdropFilter: 'blur(60px) saturate(180%)',
          border: '1px solid var(--acrylic-border)',
          boxShadow: 'var(--shadow-lg)',
        }}
      >
        {/* === 标题栏（可拖拽） === */}
        <div
          className="px-3 py-2 border-b border-border/50 flex items-center gap-1.5 cursor-move select-none"
          onPointerDown={handleDragStart}
          style={{ background: 'rgba(128,128,128,0.05)' }}
        >
          {/* Port selector — matching SecondaryToolbar style */}
          <div className="relative flex items-center" ref={portRef}>
            <button
              onClick={(e) => { e.stopPropagation(); setPortMenuOpen(!portMenuOpen); }}
              className="pl-2.5 pr-8 py-1.5 rounded-xl bg-input-background border border-border text-xs focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-w-[140px] flex items-center justify-between"
            >
              <div className="flex items-center gap-1.5">
                {selectedPort ? (
                  <>
                    {getPortIcon(selectedPort, portTypes, portDescriptions)}
                    <span className="text-foreground">{selectedPort}</span>
                    {isPortLc3Connected && <Circle className="w-2 h-2 fill-green-500 text-green-500" />}
                  </>
                ) : (
                  <>
                    <Usb className="w-3.5 h-3.5 text-primary" />
                    <span className="text-muted-foreground">选择串口</span>
                  </>
                )}
              </div>
              <div className="absolute right-2 flex items-center gap-0.5">
                <motion.button
                  whileHover={{ scale: 1.15, rotate: 180 }}
                  whileTap={{ scale: 0.9 }}
                  onClick={(e) => { e.stopPropagation(); onRefreshPorts(); }}
                  className="p-0.5 hover:bg-accent/30 rounded transition-colors"
                >
                  <RefreshCw className="w-3 h-3 text-muted-foreground" />
                </motion.button>
                <ChevronDown className={`w-3 h-3 text-muted-foreground transition-transform duration-200 ${portMenuOpen ? 'rotate-180' : ''}`} />
              </div>
            </button>

            <AnimatePresence>
              {portMenuOpen && (
                <motion.div
                  initial={{ opacity: 0, y: -10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -10 }}
                  transition={{ duration: 0.2, ease: [0.2, 0.8, 0.2, 1] }}
                  className="absolute top-full mt-1 left-0 w-[200px] rounded-xl overflow-hidden"
                  style={{ zIndex: 99999, ...MENU_STYLE }}
                >
                  <div className="py-1">
                    {availablePorts.map(port => {
                      const isMainConn = mainConnectedPorts.includes(port);
                      const isLc3Conn = lc3ConnectedPorts.includes(port);
                      const isSelected = selectedPort === port;
                      return (
                        <div
                          key={port}
                          className="px-3 py-2 hover:bg-accent/30 transition-colors flex items-center justify-between"
                        >
                          <div className="flex items-center gap-2">
                            <Circle className={`w-2.5 h-2.5 ${isLc3Conn ? 'fill-green-500 text-green-500' : 'fill-muted-foreground text-muted-foreground'}`} />
                            <span className="text-sm">{port}</span>
                            {getPortIcon(port, portTypes, portDescriptions)}
                            {isMainConn && !isSelected && <span className="text-[10px] text-yellow-500">主面板</span>}
                          </div>
                          <div
                            onClick={(e) => {
                              e.stopPropagation();
                              if (isSelected) { handleDisconnectPort(); }
                              else { handleConnectPort(port); }
                            }}
                            className={`w-9 h-5 rounded-full cursor-pointer transition-colors relative shrink-0 ${isSelected ? 'bg-primary' : 'bg-muted'}`}
                          >
                            <div className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-all ${isSelected ? 'left-[18px]' : 'left-0.5'}`} />
                          </div>
                        </div>
                      );
                    })}
                    {availablePorts.length === 0 && (
                      <div className="px-3 py-2 text-sm text-muted-foreground">无可用串口</div>
                    )}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>

          {/* Baud rate selector — matching SecondaryToolbar style */}
          <div className="relative flex items-center gap-2 ml-1" ref={baudRef}>
            <span className="text-sm text-muted-foreground font-medium">波特率</span>
            <button
              onClick={(e) => { e.stopPropagation(); setBaudMenuOpen(!baudMenuOpen); }}
              className="pl-2 pr-6 py-1.5 rounded-xl bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-w-[100px] flex items-center justify-between"
            >
              <span className="text-foreground">{baudRate}</span>
              <ChevronDown className={`w-3 h-3 text-muted-foreground transition-transform duration-200 ${baudMenuOpen ? 'rotate-180' : ''}`} />
            </button>

            <AnimatePresence>
              {baudMenuOpen && (
                <motion.div
                  initial={{ opacity: 0, y: -10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -10 }}
                  transition={{ duration: 0.2, ease: [0.2, 0.8, 0.2, 1] }}
                  className="absolute top-full mt-1 left-0 min-w-[100px] rounded-xl overflow-hidden"
                  style={{ zIndex: 99999, ...MENU_STYLE }}
                >
                  <div className="py-1">
                    {DEFAULT_BAUD_RATES.map(rate => (
                      <button
                        key={rate}
                        onClick={(e) => { e.stopPropagation(); handleBaudSelect(rate); }}
                        className={`w-full px-3 py-1.5 text-xs text-left hover:bg-accent/30 transition-colors ${baudRate === rate ? 'text-primary font-medium' : ''}`}
                      >
                        {rate}
                      </button>
                    ))}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>

          <div className="flex-1" />

          <motion.button whileHover={{ scale: 1.1 }} whileTap={{ scale: 0.9 }}
            onClick={() => { resetAll(); onClose(); }}
            className="p-0.5 rounded-lg hover:bg-accent/30 transition-colors shrink-0">
            <X className="w-4 h-4" />
          </motion.button>
        </div>

        {/* === 参数区域 === */}
        <div className="p-4 space-y-2.5">
          <div className="flex gap-2">
            <ParamDropdown label="Format" value={format} options={['LC3', 'LC3 Plus', 'LC3 Plus HR']} onChange={setFormat} />
            <ParamDropdown label="Ch" value={channels} options={['1', '2']} onChange={setChannels} />
            <ParamDropdown label="Frame" value={frameMs} options={['2.5ms', '5ms', '7.5ms', '10ms']} onChange={setFrameMs} />
          </div>

          <div className="flex gap-2">
            <ParamDropdown label="Samplerate" value={sampleRate} options={['16KHz', '24KHz', '32KHz', '48KHz', '其他']} unit="KHz" onChange={setSampleRate} />
            <ParamDropdown label="Bitrate" value={bitrate} options={['64Kbps', '80Kbps', '96Kbps', '其他']} unit="Kbps" onChange={setBitrate} />
          </div>

          <div className="h-px bg-border/50" />

          <motion.button
            whileHover={{ scale: 1.01 }}
            onClick={handleImportFile}
            disabled={capturing}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors hover:bg-black/10 dark:hover:bg-white/10 disabled:opacity-40 cursor-pointer group">
            <motion.div whileHover={{ scale: 1.15 }}>
              <FolderOpen className="w-4 h-4 text-muted-foreground" />
            </motion.div>
            <span className="text-muted-foreground">本地解码</span>
          </motion.button>

          <motion.button
            whileHover={{ scale: 1.01 }}
            onClick={handleSelectOutputDir}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors hover:bg-black/10 dark:hover:bg-white/10 cursor-pointer group">
            <motion.div whileHover={{ scale: 1.15 }}>
              <FolderOpen className="w-4 h-4 shrink-0 text-muted-foreground" />
            </motion.div>
            <span className="truncate text-left text-muted-foreground">
              {outputDir || '选择输出目录'}
            </span>
          </motion.button>

          <div className="flex items-center gap-3 text-[11px]">
            <label className="flex items-center gap-1 text-muted-foreground cursor-pointer select-none">
              <input type="checkbox" checked={saveRaw} onChange={e => setSaveRaw(e.target.checked)} className="w-3 h-3 rounded accent-primary" />
              RAW (.bin)
            </label>
            <label className="flex items-center gap-1 text-muted-foreground cursor-pointer select-none">
              <input type="checkbox" checked={saveWav} onChange={e => setSaveWav(e.target.checked)} className="w-3 h-3 rounded accent-primary" />
              WAV (.wav)
            </label>
          </div>

          <div className="h-px bg-border/50" />

          <div className="flex items-center justify-between text-[11px] text-muted-foreground">
            <span>已收: {capturing ? byteCount.toLocaleString() : capturedCount.toLocaleString()} 字节</span>
            <span>帧数: {estimatedFrames > 0 ? `~${estimatedFrames.toLocaleString()}` : '-'}</span>
          </div>

          {decoding && (
            <div className="h-1.5 bg-muted/50 rounded-full overflow-hidden">
              <motion.div className="h-full bg-gradient-to-r from-primary to-[#5B4FD6] rounded-full"
                animate={{ width: ['5%', '90%'] }}
                transition={{ duration: 6, ease: 'easeInOut', repeat: Infinity }} />
            </div>
          )}

          <div className="h-px bg-border/50" />

          <div className={`h-[68px] overflow-y-auto rounded-lg p-2 text-[10px] font-mono leading-relaxed ${ITEM_BG}`}>
            {logs.length === 0 ? (
              <div className="text-muted-foreground/30">日志输出...</div>
            ) : (
              logs.map((l, i) => <div key={i} className="whitespace-pre-wrap break-all">{l}</div>)
            )}
            <div ref={logEndRef} />
          </div>

          <div className="flex gap-2">
            <button
              onClick={handleCaptureToggle}
              disabled={!selectedPort}
              className={`flex-1 flex items-center justify-center gap-2 py-2 rounded-lg text-sm font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed ${
                capturing ? 'bg-red-500 hover:bg-red-600 text-white' : 'bg-green-500 hover:bg-green-600 text-white'
              }`}>
              {capturing ? '■ 停止捕获' : '▶ 开始捕获'}
            </button>

            <button onClick={handleDecode}
              disabled={capturedData.length === 0 || decoding || !outputDir}
              className="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg bg-primary hover:bg-primary/90 text-white text-sm font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed">
              {decoding ? '解码中...' : '▲ 解码导出'}
            </button>
          </div>
        </div>
      </div>
    </div>,
    document.body
  );
}
