import { Circle } from 'lucide-react';

interface StatusBarProps {
  isConnected: boolean;
  ports: string[];
  device?: string;
}

export function StatusBar({ isConnected, ports, device }: StatusBarProps) {
  return (
    <div
      className="acrylic-bar flex items-center gap-4 px-4 py-1.5 text-xs border-t shrink-0"
      style={{
        background: 'var(--acrylic-tint)',
        backdropFilter: 'blur(30px) saturate(140%)',
        WebkitBackdropFilter: 'blur(30px) saturate(140%)',
        borderColor: 'var(--acrylic-border)'
      }}
    >
      <div className="flex items-center gap-1.5">
        <Circle
          className={`w-2 h-2 ${isConnected ? 'fill-green-500 text-green-500' : 'fill-muted-foreground text-muted-foreground'}`}
        />
        <span>{isConnected ? '已连接' : '未连接'}</span>
      </div>

      {ports.length > 0 && (
        <div className="flex items-center gap-1">
          <span className="text-muted-foreground">串口:</span>
          <span>{ports.join(", ")}</span>
        </div>
      )}

      {device && (
        <div className="flex items-center gap-1">
          <span className="text-muted-foreground">设备:</span>
          <span>{device}</span>
        </div>
      )}
    </div>
  );
}
