import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { CapturePanel } from './CapturePanel';
import { DecodePanel } from './DecodePanel';
import { LogViewer } from './LogViewer';

export interface Lc3DecodeResult {
  success: boolean;
  frame_count: number;
  frame_samples: number;
  duration_secs: number;
  leftover_bytes: number;
  saved_files: string[];
  error: string | null;
}

export interface Lc3CaptureStatus {
  port: string;
  byte_count: number;
  active: boolean;
}

export default function LC3ToolKitApp() {
  const [connectedPorts, setConnectedPorts] = useState<string[]>([]);
  const [selectedPort, setSelectedPort] = useState('');
  const [capturing, setCapturing] = useState(false);
  const [capturedData, setCapturedData] = useState<number[]>([]);
  const [byteCount, setByteCount] = useState(0);
  const [capturedCount, setCapturedCount] = useState(0);

  const [format, setFormat] = useState('LC3 Plus');
  const [channels, setChannels] = useState('1');
  const [frameMs, setFrameMs] = useState('10ms');
  const [sampleRate, setSampleRate] = useState('48KHz');
  const [customSampleRate, setCustomSampleRate] = useState('');
  const [bitrate, setBitrate] = useState('80Kbps');
  const [customBitrate, setCustomBitrate] = useState('');
  const [outputDir, setOutputDir] = useState('');
  const [saveRaw, setSaveRaw] = useState(true);
  const [saveWav, setSaveWav] = useState(true);
  const [decoding, setDecoding] = useState(false);

  const [logs, setLogs] = useState<string[]>([]);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const addLog = useCallback((msg: string) => {
    const t = new Date().toLocaleTimeString('zh-CN', { hour12: false });
    setLogs(prev => [...prev, `[${t}] ${msg}`]);
  }, []);

  const refreshConnectedPorts = useCallback(async () => {
    try {
      const ports = await invoke<string[]>('lc3_get_connected_ports');
      setConnectedPorts(ports);
    } catch {}
  }, []);

  useEffect(() => {
    refreshConnectedPorts();
  }, [refreshConnectedPorts]);

  const handleStartCapture = async () => {
    if (!selectedPort) {
      addLog('请先选择串口');
      return;
    }
    try {
      await invoke('lc3_start_capture', { port: selectedPort });
      setCapturing(true);
      setByteCount(0);
      addLog(`开始捕获: ${selectedPort}`);

      pollRef.current = setInterval(async () => {
        try {
          const status = await invoke<Lc3CaptureStatus>('lc3_get_capture_status', { port: selectedPort });
          setByteCount(status.byte_count);
        } catch {}
      }, 200);
    } catch (e) {
      addLog(`开始捕获失败: ${e}`);
    }
  };

  const handleStopCapture = async () => {
    if (pollRef.current) {
      clearInterval(pollRef.current);
      pollRef.current = null;
    }
    try {
      const data = await invoke<number[]>('lc3_stop_capture', { port: selectedPort });
      setCapturedData(data);
      const n = data.length;
      setCapturedCount(n);
      addLog(`停止捕获, 共收到 ${n.toLocaleString()} 字节`);

      try {
        const status = await invoke<Lc3CaptureStatus>('lc3_get_capture_status', { port: selectedPort });
        setByteCount(status.byte_count);
      } catch {}
    } catch (e) {
      addLog(`停止捕获失败: ${e}`);
    }
    setCapturing(false);
  };

  useEffect(() => {
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, []);

  const handleImportFile = async () => {
    try {
      const path = await openDialog({
        multiple: false,
        filters: [{ name: 'LC3 数据文件', extensions: ['bin', 'txt'] }],
      });
      if (!path) return;

      const data = await invoke<number[]>('lc3_import_file', { path: path as string });
      setCapturedData(data);
      setCapturedCount(data.length);
      const fileName = (path as string).split(/[/\\]/).pop();
      addLog(`已导入: ${fileName} (${data.length.toLocaleString()} 字节)`);
    } catch (e) {
      addLog(`导入文件失败: ${e}`);
    }
  };

  const handleSelectOutputDir = async () => {
    try {
      const dir = await openDialog({
        directory: true,
        multiple: false,
        defaultPath: outputDir || undefined,
      });
      if (dir) setOutputDir(dir as string);
    } catch {}
  };

  const getSampleRateVal = (): number => {
    const v = sampleRate === '其他' ? customSampleRate : sampleRate;
    return parseInt(v.replace(/KHz|kHz/i, '')) * 1000;
  };

  const getBitrateVal = (): number => {
    const v = bitrate === '其他' ? customBitrate : bitrate;
    return parseInt(v.replace(/Kbps|kbps/i, '')) * 1000;
  };

  const getFrameMsVal = (): number => {
    return parseFloat(frameMs.replace('ms', ''));
  };

  const getChannelsVal = (): number => {
    return parseInt(channels);
  };

  const getHrmode = (): boolean => {
    return format === 'LC3 Plus HR';
  };

  const handleDecode = async () => {
    if (capturedData.length === 0) {
      addLog('没有可解码的数据');
      return;
    }
    if (!outputDir) {
      addLog('请选择输出目录');
      return;
    }

    const sr = getSampleRateVal();
    const br = getBitrateVal();
    const fms = getFrameMsVal();
    const nch = getChannelsVal();
    const hrmode = getHrmode();

    if (isNaN(sr) || sr <= 0) { addLog('无效的采样率'); return; }
    if (isNaN(br) || br <= 0) { addLog('无效的比特率'); return; }
    if (isNaN(fms) || fms <= 0) { addLog('无效的帧时长'); return; }

    setDecoding(true);
    const now = new Date();
    const ts = `${now.getFullYear()}${String(now.getMonth()+1).padStart(2,'0')}${String(now.getDate()).padStart(2,'0')}_${String(now.getHours()).padStart(2,'0')}${String(now.getMinutes()).padStart(2,'0')}${String(now.getSeconds()).padStart(2,'0')}`;
    const baseName = `LC3_${ts}`;

    addLog(`开始解码: ${fms}ms, ${sr/1000}KHz, ${br/1000}Kbps, ${nch}ch${hrmode?' HR':''}`);

    try {
      const result = await invoke<Lc3DecodeResult>('lc3_decode_and_export', {
        data: capturedData,
        sampleRate: sr,
        numChannels: nch,
        bitrate: br,
        frameDurationMs: fms,
        hrmode,
        outputDir,
        baseName,
        saveRaw,
        saveWav,
      });

      if (result.success) {
        addLog(`解码完成: ${result.frame_count} 帧, ${result.duration_secs.toFixed(2)}s`);
        if (result.leftover_bytes > 0) {
          addLog(`尾部剩余 ${result.leftover_bytes} 字节 (不足一帧, 已丢弃)`);
        }
        if (result.saved_files.length > 0) {
          addLog(`已保存: ${result.saved_files.join(', ')} → ${outputDir}`);
        }
      } else {
        addLog(`解码失败: ${result.error || '未知错误'}`);
      }
    } catch (e) {
      addLog(`解码失败: ${e}`);
    }
    setDecoding(false);
  };

  const [showSampleRateCustom, setShowSampleRateCustom] = useState(false);
  const [showBitrateCustom, setShowBitrateCustom] = useState(false);

  return (
    <div className="w-screen h-screen flex flex-col bg-[#F0F5FF] text-foreground overflow-hidden select-none">
      <div className="flex-1 min-h-0 flex gap-4 p-4">
        <div className="w-[320px] shrink-0 flex flex-col gap-3">
          <CapturePanel
            connectedPorts={connectedPorts}
            selectedPort={selectedPort}
            onSelectPort={setSelectedPort}
            capturing={capturing}
            onStart={handleStartCapture}
            onStop={handleStopCapture}
            onImport={handleImportFile}
            onRefresh={refreshConnectedPorts}
            byteCount={byteCount}
            capturedCount={capturedCount}
          />
        </div>

        <div className="w-[380px] shrink-0 flex flex-col gap-3">
          <DecodePanel
            format={format}
            onFormatChange={setFormat}
            channels={channels}
            onChannelsChange={setChannels}
            frameMs={frameMs}
            onFrameMsChange={setFrameMs}
            sampleRate={sampleRate}
            onSampleRateChange={(v) => { setSampleRate(v); setShowSampleRateCustom(v === '其他'); }}
            customSampleRate={customSampleRate}
            onCustomSampleRateChange={setCustomSampleRate}
            showSampleRateCustom={showSampleRateCustom}
            bitrate={bitrate}
            onBitrateChange={(v) => { setBitrate(v); setShowBitrateCustom(v === '其他'); }}
            customBitrate={customBitrate}
            onCustomBitrateChange={setCustomBitrate}
            showBitrateCustom={showBitrateCustom}
            outputDir={outputDir}
            onSelectOutputDir={handleSelectOutputDir}
            saveRaw={saveRaw}
            onSaveRawChange={setSaveRaw}
            saveWav={saveWav}
            onSaveWavChange={setSaveWav}
            decoding={decoding}
            onDecode={handleDecode}
            hasData={capturedData.length > 0}
          />
        </div>
      </div>

      <LogViewer logs={logs} decoding={decoding} />
    </div>
  );
}
