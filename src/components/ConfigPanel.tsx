import { motion, AnimatePresence } from 'motion/react';
import { Button } from './Button';
import { ConfigToggleClose } from './ConfigToggleClose';
import { useState, useRef, useEffect, useMemo, useCallback } from 'react';

interface ConfigOption {
  label: string;
  value: number;
}

interface ConfigItem {
  entry_type: string;
  label_cn: string;
  name: string;
  tooltip: string;
  var_name: string;
  offset: number;
  bit_offset: number;
  bit_width: number;
  size: number;
  value: any;
  default_value: any;
  min_val: number;
  max_val: number;
  options: ConfigOption[];
  val_type: number;
  str_length: number;
  children: ConfigItem[];
  level_value: number;
  ui_condition_var: string | null;
}

interface ConfigPanelProps {
  isOpen: boolean;
  onClose: () => void;
  configTree: ConfigItem[];
  infoData: number[];
  onSave: (changes: [ConfigItem, any][]) => void;
}

export function ConfigPanel({ isOpen, onClose, configTree, infoData: _infoData, onSave }: ConfigPanelProps) {
  const [activeSection, setActiveSection] = useState('');
  const [changes, setChanges] = useState<Map<string, any>>(new Map());
  const contentRef = useRef<HTMLDivElement>(null);
  const bookmarkRef = useRef<HTMLDivElement>(null);
  const sectionRefs = useRef<{ [key: string]: HTMLElement }>({});
  const bookmarkRefs = useRef<{ [key: string]: HTMLElement }>({});

  // Calculate bookmark width based on longest label
  const bookmarkWidth = useMemo(() => {
    if (configTree.length === 0) return 0;
    const maxLabelLength = Math.max(...configTree.map(s => s.label_cn.length));
    return maxLabelLength * 12 + 20;
  }, [configTree]);

  // Total panel width: 320px content + bookmark width
  const panelWidth = 320 + bookmarkWidth;

  useEffect(() => {
    if (configTree.length > 0 && !activeSection) {
      setActiveSection(configTree[0].name);
    }
  }, [configTree]);

  // Auto-follow bookmark highlight on scroll
  useEffect(() => {
    const content = contentRef.current;
    if (!content) return;

    const handleScroll = () => {
      const scrollTop = content.scrollTop;
      const containerHeight = content.clientHeight;
      const threshold = containerHeight * 0.25;

      let closest = '';
      let minDist = Infinity;

      for (const section of configTree) {
        const el = sectionRefs.current[section.name];
        if (!el) continue;
        const dist = Math.abs(el.offsetTop - scrollTop - threshold);
        if (dist < minDist) {
          minDist = dist;
          closest = section.name;
        }
      }

      if (closest) setActiveSection(closest);
    };

    content.addEventListener('scroll', handleScroll, { passive: true });
    handleScroll();
    return () => content.removeEventListener('scroll', handleScroll);
  }, [isOpen, configTree]);

  const scrollToSection = (name: string) => {
    setActiveSection(name);
    const element = sectionRefs.current[name];
    if (element && contentRef.current) {
      const containerHeight = contentRef.current.clientHeight;
      // Position section 1/4 from the top
      const offsetTop = element.offsetTop - containerHeight / 4;
      contentRef.current.scrollTo({
        top: Math.max(0, offsetTop),
        behavior: 'smooth'
      });
    }
  };

  const handleValueChange = (item: ConfigItem, value: any) => {
    setChanges(prev => new Map(prev).set(item.var_name || item.name, value));
  };

  const handleSave = () => {
    const changesList: [ConfigItem, any][] = [];
    for (const [key, value] of changes) {
      const item = findItemByName(configTree, key);
      if (item) {
        changesList.push([item, value]);
      }
    }
    onSave(changesList);
    setChanges(new Map());
  };

  const findItemByName = (items: ConfigItem[], name: string): ConfigItem | null => {
    for (const item of items) {
      if (item.var_name === name || item.name === name) {
        return item;
      }
      if (item.children) {
        const found = findItemByName(item.children, name);
        if (found) return found;
      }
    }
    return null;
  };

  const getDisplayValue = (item: ConfigItem): any => {
    if (changes.has(item.var_name || item.name)) {
      return changes.get(item.var_name || item.name);
    }
    return item.value;
  };

  // Check if an item should be visible based on LVL and ui_condition
  const isItemVisible = useCallback((item: ConfigItem, sectionChildren: ConfigItem[], itemIndex: number): boolean => {
    // Find the most recent LVL before this item
    let lastLvlValue = 1; // Default to visible if no LVL found
    
    for (let i = itemIndex - 1; i >= 0; i--) {
      if (sectionChildren[i].entry_type === 'LVL') {
        lastLvlValue = sectionChildren[i].level_value;
        break;
      }
    }
    
    // Check LVL: if bit(0) is 0, hide the item
    if ((lastLvlValue & 1) === 0) {
      return false;
    }
    
    // Check ui_condition: if present and non-zero, it's a variable name
    if (item.ui_condition_var && item.ui_condition_var !== '') {
      // Find the variable in the config tree
      const conditionVar = findItemByName(configTree, item.ui_condition_var);
      if (conditionVar) {
        // Recursively check if the condition variable itself is visible
        for (const section of configTree) {
          const condIdx = section.children?.findIndex(c =>
            c.var_name === item.ui_condition_var || c.name === item.ui_condition_var
          ) ?? -1;
          if (condIdx >= 0) {
            if (!isItemVisible(conditionVar, section.children, condIdx)) {
              return false;
            }
            break;
          }
        }

        const varValue = getDisplayValue(conditionVar);
        let shouldShow = false;
        if (typeof varValue === 'boolean') {
          shouldShow = varValue;
        } else if (typeof varValue === 'number') {
          shouldShow = varValue !== 0;
        } else if (typeof varValue === 'object' && varValue !== null) {
          if (varValue.Bool !== undefined) {
            shouldShow = varValue.Bool;
          } else if (varValue.Int !== undefined) {
            shouldShow = varValue.Int !== 0;
          }
        }
        return shouldShow;
      }
    }
    
    return true;
  }, [changes, configTree]);

  const renderConfigItem = (item: ConfigItem, index: number, sectionChildren: ConfigItem[]) => {
    // Check visibility based on LVL and ui_condition
    if (!isItemVisible(item, sectionChildren, index)) {
      return null;
    }

    const displayValue = getDisplayValue(item);
    const key = item.var_name || `${item.name}-${index}`;

    switch (item.entry_type) {
      case 'CHK':
        return (
          <div key={key} className="flex items-center justify-between py-2">
            <label className="text-sm" title={item.tooltip}>{item.label_cn}</label>
            <input
              type="checkbox"
              checked={!!displayValue}
              onChange={(e) => handleValueChange(item, e.target.checked)}
              className="rounded"
            />
          </div>
        );

      case 'LST':
        return (
          <div key={key} className="py-2">
            <label className="text-sm text-muted-foreground block mb-1" title={item.tooltip}>{item.label_cn}</label>
            <select
              value={displayValue || ''}
              onChange={(e) => handleValueChange(item, e.target.value)}
              className="w-full px-3 py-2 rounded-lg bg-input-background border border-border text-sm"
            >
              {item.options.map((opt, i) => (
                <option key={i} value={opt.value}>{opt.label}</option>
              ))}
            </select>
          </div>
        );

      case 'LSV':
        return (
          <div key={key} className="py-2">
            <label className="text-sm text-muted-foreground block mb-1" title={item.tooltip}>{item.label_cn}</label>
            <select
              value={displayValue ?? ''}
              onChange={(e) => handleValueChange(item, Number(e.target.value))}
              className="w-full px-3 py-2 rounded-lg bg-input-background border border-border text-sm"
            >
              {item.options.map((opt, i) => (
                <option key={i} value={opt.value}>{opt.label}</option>
              ))}
            </select>
          </div>
        );

      case 'U08':
      case 'S08':
      case 'U16':
      case 'UBT':
        return (
          <div key={key} className="py-2">
            <label className="text-sm text-muted-foreground block mb-1" title={item.tooltip}>{item.label_cn}</label>
            <div className="flex items-center gap-2">
              <input
                type="range"
                min={item.min_val}
                max={item.max_val}
                value={displayValue ?? item.min_val}
                onChange={(e) => handleValueChange(item, Number(e.target.value))}
                className="flex-1"
              />
              <span className="text-sm w-12 text-right">{displayValue ?? item.min_val}</span>
            </div>
          </div>
        );

      case 'TXT':
        return (
          <div key={key} className="py-2">
            <label className="text-sm text-muted-foreground block mb-1" title={item.tooltip}>{item.label_cn}</label>
            <input
              type="text"
              value={displayValue || ''}
              onChange={(e) => handleValueChange(item, e.target.value)}
              maxLength={item.str_length || 32}
              className="w-full px-3 py-2 rounded-lg bg-input-background border border-border text-sm"
            />
          </div>
        );

      case 'MAC':
        return (
          <div key={key} className="py-2">
            <label className="text-sm text-muted-foreground block mb-1" title={item.tooltip}>{item.label_cn}</label>
            <input
              type="text"
              value={displayValue || ''}
              onChange={(e) => handleValueChange(item, e.target.value)}
              placeholder="AA:BB:CC:DD:EE:FF"
              className="w-full px-3 py-2 rounded-lg bg-input-background border border-border text-sm"
            />
          </div>
        );

      default:
        return null;
    }
  };

  const renderSection = (section: ConfigItem, sectionIndex: number) => {
    if (section.entry_type !== 'SUB') return null;

    const sectionColors = ['#0078d4', '#5B4FD6'];
    const color = sectionColors[sectionIndex % sectionColors.length];

    // Filter visible children
    const visibleChildren = section.children?.filter((item, idx) => 
      isItemVisible(item, section.children, idx)
    ) || [];

    // Skip section if no visible children
    if (visibleChildren.length === 0) return null;

    return (
      <section
        key={section.name}
        ref={(el) => el && (sectionRefs.current[section.name] = el)}
      >
        <h3 className="font-semibold mb-4 text-base" style={{ borderLeft: `3px solid ${color}`, paddingLeft: '12px' }}>
          {section.label_cn}
        </h3>
        <div className="space-y-1 pl-2">
          {section.children?.map((item, idx) => renderConfigItem(item, idx, section.children))}
        </div>
      </section>
    );
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Backdrop - covers toolbar but not titlebar */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed left-0 right-0 top-12 bottom-0 bg-black/20 backdrop-blur-sm z-[350]"
            onClick={onClose}
          />

          {/* Panel */}
          <motion.div
            initial={{ x: '100%' }}
            animate={{ x: 0 }}
            exit={{ x: '100%' }}
            transition={{ duration: 0.26, ease: [0.2, 0.8, 0.2, 1] }}
            className="fixed right-0 top-12 bottom-0 z-[360] flex"
            style={{
              width: `${panelWidth}px`,
              background: 'var(--acrylic-tint)',
              backdropFilter: 'blur(60px) saturate(180%)',
              WebkitBackdropFilter: 'blur(60px) saturate(180%)',
              borderLeft: '1px solid var(--acrylic-border)',
              boxShadow: 'var(--shadow-lg)'
            }}
          >
            <div className="absolute left-0 top-1/2 -translate-y-1/2 z-10">
              <ConfigToggleClose onClick={onClose} className="relative outline-none py-4 pr-6" />
            </div>
            <div className="flex-1 flex flex-col">
              <div className="px-6 py-4 border-b border-border/50">
                <h2 className="font-semibold text-lg">配置选项</h2>
              </div>

              <div className="flex-1 flex overflow-hidden">
                <div ref={contentRef} className="flex-1 overflow-y-auto px-6 py-4 space-y-6">
                  {configTree.length === 0 ? (
                    <div className="text-center text-muted-foreground py-12">
                      <p className="text-lg mb-2">未加载配置</p>
                      <p className="text-sm">请先打开DCF文件以加载配置选项</p>
                    </div>
                  ) : (
                    configTree.map((section, idx) => renderSection(section, idx))
                  )}
                </div>

                {configTree.length > 0 && (
                  <div ref={bookmarkRef} className="border-l border-border/30 py-4 px-3 overflow-y-auto" style={{ width: `${bookmarkWidth}px`, flexShrink: 0 }}>
                    <div className="space-y-1 relative">
                      {configTree
                        .filter(section => {
                          // Only show bookmarks for sections with visible children
                          if (section.entry_type !== 'SUB') return false;
                          return section.children?.some((item, idx) => 
                            isItemVisible(item, section.children, idx)
                          ) ?? false;
                        })
                        .map((section) => {
                          // Find original index for color
                          const originalIdx = configTree.indexOf(section);
                          const sectionColors = ['#0078d4', '#5B4FD6'];
                          const color = sectionColors[originalIdx % sectionColors.length];
                          
                          return (
                            <motion.button
                              key={section.name}
                              ref={(el) => el && (bookmarkRefs.current[section.name] = el)}
                              onClick={() => scrollToSection(section.name)}
                              className={`w-full text-left px-3 py-2 text-xs rounded-lg transition-colors relative whitespace-nowrap ${
                                activeSection === section.name
                                  ? 'font-semibold'
                                  : 'text-muted-foreground hover:text-foreground hover:bg-accent/20'
                              }`}
                              style={{
                                color: activeSection === section.name ? color : undefined
                              }}
                              whileHover={{ x: 4 }}
                              transition={{ duration: 0.15 }}
                            >
                              <motion.div
                                className="absolute inset-0 rounded-lg"
                                initial={false}
                                animate={{
                                  opacity: activeSection === section.name ? 1 : 0,
                                  x: activeSection === section.name ? 0 : -10
                                }}
                                style={{
                                  background: `linear-gradient(90deg, ${color}15, transparent)`,
                                  borderLeft: activeSection === section.name ? `3px solid ${color}` : '3px solid transparent'
                                }}
                                transition={{ duration: 0.15, ease: "easeOut" }}
                              />
                              <span className="relative z-10">{section.label_cn}</span>
                            </motion.button>
                          );
                        })
                        }
                    </div>
                  </div>
                )}
              </div>

              {configTree.length > 0 && (
                <div className="px-6 py-4 border-t border-border/50 flex gap-2">
                  <Button variant="primary" className="flex-1" onClick={handleSave}>
                    应用
                  </Button>
                  <Button variant="secondary" className="flex-1" onClick={onClose}>
                    取消
                  </Button>
                </div>
              )}
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
