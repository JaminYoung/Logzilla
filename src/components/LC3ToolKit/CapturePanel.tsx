import { Radio, RefreshCw, Upload, Play, Square } from 'lucide-react';

interface CapturePanelProps {
  connectedPorts: string[];
  selectedPort: string;
  onSelectPort: (port: string) => void;
  capturing: boolean;
  onStart: () => void;
  onStop: () => void;
  onImport: () => void;
  onRefresh: () => void;
  byteCount: number;
  capturedCount: number;
}

export function CapturePanel({
  connectedPorts,
  selectedPort,
  onSelectPort,
  capturing,
  onStart,
  onStop,
  onImport,
  onRefresh,
  byteCount,
  capturedCount,
}: CapturePanelProps) {
  return (
    <div
      className="flex flex-col rounded-xl border border-[#E2E8F0] overflow-hidden"
      style={{ background: 'var(--acrylic-tint)', backdropFilter: 'blur(20px)' }}
    >
      <div className="px-4 py-3 border-b border-[#E2E8F0]/50 flex items-center gap-2">
        <Radio className="w-4 h-4 text-muted-foreground" />
        <span className="text-sm font-semibold text-foreground">串口捕获</span>
      </div>

      <div className="p-4 flex flex-col gap-3">
        <div>
          <label className="text-xs text-muted-foreground mb-1 block">已连接串口</label>
          <div className="flex items-center gap-2">
            <select
              value={selectedPort}
              onChange={(e) => onSelectPort(e.target.value)}
              disabled={capturing}
              className="flex-1 px-3 py-2 rounded-lg bg-input-background border border-border text-sm text-foreground focus:outline-none focus:ring-1 focus:ring-primary/50 disabled:opacity-50"
            >
              <option value="">-- 选择串口 --</option>
              {connectedPorts.map(p => (
                <option key={p} value={p}>{p}</option>
              ))}
            </select>
            <button
              onClick={onRefresh}
              disabled={capturing}
              className="p-2 rounded-lg hover:bg-accent/30 transition-colors disabled:opacity-50"
              title="刷新串口列表"
            >
              <RefreshCw className="w-4 h-4 text-muted-foreground" />
            </button>
          </div>
        </div>

        <div className="flex gap-2">
          {!capturing ? (
            <button
              onClick={onStart}
              disabled={!selectedPort}
              className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-lg bg-green-500 hover:bg-green-600 text-white text-sm font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <Play className="w-4 h-4 fill-current" />
              开始捕获
            </button>
          ) : (
            <button
              onClick={onStop}
              className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-lg bg-red-500 hover:bg-red-600 text-white text-sm font-semibold transition-colors"
            >
              <Square className="w-4 h-4 fill-current" />
              停止捕获
            </button>
          )}
        </div>

        <div className="h-px bg-[#E2E8F0]/50" />

        <button
          onClick={onImport}
          disabled={capturing}
          className="flex items-center justify-center gap-2 py-2.5 rounded-lg border border-[#E2E8F0] hover:bg-accent/20 text-sm text-muted-foreground hover:text-foreground transition-colors disabled:opacity-40"
        >
          <Upload className="w-4 h-4" />
          导入文件
        </button>
      </div>

      <div className="px-4 py-3 border-t border-[#E2E8F0]/50 flex flex-col gap-1">
        {capturing ? (
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
            <span className="text-xs text-muted-foreground">
              捕获中: {byteCount.toLocaleString()} 字节
            </span>
          </div>
        ) : capturedCount > 0 ? (
          <div className="text-xs text-muted-foreground">
            已缓存: {capturedCount.toLocaleString()} 字节
          </div>
        ) : (
          <div className="text-xs text-muted-foreground/60">
            等待捕获或导入
          </div>
        )}
      </div>
    </div>
  );
}
