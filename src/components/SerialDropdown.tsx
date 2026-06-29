import { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Usb, ChevronDown, Circle } from 'lucide-react';

interface PortStatus {
  port: string;
  isConnected: boolean;
}

interface SerialDropdownProps {
  ports: string[];
  portStatuses: PortStatus[];
  baudRate: number;
  onPortToggle: (port: string) => void;
  onBaudRateChange: (rate: number) => void;
}

export function SerialDropdown({
  ports,
  portStatuses,
  baudRate,
  onPortToggle,
  onBaudRateChange
}: SerialDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const connectedPorts = portStatuses.filter(p => p.isConnected);

  return (
    <div className="flex items-center gap-3">
      <Usb className="w-5 h-5 text-primary" />

      {/* Custom Dropdown */}
      <div className="relative" ref={dropdownRef}>
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="pl-3 pr-8 py-2 rounded-xl bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-w-[180px] flex items-center justify-between"
        >
          <span className="text-foreground">
            {connectedPorts.length === 0
              ? '选择串口...'
              : connectedPorts.length === 1
              ? connectedPorts[0].port
              : `${connectedPorts.length} 个串口已连接`}
          </span>
          <ChevronDown className={`w-4 h-4 text-muted-foreground transition-transform duration-200 ${isOpen ? 'rotate-180' : ''}`} />
        </button>

        <AnimatePresence>
          {isOpen && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.2, ease: [0.2, 0.8, 0.2, 1] }}
              className="absolute top-full mt-2 left-0 w-full rounded-xl overflow-hidden z-[9999]"
              style={{
                background: 'var(--acrylic-tint)',
                backdropFilter: 'blur(40px) saturate(150%)',
                WebkitBackdropFilter: 'blur(40px) saturate(150%)',
                border: '1px solid var(--acrylic-border)',
                boxShadow: 'var(--shadow-md)'
              }}
            >
              <div className="py-2">
                {ports.map(port => {
                  const status = portStatuses.find(p => p.port === port);
                  const isConnected = status?.isConnected || false;

                  return (
                    <div
                      key={port}
                      className="px-3 py-2 hover:bg-accent/30 transition-colors flex items-center justify-between group"
                    >
                      <div className="flex items-center gap-2">
                        <Circle
                          className={`w-2.5 h-2.5 ${
                            isConnected
                              ? 'fill-green-500 text-green-500'
                              : 'fill-muted-foreground text-muted-foreground'
                          }`}
                        />
                        <span className="text-sm">{port}</span>
                      </div>
                      <div
                        onClick={(e) => { e.stopPropagation(); onPortToggle(port); }}
                        className={`w-9 h-5 rounded-full cursor-pointer transition-colors relative shrink-0 ${isConnected ? 'bg-primary' : 'bg-muted'}`}
                      >
                        <div className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-all ${isConnected ? 'left-[18px]' : 'left-0.5'}`} />
                      </div>
                    </div>
                  );
                })}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Baud Rate Selector */}
      <select
        value={baudRate}
        onChange={(e) => onBaudRateChange(Number(e.target.value))}
        className="px-3 py-2 rounded-xl bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-w-[100px]"
      >
        <option value={9600}>9600</option>
        <option value={19200}>19200</option>
        <option value={38400}>38400</option>
        <option value={57600}>57600</option>
        <option value={115200}>115200</option>
        <option value={230400}>230400</option>
        <option value={460800}>460800</option>
        <option value={921600}>921600</option>
      </select>
    </div>
  );
}
