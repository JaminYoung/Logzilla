import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { emit, listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { FolderOpen, ChevronDown, Circle, Bluetooth, Usb, Cable, RefreshCw, X, Zap } from 'lucide-react';
import { motion, AnimatePresence } from 'motion/react';

interface Lc3DecodeResult { success: boolean; frame_count: number; frame_samples: number; duration_secs: number; leftover_bytes: number; saved_files: string[]; error: string | null; }
interface Lc3CaptureStatus { port: string; byte_count: number; active: boolean; }
interface SerialPortInfo { port_name: string; description: string; vid: number | null; pid: number | null; port_type: string; }

function getPortIcon(type: string, desc: string) {
  const d = desc.toLowerCase();
  if (type === 'bluetooth' || d.includes('bluetooth') || d.includes('bth'))
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

const DEFAULT_BAUD_RATES = [9600, 115200, 1500000, 6000000];

export function LC3ToolKitWindow() {
  const [availablePorts, setAvailablePorts] = useState<SerialPortInfo[]>([]);
  const [connectedPorts, setConnectedPorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState('');
  const [baudRate, setBaudRate] = useState(6000000);
  const [customBaud, setCustomBaud] = useState('');
  const [capturing, setCapturing] = useState(false);
  const [capturedData, setCapturedData] = useState<number[]>([]);
  const [byteCount, setByteCount] = useState(0);
  const [capturedCount, setCapturedCount] = useState(0);

  const [format, setFormat] = useState('LC3');
  const [channels, setChannels] = useState('1');
  const [frameMs, setFrameMs] = useState('10ms');
  const [sampleRate, setSampleRate] = useState('48KHz');
  const [bitrate, setBitrate] = useState('80Kbps');
  const [outputDir, setOutputDir] = useState('');
  const [saveRaw, setSaveRaw] = useState(true);
  const [saveWav, setSaveWav] = useState(true);
  const [decoding, setDecoding] = useState(false);

  const [portMenuOpen, setPortMenuOpen] = useState(false);
  const [baudMenuOpen, setBaudMenuOpen] = useState(false);
  const portRef = useRef<HTMLDivElement>(null);
  const baudRef = useRef<HTMLDivElement>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const logEndRef = useRef<HTMLDivElement>(null);

  // Sync dark/light theme with main window
  useEffect(() => {
    const savedTheme = localStorage.getItem('theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const isDark = savedTheme ? savedTheme === 'dark' : prefersDark;
    document.documentElement.classList.toggle('dark', isDark);

    const unlisten = listen<{ theme: string }>('theme-changed', (event) => {
      document.documentElement.classList.toggle('dark', event.payload.theme === 'dark');
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

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

  // Refresh available ports
  const refreshPorts = useCallback(async () => {
    try {
      const ports = await invoke<SerialPortInfo[]>('list_serial_ports');
      setAvailablePorts(ports);
    } catch {}
  }, []);

  // Refresh connected ports (LC3-specific)
  const refreshLc3Ports = useCallback(async () => {
    try { setConnectedPorts(await invoke<string[]>('lc3_get_connected_ports')); } catch {}
  }, []);

  useEffect(() => {
    refreshPorts();
    refreshLc3Ports();
  }, [refreshPorts, refreshLc3Ports]);

  // Handle port connect/disconnect with auto-steal from main window
  const handleTogglePort = async (portName: string) => {
    const isConnected = connectedPorts.includes(portName);
    if (isConnected) {
      // Disconnect
      if (capturing) { addLog('请先停止捕获'); return; }
      setSelectedPort('');
      await emit('lc3-port-changed', { port: '', action: 'disconnect' });
    } else {
      // Connect — steal from main window if needed
      try {
        await invoke('open_serial_port', { port: portName, baudrate: baudRate });
        setSelectedPort(portName);
        addLog(`已连接: ${portName}`);
        await emit('lc3-port-changed', { port: portName, action: 'steal' });
      } catch (e) {
        // Port might be used by main window, try close first then reopen
        try {
          await invoke('close_serial_port', { port: portName });
          await invoke('open_serial_port', { port: portName, baudrate: baudRate });
          setSelectedPort(portName);
          addLog(`已从主面板抢占串口: ${portName}`);
          await emit('lc3-port-changed', { port: portName, action: 'steal' });
        } catch (e2) {
          addLog(`连接失败: ${e2}`);
        }
      }
    }
    setPortMenuOpen(false);
  };

  const handleBaudSelect = (rate: number) => {
    setBaudRate(rate);
    setBaudMenuOpen(false);
    setCustomBaud('');
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
      if (!selectedPort) { addLog('请先选择串口'); return; }
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
    if (!selectedPort || capturing) {
      if (pollRef.current) { clearInterval(pollRef.current); pollRef.current = null; }
      return;
    }
    pollRef.current = setInterval(async () => {
      try { await invoke<number[]>('read_serial_data', { port: selectedPort, timestampEnabled: false, maxLines: 1000, panel: 0 }); } catch {}
    }, 100);
    return () => { if (pollRef.current) clearInterval(pollRef.current); };
  }, [selectedPort, capturing]);

  useEffect(() => {
    return () => { if (pollRef.current) clearInterval(pollRef.current); };
  }, []);

  const handleDecode = async () => {
    if (capturedData.length === 0) { addLog('没有可解码的数据'); return; }
    if (!outputDir) { addLog('请选择输出目录'); return; }

    const sr = parseInt(sampleRate.replace(/KHz|kHz/i, '')) * 1000;
    const br = parseInt(bitrate.replace(/Kbps|kbps/i, '')) * 1000;
    const fms = parseFloat(frameMs.replace('ms', ''));
    const nch = parseInt(channels);
    const hrmode = format === 'LC3 Plus HR';
    if (isNaN(sr) || sr <= 0) { addLog('无效的采样率'); return; }
    if (isNaN(br) || br <= 0) { addLog('无效的比特率'); return; }
    if (isNaN(fms) || fms <= 0) { addLog('无效的帧时长'); return; }

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
      } else { addLog(`解码失败: ${result.error || '未知错误'}`); }
    } catch (e) { addLog(`解码失败: ${e}`); }
    setDecoding(false);
  };

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

  return (
    <div className="w-screen h-screen flex flex-col bg-background text-foreground overflow-hidden select-none">
      {/* === Title bar === */}
      <div
        className="h-10 flex items-center justify-between select-none border-b border-border/50 shrink-0"
        style={{ background: 'var(--acrylic-tint)', backdropFilter: 'blur(40px) saturate(150%)', WebkitBackdropFilter: 'blur(40px) saturate(150%)' }}
      >
        <div className="flex items-center gap-2 pl-3 flex-1 h-full cursor-default" onMouseDown={() => getCurrentWindow().startDragging()}>
          <div className="flex items-center justify-center w-6 h-6 rounded-md bg-gradient-to-br from-primary to-[#5B4FD6] text-white">
            <Zap className="w-3.5 h-3.5" />
          </div>
          <span className="text-sm font-semibold">LC3 Tool Kit</span>
        </div>
        <button
          className="w-10 h-10 flex items-center justify-center transition-colors hover:bg-[#c42b1c] hover:text-white cursor-pointer shrink-0"
          onClick={async () => { try { await getCurrentWindow().close(); } catch(e) { console.error('close error', e); } }}
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* === Content area === */}
      <div className="flex-1 min-h-0 px-4 pt-4 pb-4 flex flex-col overflow-hidden">
        {/* Port & Baud selectors */}
        <div className="flex items-center gap-7 mb-2.5 shrink-0">
          {/* Port selector */}
          <div className="relative" ref={portRef}>
            <button
              onClick={() => setPortMenuOpen(!portMenuOpen)}
              className="pl-3 pr-8 py-2 rounded-xl bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-w-[180px] flex items-center justify-between cursor-pointer"
            >
              <div className="flex items-center gap-2">
                <Usb className="w-4 h-4 text-primary" />
                <span className="text-foreground">{selectedPort || '选择串口...'}</span>
              </div>
                <div className="absolute right-2 flex items-center gap-0.5">
                  <motion.button
                    whileHover={{ scale: 1.15, rotate: 180 }}
                    whileTap={{ scale: 0.9 }}
                    onClick={(e) => { e.stopPropagation(); refreshPorts(); }}
                    className="p-0.5 hover:bg-accent/30 rounded transition-colors cursor-pointer"
                  >
                    <RefreshCw className="w-3.5 h-3.5 text-muted-foreground" />
                  </motion.button>
                  <ChevronDown className={`w-4 h-4 text-muted-foreground transition-transform duration-200 ${portMenuOpen ? 'rotate-180' : ''}`} />
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
                      const isConnected = connectedPorts.includes(port.port_name);
                      const isSelected = selectedPort === port.port_name;
                      return (
                        <div
                          key={port.port_name}
                          onClick={() => handleTogglePort(port.port_name)}
                          className="px-3 py-2 hover:bg-accent/30 transition-colors flex items-center justify-between cursor-pointer"
                        >
                          <div className="flex items-center gap-2">
                            <Circle className={`w-2.5 h-2.5 ${isConnected ? 'fill-green-500 text-green-500' : 'fill-muted-foreground text-muted-foreground'}`} />
                            <span className="text-sm">{port.port_name}</span>
                            {getPortIcon(port.port_type, port.description)}
                          </div>
                          <div
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

          {/* Baud rate selector */}
          <div className="relative flex items-center gap-1.5" ref={baudRef}>
            <span className="text-sm text-muted-foreground font-medium">波特率</span>
            <button
              onClick={() => setBaudMenuOpen(!baudMenuOpen)}
              className="pl-2 pr-6 py-2 rounded-xl bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-w-[100px] flex items-center justify-between cursor-pointer"
            >
              <span className="text-foreground">{baudRate}</span>
              <ChevronDown className={`w-4 h-4 text-muted-foreground transition-transform duration-200 ${baudMenuOpen ? 'rotate-180' : ''}`} />
            </button>

            <AnimatePresence>
              {baudMenuOpen && (
                <motion.div
                  initial={{ opacity: 0, y: -10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -10 }}
                  transition={{ duration: 0.2, ease: [0.2, 0.8, 0.2, 1] }}
                  className="absolute top-full mt-1 right-0 min-w-[100px] rounded-xl overflow-hidden"
                  style={{ zIndex: 99999, ...MENU_STYLE }}
                >
                  <div className="py-1">
                    {DEFAULT_BAUD_RATES.map(rate => (
                      <button
                        key={rate}
                        onClick={() => handleBaudSelect(rate)}
                        className={`w-full px-4 py-2 text-sm text-left hover:bg-accent/30 transition-colors cursor-pointer ${baudRate === rate ? 'text-primary font-medium' : ''}`}
                      >
                        {rate}
                      </button>
                    ))}
                    <div className="px-2 py-1">
                      <input
                        type="number"
                        value={customBaud}
                        onChange={(e) => setCustomBaud(e.target.value)}
                        placeholder="自定义..."
                        className="w-full px-3 py-1.5 rounded-lg bg-input-background border border-border text-sm focus:outline-none focus:ring-1 focus:ring-primary/50"
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            const rate = parseInt(customBaud);
                            if (rate > 0) handleBaudSelect(rate);
                          }
                        }}
                      />
                    </div>
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </div>

        {/* Fixed sections */}
        <div className="space-y-2.5 shrink-0">
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
            <FolderOpen className="w-4 h-4 text-muted-foreground" />
            <span className="text-muted-foreground">本地解码</span>
          </motion.button>

          <motion.button
            whileHover={{ scale: 1.01 }}
            onClick={handleSelectOutputDir}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors hover:bg-black/10 dark:hover:bg-white/10 cursor-pointer group">
            <FolderOpen className="w-4 h-4 shrink-0 text-muted-foreground" />
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
        </div>

        {/* Log area — grows to fill remaining space */}
        <div className="flex-1 min-h-[40px] mt-2.5 overflow-y-auto rounded-lg p-2 text-[10px] font-mono leading-relaxed bg-black/5 dark:bg-white/5">
          {logs.length === 0 ? (
            <div className="text-muted-foreground/30">日志输出...</div>
          ) : (
            logs.map((l, i) => <div key={i} className="whitespace-pre-wrap break-all">{l}</div>)
          )}
          <div ref={logEndRef} />
        </div>

        {/* Bottom buttons — always visible at the bottom */}
        <div className="flex gap-2 mt-2.5 shrink-0">
          <button
            onClick={handleCaptureToggle}
            disabled={!selectedPort}
            className={`flex-1 flex items-center justify-center gap-2 py-2 rounded-lg text-sm font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer ${
              capturing ? 'bg-red-500 hover:bg-red-600 text-white' : 'bg-green-500 hover:bg-green-600 text-white'
            }`}>
            {capturing ? '■ 停止捕获' : '▶ 开始捕获'}
          </button>

          <button onClick={handleDecode}
            disabled={capturedData.length === 0 || decoding || !outputDir}
            className="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg bg-primary hover:bg-primary/90 text-white text-sm font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer">
            {decoding ? '解码中...' : '▲ 解码导出'}
          </button>
        </div>
      </div>
    </div>
  );
}
