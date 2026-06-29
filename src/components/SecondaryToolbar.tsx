import { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Menu, FolderOpen, Usb, ChevronDown, Circle, Play, Square, Columns2, Square as SquareIcon, Sun, Moon, RefreshCw, Bluetooth, Cable } from 'lucide-react';

interface SecondaryToolbarProps {
  onMenuClick: () => void;
  selectedPort: string;
  baudRate: number;
  firmwarePath: string;
  dualMode: boolean;
  isFlashing: boolean;
  onPortChange: (port: string) => void;
  onBaudRateChange: (rate: number) => void;
  onFirmwareSelect: () => void;
  onDualModeToggle: () => void;
  onThemeToggle: () => void;
  onStartFlash: () => void;
  onStopFlash: () => void;
  onRefreshPorts: () => void;
  isDark: boolean;
  availablePorts: string[];
  connectedPorts: string[];
  portTypes: Record<string, string>;
  portDescriptions: Record<string, string>;
}

export function SecondaryToolbar({
  onMenuClick,
  baudRate,
  firmwarePath,
  dualMode,
  isFlashing,
  onPortChange,
  onBaudRateChange,
  onFirmwareSelect,
  onDualModeToggle,
  onThemeToggle,
  onStartFlash,
  onStopFlash,
  onRefreshPorts,
  isDark,
  availablePorts,
  connectedPorts,
  portTypes,
  portDescriptions,
}: SecondaryToolbarProps) {
  const [portDropdownOpen, setPortDropdownOpen] = useState(false);
  const [baudDropdownOpen, setBaudDropdownOpen] = useState(false);
  const [customBaud, setCustomBaud] = useState('');

  const portDropdownRef = useRef<HTMLDivElement>(null);
  const baudDropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (portDropdownRef.current && !portDropdownRef.current.contains(event.target as Node)) {
        setPortDropdownOpen(false);
      }
      if (baudDropdownRef.current && !baudDropdownRef.current.contains(event.target as Node)) {
        setBaudDropdownOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handlePortToggle = (port: string) => {
    // Toggle connection - if connected, disconnect; if disconnected, connect
    const isConnected = connectedPorts.includes(port);
    if (isConnected) {
      onPortChange(port); // This will disconnect
    } else {
      onPortChange(port); // This will connect
    }
  };

  const handleBaudSelect = (rate: number) => {
    onBaudRateChange(rate);
    setBaudDropdownOpen(false);
    setCustomBaud('');
  };

  const getPortIcon = (port: string) => {
    const type = portTypes[port] || 'unknown';
    const desc = (portDescriptions[port] || '').toLowerCase();
    if (type === 'bluetooth' || desc.includes('bluetooth') || desc.includes('bth')) {
      return <Bluetooth className="w-3.5 h-3.5 text-blue-500 shrink-0" />;
    }
    if (type === 'usb') {
      return <Usb className="w-3.5 h-3.5 text-muted-foreground shrink-0" />;
    }
    return <Cable className="w-3.5 h-3.5 text-muted-foreground shrink-0" />;
  };

  const defaultBaudRates = [9600, 115200, 1500000, 6000000];
  const hasConnectedPorts = connectedPorts.length > 0;

  return (
    <div
      className="acrylic-bar px-4 py-2 flex items-center gap-3 border-b shrink-0 relative z-[300]"
      style={{
        background: 'var(--acrylic-tint)',
        backdropFilter: 'blur(30px) saturate(140%)',
        WebkitBackdropFilter: 'blur(30px) saturate(140%)',
        borderColor: 'var(--acrylic-border)'
      }}
    >
      {/* Menu Icon */}
      <motion.button
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.95 }}
        onClick={onMenuClick}
        className="p-2 rounded-lg hover:bg-accent/30 transition-colors"
      >
        <Menu className="w-5 h-5" />
      </motion.button>

      <div className="w-px h-6 bg-border" />

      {/* Serial Port Dropdown */}
      <div className="relative flex items-center" ref={portDropdownRef}>
        <button
          onClick={() => setPortDropdownOpen(!portDropdownOpen)}
          className="pl-3 pr-10 py-2 rounded-xl bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-w-[160px] flex items-center justify-between"
        >
          <div className="flex items-center gap-2">
            <Usb className="w-4 h-4 text-primary" />
            <span className="text-foreground">
              选择串口
            </span>
          </div>
          <div className="absolute right-2 flex items-center gap-1">
            <motion.button
              whileHover={{ scale: 1.1, rotate: 180 }}
              whileTap={{ scale: 0.9 }}
              onClick={(e) => {
                e.stopPropagation();
                onRefreshPorts();
              }}
              className="p-0.5 hover:bg-accent/30 rounded transition-colors"
            >
              <RefreshCw className="w-3.5 h-3.5 text-muted-foreground" />
            </motion.button>
            <ChevronDown className={`w-3.5 h-3.5 text-muted-foreground transition-transform duration-200 ${portDropdownOpen ? 'rotate-180' : ''}`} />
          </div>
        </button>

        <AnimatePresence>
          {portDropdownOpen && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.2, ease: [0.2, 0.8, 0.2, 1] }}
              className="absolute top-full mt-1 left-0 w-[200px] rounded-xl overflow-hidden"
              style={{
                zIndex: 99999,
                background: 'var(--acrylic-tint)',
                backdropFilter: 'blur(40px) saturate(150%)',
                WebkitBackdropFilter: 'blur(40px) saturate(150%)',
                border: '1px solid var(--acrylic-border)',
                boxShadow: 'var(--shadow-md)'
              }}
            >
              <div className="py-1">
                {availablePorts.map(port => {
                  const isConnected = connectedPorts.includes(port);
                  return (
                    <div
                      key={port}
                      className="px-3 py-2 hover:bg-accent/30 transition-colors flex items-center justify-between"
                    >
                      <div className="flex items-center gap-2">
                        <Circle className={`w-2.5 h-2.5 ${isConnected ? 'fill-green-500 text-green-500' : 'fill-muted-foreground text-muted-foreground'}`} />
                        <span className="text-sm">{port}</span>
                        {getPortIcon(port)}
                      </div>
                      <div
                        onClick={(e) => { e.stopPropagation(); handlePortToggle(port); }}
                        className={`w-9 h-5 rounded-full cursor-pointer transition-colors relative shrink-0 ${isConnected ? 'bg-primary' : 'bg-muted'}`}
                      >
                        <div className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-all ${isConnected ? 'left-[18px]' : 'left-0.5'}`} />
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

      <div className="w-px h-6 bg-border" />

      {/* Baud Rate Dropdown */}
      <div className="relative flex items-center gap-2" ref={baudDropdownRef}>
        <span className="text-sm text-muted-foreground font-medium">波特率</span>
        <button
          onClick={() => setBaudDropdownOpen(!baudDropdownOpen)}
          className="pl-2 pr-6 py-1.5 rounded-xl bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all min-w-[100px] flex items-center justify-between"
        >
          <span className="text-foreground">{baudRate}</span>
          <ChevronDown className={`w-3 h-3 text-muted-foreground transition-transform duration-200 ${baudDropdownOpen ? 'rotate-180' : ''}`} />
        </button>

        <AnimatePresence>
          {baudDropdownOpen && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.2, ease: [0.2, 0.8, 0.2, 1] }}
              className="absolute top-full mt-1 left-0 min-w-[120px] rounded-xl overflow-hidden"
              style={{
                zIndex: 99999,
                background: 'var(--acrylic-tint)',
                backdropFilter: 'blur(40px) saturate(150%)',
                WebkitBackdropFilter: 'blur(40px) saturate(150%)',
                border: '1px solid var(--acrylic-border)',
                boxShadow: 'var(--shadow-md)'
              }}
            >
              <div className="py-1">
                {defaultBaudRates.map(rate => (
                  <button
                    key={rate}
                    onClick={() => handleBaudSelect(rate)}
                    className={`w-full px-4 py-2 text-sm text-left hover:bg-accent/30 transition-colors ${baudRate === rate ? 'text-primary font-medium' : ''}`}
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
                    className="w-full px-3 py-1.5 rounded-lg bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
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

      <div className="w-px h-6 bg-border" />

      {/* Firmware Path Display */}
      <div className="flex items-center gap-2 flex-1 min-w-0">
        <input
          type="text"
          value={firmwarePath}
          readOnly
          placeholder="未选择固件..."
          className="flex-1 min-w-0 px-3 py-1.5 rounded-xl bg-input-background border border-border text-sm truncate"
        />
        <motion.button
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          onClick={onFirmwareSelect}
          className="px-3 py-1.5 rounded-xl bg-secondary text-secondary-foreground text-sm font-medium flex items-center gap-2 hover:bg-muted transition-all shrink-0"
        >
          <FolderOpen className="w-4 h-4" />
          浏览
        </motion.button>
      </div>

      <div className="w-px h-6 bg-border" />

      {/* Flash Button */}
      <motion.button
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        onClick={isFlashing ? onStopFlash : onStartFlash}
        disabled={!hasConnectedPorts && !isFlashing}
        className={`px-4 py-1.5 rounded-xl text-sm font-medium flex items-center gap-2 transition-all ${
          isFlashing
            ? 'bg-destructive text-destructive-foreground hover:brightness-110'
            : 'bg-primary text-primary-foreground hover:brightness-110 disabled:opacity-50 disabled:cursor-not-allowed'
        }`}
      >
        {isFlashing ? (
          <>
            <Square className="w-4 h-4" />
            停止
          </>
        ) : (
          <>
            <Play className="w-4 h-4" />
            烧录
          </>
        )}
      </motion.button>

      <div className="w-px h-6 bg-border" />

      {/* Dual Mode Toggle - Icon only */}
      <motion.button
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.95 }}
        onClick={onDualModeToggle}
        className="p-2 rounded-xl bg-secondary text-secondary-foreground hover:bg-muted transition-all"
        title={dualMode ? '单窗口' : '双窗口'}
      >
        {dualMode ? <SquareIcon className="w-4 h-4" /> : <Columns2 className="w-4 h-4" />}
      </motion.button>

      <div className="w-px h-6 bg-border" />

      {/* Theme Toggle */}
      <motion.button
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.95 }}
        onClick={onThemeToggle}
        className="p-2 rounded-xl bg-secondary text-secondary-foreground hover:bg-muted transition-all"
      >
        {isDark ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
      </motion.button>
    </div>
  );
}
