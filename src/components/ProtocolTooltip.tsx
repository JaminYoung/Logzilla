import { useEffect, useRef } from 'react';
import { motion } from 'motion/react';
import { X, HelpCircle } from 'lucide-react';

export interface ProtocolField {
  name: string;
  value: string;
}

export interface ParsedProtocol {
  protocol: string;
  name: string;
  opcode_info: string;
  recognized: boolean;
  fields: ProtocolField[];
}

interface ProtocolTooltipProps {
  data: ParsedProtocol;
  x: number;
  y: number;
  onClose: () => void;
}

export function ProtocolTooltip({ data, x, y, onClose }: ProtocolTooltipProps) {
  const isDark = document.documentElement.classList.contains('dark');
  const tooltipRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (tooltipRef.current && !tooltipRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const timer = setTimeout(() => {
      document.addEventListener('mousedown', handleClickOutside);
    }, 50);
    return () => {
      clearTimeout(timer);
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [onClose]);

  const tooltipWidth = 360;
  const left = Math.min(x, window.innerWidth - tooltipWidth - 20);
  const top = Math.min(y + 10, window.innerHeight - 300);

  const protocolColors: Record<string, string> = {
    'HCI CMD': 'bg-blue-500/15 text-blue-500',
    'HCI EVT': 'bg-green-500/15 text-green-500',
    'HCI ACL': 'bg-gray-500/15 text-gray-500',
    'LMP': 'bg-orange-500/15 text-orange-500',
    'LLCP': 'bg-purple-500/15 text-purple-500',
    '???': 'bg-muted/50 text-muted-foreground',
  };

  const borderColor = isDark ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(255, 255, 255, 0.4)';

  return (
    <motion.div
      ref={tooltipRef}
      initial={{ opacity: 0, y: -5 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -5 }}
      transition={{ duration: 0.15 }}
      className="fixed z-[9999] rounded-xl overflow-hidden select-text"
      style={{
        left,
        top,
        width: tooltipWidth,
        background: isDark ? 'rgba(44, 44, 44, 0.85)' : 'rgba(252, 252, 252, 0.85)',
        backdropFilter: 'blur(60px) saturate(180%)',
        WebkitBackdropFilter: 'blur(60px) saturate(180%)',
        border: borderColor,
        boxShadow: isDark ? '0 8px 32px rgba(0, 0, 0, 0.5)' : '0 8px 32px rgba(0, 0, 0, 0.16)',
      }}
    >
      <div className="flex items-center justify-between px-3 py-2 border-b border-border/15">
        <span className={`px-2 py-0.5 rounded text-xs font-medium ${protocolColors[data.protocol] || protocolColors['???']}`}>
          {data.protocol}
        </span>
        <button
          onClick={onClose}
          className="p-1 rounded hover:bg-accent/30 transition-colors"
        >
          <X className="w-3.5 h-3.5 text-muted-foreground" />
        </button>
      </div>

      {data.recognized ? (
        <>
          <div className="px-3 py-2 border-b border-border/10">
            <div className="font-medium text-sm">{data.name}</div>
            {data.opcode_info && (
              <div className="text-xs text-muted-foreground font-mono mt-0.5">
                {data.opcode_info}
              </div>
            )}
          </div>

          {data.fields.length > 0 && (
            <div className="px-3 py-2 max-h-[220px] overflow-y-auto">
              <table className="w-full text-xs">
                <tbody>
                  {data.fields.map((f, i) => (
                    <tr key={i} className="border-b border-border/5 last:border-0">
                      <td className="py-1 pr-3 font-medium text-foreground/70 whitespace-nowrap align-top">
                        {f.name}
                      </td>
                      <td className="py-1 font-mono text-muted-foreground break-all">
                        {f.value}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </>
      ) : (
        <div className="px-3 py-6 text-center">
          <HelpCircle className="w-8 h-8 text-muted-foreground/20 mx-auto mb-2" />
          <div className="text-sm text-muted-foreground">
            I don't know either, still learning...
          </div>
        </div>
      )}
    </motion.div>
  );
}
