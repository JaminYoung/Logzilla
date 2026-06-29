import { Sliders, FolderOpen, Zap } from 'lucide-react';

interface DecodePanelProps {
  format: string;
  onFormatChange: (v: string) => void;
  channels: string;
  onChannelsChange: (v: string) => void;
  frameMs: string;
  onFrameMsChange: (v: string) => void;
  sampleRate: string;
  onSampleRateChange: (v: string) => void;
  customSampleRate: string;
  onCustomSampleRateChange: (v: string) => void;
  showSampleRateCustom: boolean;
  bitrate: string;
  onBitrateChange: (v: string) => void;
  customBitrate: string;
  onCustomBitrateChange: (v: string) => void;
  showBitrateCustom: boolean;
  outputDir: string;
  onSelectOutputDir: () => void;
  saveRaw: boolean;
  onSaveRawChange: (v: boolean) => void;
  saveWav: boolean;
  onSaveWavChange: (v: boolean) => void;
  decoding: boolean;
  onDecode: () => void;
  hasData: boolean;
}

function SelectRow({ label, value, options, onChange }: {
  label: string;
  value: string;
  options: string[];
  onChange: (v: string) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-xs text-muted-foreground w-20 shrink-0">{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="flex-1 px-2 py-1.5 rounded-lg bg-input-background border border-border text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary/50"
      >
        {options.map(o => <option key={o} value={o}>{o}</option>)}
      </select>
    </div>
  );
}

export function DecodePanel({
  format, onFormatChange,
  channels, onChannelsChange,
  frameMs, onFrameMsChange,
  sampleRate, onSampleRateChange,
  customSampleRate, onCustomSampleRateChange,
  showSampleRateCustom,
  bitrate, onBitrateChange,
  customBitrate, onCustomBitrateChange,
  showBitrateCustom,
  outputDir, onSelectOutputDir,
  saveRaw, onSaveRawChange,
  saveWav, onSaveWavChange,
  decoding, onDecode, hasData,
}: DecodePanelProps) {
  return (
    <div
      className="flex flex-col rounded-xl border border-[#E2E8F0] overflow-hidden"
      style={{ background: 'var(--acrylic-tint)', backdropFilter: 'blur(20px)' }}
    >
      <div className="px-4 py-3 border-b border-[#E2E8F0]/50 flex items-center gap-2">
        <Sliders className="w-4 h-4 text-muted-foreground" />
        <span className="text-sm font-semibold text-foreground">LC3 参数</span>
      </div>

      <div className="p-4 flex flex-col gap-3">
        <SelectRow label="Format" value={format} options={['LC3', 'LC3 Plus', 'LC3 Plus HR']} onChange={onFormatChange} />
        <SelectRow label="Channels" value={channels} options={['1', '2']} onChange={onChannelsChange} />
        <SelectRow label="Frame" value={frameMs} options={['2.5ms', '5ms', '7.5ms', '10ms']} onChange={onFrameMsChange} />

        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground w-20 shrink-0">Sample</span>
          <div className="flex-1 flex gap-1">
            {showSampleRateCustom ? (
              <input
                type="text"
                value={customSampleRate}
                onChange={(e) => onCustomSampleRateChange(e.target.value)}
                placeholder="如 48KHz"
                className="flex-1 px-2 py-1.5 rounded-lg bg-input-background border border-border text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary/50"
              />
            ) : (
              <select
                value={sampleRate}
                onChange={(e) => onSampleRateChange(e.target.value)}
                className="flex-1 px-2 py-1.5 rounded-lg bg-input-background border border-border text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary/50"
              >
                {['16KHz', '24KHz', '32KHz', '48KHz', '其他'].map(o => <option key={o} value={o}>{o}</option>)}
              </select>
            )}
          </div>
        </div>

        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground w-20 shrink-0">Bitrate</span>
          <div className="flex-1">
            {showBitrateCustom ? (
              <input
                type="text"
                value={customBitrate}
                onChange={(e) => onCustomBitrateChange(e.target.value)}
                placeholder="如 80Kbps"
                className="w-full px-2 py-1.5 rounded-lg bg-input-background border border-border text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary/50"
              />
            ) : (
              <select
                value={bitrate}
                onChange={(e) => onBitrateChange(e.target.value)}
                className="w-full px-2 py-1.5 rounded-lg bg-input-background border border-border text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary/50"
              >
                {['64Kbps', '80Kbps', '96Kbps', '其他'].map(o => <option key={o} value={o}>{o}</option>)}
              </select>
            )}
          </div>
        </div>

        <div className="h-px bg-[#E2E8F0]/50" />

        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground">Output</span>
          <div className="flex items-center gap-2 flex-1 justify-end">
            <span className="text-xs text-muted-foreground/60 truncate max-w-[180px]">
              {outputDir || '未选择'}
            </span>
            <button
              onClick={onSelectOutputDir}
              className="p-1.5 rounded-lg hover:bg-accent/30 transition-colors shrink-0"
              title="选择输出目录"
            >
              <FolderOpen className="w-4 h-4 text-muted-foreground" />
            </button>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <label className="flex items-center gap-1.5 cursor-pointer">
            <input
              type="checkbox"
              checked={saveRaw}
              onChange={(e) => onSaveRawChange(e.target.checked)}
              className="w-3.5 h-3.5 rounded accent-primary"
            />
            <span className="text-xs text-muted-foreground">RAW (.bin)</span>
          </label>
          <label className="flex items-center gap-1.5 cursor-pointer">
            <input
              type="checkbox"
              checked={saveWav}
              onChange={(e) => onSaveWavChange(e.target.checked)}
              className="w-3.5 h-3.5 rounded accent-primary"
            />
            <span className="text-xs text-muted-foreground">WAV (.wav)</span>
          </label>
        </div>

        <button
          onClick={onDecode}
          disabled={!hasData || decoding || !outputDir}
          className="flex items-center justify-center gap-2 py-2.5 rounded-lg bg-primary hover:bg-primary/90 text-white text-sm font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <Zap className="w-4 h-4" />
          {decoding ? '解码中...' : '解码导出'}
        </button>
      </div>
    </div>
  );
}
