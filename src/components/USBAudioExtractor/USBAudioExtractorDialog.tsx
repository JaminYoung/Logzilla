import { motion, AnimatePresence } from 'motion/react';
import { X, FolderOpen, Zap, FileAudio, ChevronDown } from 'lucide-react';
import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

interface InterfaceInfo {
  interface: number;
  endpoint: number;
  direction: string;
  sample_rate: number;
  bit_depth: number;
  channels: number;
}

interface UsbAudioAnalysis {
  file_name: string;
  total_packets: number;
  iso_packets: number;
  interfaces: InterfaceInfo[];
  set_interface_count: number;
}

interface ExtractResultDto {
  segment_count: number;
  files: string[];
}

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onToast: (msg: string) => void;
}

const splitModes = [
  { value: 'A', label: '按接口独立分割', desc: '根据接口和方向独立分组，在长时间停顿或接口切换时自动分割音频' },
  { value: 'B', label: '全局同步分割',   desc: '所有接口按统一时间轴分割，任一接口切换时所有流同步分割' },
  { value: 'C', label: '仅格式变化分割', desc: '仅在音频采样率/位深/通道数变化时分割，忽略时间间隔' },
];

let persistedOutputDir = '';
let persistedFileDir = '';

export function USBAudioExtractorDialog({ isOpen, onClose, onToast }: Props) {
  const [filePath, setFilePath] = useState('');
  const [analysis, setAnalysis] = useState<UsbAudioAnalysis | null>(null);
  const [loading, setLoading] = useState(false);
  const [splitMode, setSplitMode] = useState('A');
  const [thresholdMs, setThresholdMs] = useState(200);
  const [outputDir, setOutputDir] = useState(persistedOutputDir);
  const [extracting, setExtracting] = useState(false);
  const [modeOpen, setModeOpen] = useState(false);
  const modeRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (modeRef.current && !modeRef.current.contains(e.target as Node)) {
        setModeOpen(false);
      }
    };
    if (modeOpen) document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [modeOpen]);

  const selectedModeDesc = splitModes.find(m => m.value === splitMode)?.desc || '';

  const handleSelectFile = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        defaultPath: persistedFileDir || undefined,
        filters: [{
          name: 'Wireshark/USBPcap 日志',
          extensions: ['pcapng', 'pcap']
        }]
      });
      if (!selected) return;
      const path = selected as string;
      persistedFileDir = path.substring(0, Math.max(0, path.lastIndexOf('\\')));
      setFilePath(path);
      setAnalysis(null);
      setLoading(true);
      try {
        const result = await invoke<UsbAudioAnalysis>('usb_audio_analyze', { path });
        setAnalysis(result);
      } catch (e) {
        onToast(`USB 音频分析失败: ${e}`);
      }
      setLoading(false);
    } catch {}
  };

  const handleBrowseOutput = async () => {
    try {
      const dir = await openDialog({
        directory: true,
        multiple: false,
        defaultPath: outputDir || undefined,
      });
      if (dir) {
        const d = dir as string;
        setOutputDir(d);
        persistedOutputDir = d;
      }
    } catch {}
  };

  const handleExtract = async () => {
    if (!filePath || !outputDir) return;
    setExtracting(true);
    try {
      const result = await invoke<ExtractResultDto>('usb_audio_extract', {
        path: filePath,
        splitMode,
        thresholdMs,
        outputDir,
      });
      setExtracting(false);
      onToast(`USB 音频提取完成: ${result.segment_count} 段, ${result.files.length} 个文件`);
    } catch (e) {
      onToast(`提取失败: ${e}`);
      setExtracting(false);
    }
  };

  const visibleInterfaces = analysis?.interfaces.filter(ifc => ifc.sample_rate > 0) || [];

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 bg-black/20 backdrop-blur-sm z-[400]"
            onClick={onClose}
          />
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            transition={{ duration: 0.2, ease: [0.2, 0.8, 0.2, 1] }}
            className="fixed inset-0 z-[401] flex items-center justify-center pointer-events-none"
          >
            <div
              className="pointer-events-auto w-[440px] rounded-xl overflow-hidden"
              style={{
                background: 'var(--acrylic-tint)',
                backdropFilter: 'blur(60px) saturate(180%)',
                WebkitBackdropFilter: 'blur(60px) saturate(180%)',
                border: '1px solid var(--acrylic-border)',
                boxShadow: 'var(--shadow-lg)'
              }}
            >
              <div className="px-5 py-4 border-b border-border/50 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <FileAudio className="w-4 h-4 text-muted-foreground" />
                  <h3 className="font-semibold text-sm">USB 音频提取</h3>
                </div>
                <motion.button
                  whileHover={{ scale: 1.1 }}
                  whileTap={{ scale: 0.9 }}
                  onClick={onClose}
                  className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
                >
                  <X className="w-4 h-4" />
                </motion.button>
              </div>

              <div className="p-5 space-y-4">
                <motion.button
                  whileHover={{ scale: 1.01 }}
                  whileTap={{ scale: 0.99 }}
                  onClick={handleSelectFile}
                  disabled={loading}
                  className="w-full flex items-center gap-2 px-0 py-1 rounded-lg text-sm transition-colors disabled:opacity-50"
                >
                  <motion.div whileHover={{ scale: 1.1 }}>
                    <FolderOpen className="w-4 h-4 text-muted-foreground shrink-0" />
                  </motion.div>
                  <span className="text-xs text-muted-foreground/60 truncate">
                    {filePath || '点击选择 USB 日志文件'}
                  </span>
                </motion.button>

                {loading && (
                  <div className="text-center text-sm text-muted-foreground py-4">
                    解析中...
                  </div>
                )}

                {analysis && !loading && (
                  <>
                    <div className="flex gap-4 text-xs text-muted-foreground">
                      <span>总包 {analysis.total_packets.toLocaleString()}</span>
                      <span>ISO包 {analysis.iso_packets.toLocaleString()}</span>
                      {analysis.set_interface_count > 0 && (
                        <span>SET_IF {analysis.set_interface_count}</span>
                      )}
                    </div>

                    {visibleInterfaces.length > 0 && (
                      <div className="space-y-1">
                        {visibleInterfaces.map((ifc, i) => (
                          <div key={i} className="text-xs text-muted-foreground grid grid-cols-[auto_auto_1fr] gap-2 items-center">
                            <span className="font-mono">EP 0x{ifc.endpoint.toString(16).padStart(2, '0').toUpperCase()} (INF {ifc.interface})</span>
                            <span className={`w-20 px-1.5 py-0.5 rounded text-[10px] font-medium text-center ${
                              ifc.direction === 'in' ? 'bg-blue-500/10 text-blue-500' : 'bg-green-500/10 text-green-500'
                            }`}>
                              {ifc.direction === 'in' ? '↑ IN (录音)' : '↓ OUT (播放)'}
                            </span>
                            <span>{ifc.channels}ch {ifc.sample_rate}Hz {ifc.bit_depth}bit</span>
                          </div>
                        ))}
                      </div>
                    )}

                    <div className="h-px bg-border/50" />

                    <div>
                      <label className="block text-xs font-medium text-muted-foreground mb-2">分段模式</label>
                      <div className="relative" ref={modeRef}>
                        <button
                          onClick={() => !extracting && setModeOpen(!modeOpen)}
                          disabled={extracting}
                          className="w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 disabled:opacity-50"
                          style={{
                            background: 'var(--acrylic-tint)',
                            backdropFilter: 'blur(40px) saturate(150%)',
                            WebkitBackdropFilter: 'blur(40px) saturate(150%)',
                            border: '1px solid var(--acrylic-border)',
                            color: 'var(--foreground)',
                          }}
                        >
                          <span>{splitModes.find(m => m.value === splitMode)?.value}. {splitModes.find(m => m.value === splitMode)?.label}</span>
                          <ChevronDown className={`w-3.5 h-3.5 transition-transform ${modeOpen ? 'rotate-180' : ''}`} />
                        </button>
                        {modeOpen && (
                          <div
                            className="absolute left-0 right-0 top-full mt-1 rounded-lg overflow-hidden z-50"
                            style={{
                              background: 'var(--acrylic-tint)',
                              backdropFilter: 'blur(40px) saturate(150%)',
                              WebkitBackdropFilter: 'blur(40px) saturate(150%)',
                              border: '1px solid var(--acrylic-border)',
                              boxShadow: 'var(--shadow-lg)',
                            }}
                          >
                            {splitModes.map(m => (
                              <div
                                key={m.value}
                                onClick={() => { setSplitMode(m.value); setModeOpen(false); }}
                                className={`px-3 py-2 text-sm cursor-pointer transition-colors ${
                                  splitMode === m.value ? 'bg-accent/30 text-foreground' : 'hover:bg-accent/20 text-muted-foreground'
                                }`}
                              >
                                {m.value}. {m.label}
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                      <div className="text-xs text-muted-foreground/60 mt-1.5 leading-relaxed">
                        {selectedModeDesc}
                      </div>
                    </div>

                    <div>
                      <label className="block text-xs font-medium text-muted-foreground mb-2">时间阈值</label>
                      <div className="flex items-center gap-2">
                        <input
                          type="number"
                          min={1}
                          value={thresholdMs}
                          onChange={e => setThresholdMs(Math.max(1, parseInt(e.target.value) || 200))}
                          disabled={extracting}
                          className="w-24 px-3 py-2 rounded-lg bg-input-background border border-border text-sm text-center focus:outline-none focus:ring-1 focus:ring-primary/50 disabled:opacity-50"
                        />
                        <span className="text-xs text-muted-foreground">ms</span>
                      </div>
                    </div>

                    <div>
                      <label className="block text-xs font-medium text-muted-foreground mb-2">输出目录</label>
                      <motion.button
                        whileHover={{ scale: 1.01 }}
                        whileTap={{ scale: 0.99 }}
                        onClick={handleBrowseOutput}
                        disabled={extracting}
                        className="w-full flex items-center gap-2 rounded-lg text-sm transition-colors disabled:opacity-50"
                      >
                        <motion.div whileHover={{ scale: 1.05 }}>
                          <FolderOpen className="w-4 h-4 text-muted-foreground shrink-0" />
                        </motion.div>
                        <span className="text-xs text-muted-foreground/60 truncate">
                          {outputDir || '点击选择输出目录'}
                        </span>
                      </motion.button>
                    </div>

                    {extracting && (
                      <div>
                        <div className="flex items-center justify-between mb-1.5">
                          <span className="text-xs text-muted-foreground">提取中...</span>
                        </div>
                        <div className="h-1.5 bg-muted/50 rounded-full overflow-hidden">
                          <motion.div
                            className="h-full bg-gradient-to-r from-primary to-[#5B4FD6] rounded-full"
                            animate={{ width: ['5%', '95%'] }}
                            transition={{ duration: 8, ease: 'easeInOut', repeat: Infinity }}
                          />
                        </div>
                      </div>
                    )}

                    <button
                      onClick={handleExtract}
                      disabled={extracting || !outputDir}
                      className="w-full flex items-center justify-center gap-2 py-2.5 rounded-lg bg-primary hover:bg-primary/90 text-white text-sm font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    >
                      <Zap className="w-4 h-4" />
                      {extracting ? '提取中...' : '开始提取'}
                    </button>
                  </>
                )}
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
