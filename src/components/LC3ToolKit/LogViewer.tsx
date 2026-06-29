import { useEffect, useRef } from 'react';

interface LogViewerProps {
  logs: string[];
  decoding: boolean;
}

export function LogViewer({ logs, decoding }: LogViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div
      className="mx-4 mb-4 rounded-xl border border-[#E2E8F0] overflow-hidden flex flex-col"
      style={{ background: 'var(--acrylic-tint)', backdropFilter: 'blur(20px)', height: '160px' }}
    >
      {decoding && (
        <div className="h-1 bg-primary/30">
          <div className="h-full bg-primary animate-progress-indeterminate" style={{ width: '40%' }} />
        </div>
      )}
      <div
        ref={containerRef}
        className="flex-1 overflow-y-auto p-3 font-mono text-xs leading-relaxed"
        style={{ color: 'var(--log-text, #64748B)' }}
      >
        {logs.length === 0 ? (
          <div className="text-muted-foreground/40">等待操作...</div>
        ) : (
          logs.map((line, i) => (
            <div key={i} className="whitespace-pre-wrap break-all">{line}</div>
          ))
        )}
      </div>
    </div>
  );
}
