import { motion, AnimatePresence } from 'motion/react';
import { X, Plus, Trash2, Pencil, Download, Upload } from 'lucide-react';
import { useEffect, useState } from 'react';
import { Button } from './Button';

export interface HighlightRule {
  id: string;
  keyword: string;
  matchType: 'plain' | 'regex';
  textColor: string;
  bgColor: string;
}

const PRESET_STYLES = [
  { textColor: '#5D4037', bgColor: '#FFE082', label: '黄' },
  { textColor: '#1B5E20', bgColor: '#A5D6A7', label: '绿' },
  { textColor: '#880E4F', bgColor: '#F48FB1', label: '粉' },
  { textColor: '#0D47A1', bgColor: '#90CAF9', label: '蓝' },
  { textColor: '#E65100', bgColor: '#FFCC80', label: '橙' },
  { textColor: '#4A148C', bgColor: '#CE93D8', label: '紫' },
];

interface HighlightSettingsDialogProps {
  isOpen: boolean;
  rules: HighlightRule[];
  initialKeyword?: string;
  initialKeywordRequestId?: number;
  onAddRule: (rule: Omit<HighlightRule, 'id'>) => void;
  onUpdateRule: (id: string, rule: Omit<HighlightRule, 'id'>) => void;
  onDeleteRule: (id: string) => void;
  onClose: () => void;
}

export function HighlightSettingsDialog({ isOpen, rules, initialKeyword, initialKeywordRequestId, onAddRule, onUpdateRule, onDeleteRule, onClose }: HighlightSettingsDialogProps) {
  const [mode, setMode] = useState<'idle' | 'adding' | 'editing'>('idle');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [keyword, setKeyword] = useState('');
  const [matchType, setMatchType] = useState<'plain' | 'regex'>('plain');
  const [selectedStyle, setSelectedStyle] = useState<number | null>(0);
  const [customTextColor, setCustomTextColor] = useState(PRESET_STYLES[0].textColor);
  const [customBgColor, setCustomBgColor] = useState(PRESET_STYLES[0].bgColor);

  useEffect(() => {
    if (!isOpen || initialKeywordRequestId === undefined) return;
    const trimmed = initialKeyword?.trim();
    if (!trimmed) {
      setKeyword('');
      setMatchType('plain');
      setSelectedStyle(0);
      setCustomTextColor(PRESET_STYLES[0].textColor);
      setCustomBgColor(PRESET_STYLES[0].bgColor);
      setEditingId(null);
      setMode('idle');
      return;
    }
    setKeyword(trimmed);
    setMatchType('plain');
    setSelectedStyle(0);
    setCustomTextColor(PRESET_STYLES[0].textColor);
    setCustomBgColor(PRESET_STYLES[0].bgColor);
    setEditingId(null);
    setMode('adding');
  }, [isOpen, initialKeyword, initialKeywordRequestId]);

  const resetForm = () => {
    setKeyword('');
    setMatchType('plain');
    setSelectedStyle(0);
    setCustomTextColor(PRESET_STYLES[0].textColor);
    setCustomBgColor(PRESET_STYLES[0].bgColor);
    setEditingId(null);
    setMode('idle');
  };

  const handleConfirmAdd = () => {
    const trimmed = keyword.trim();
    if (!trimmed) return;
    if (editingId) {
      onUpdateRule(editingId, { keyword: trimmed, matchType, textColor: customTextColor, bgColor: customBgColor });
    } else {
      onAddRule({ keyword: trimmed, matchType, textColor: customTextColor, bgColor: customBgColor });
    }
    resetForm();
  };

  const handleCancelAdd = () => {
    resetForm();
  };

  const handleEdit = (rule: HighlightRule) => {
    setKeyword(rule.keyword);
    setMatchType(rule.matchType);
    setCustomTextColor(rule.textColor);
    setCustomBgColor(rule.bgColor);
    const presetIdx = PRESET_STYLES.findIndex(s => s.textColor === rule.textColor && s.bgColor === rule.bgColor);
    setSelectedStyle(presetIdx >= 0 ? presetIdx : null);
    setEditingId(rule.id);
    setMode('editing');
  };

  const handleExport = async () => {
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({
        defaultPath: 'highlight-rules.json',
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });
      if (!path) return;
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('write_file', { path, content: JSON.stringify(rules, null, 2) });
    } catch {}
  };

  const handleImport = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      });
      if (!selected) return;
      const { invoke } = await import('@tauri-apps/api/core');
      const text = await invoke<string>('read_file', { path: selected as string });
      const imported: any[] = JSON.parse(text);
      if (!Array.isArray(imported)) throw new Error('格式错误');
      for (const r of imported) {
        if (r.keyword && (r.matchType === 'plain' || r.matchType === 'regex')) {
          onAddRule({
            keyword: r.keyword,
            matchType: r.matchType,
            textColor: r.textColor || '#000000',
            bgColor: r.bgColor || 'transparent',
          });
        }
      }
    } catch {}
  };

  const handlePresetClick = (i: number) => {
    setSelectedStyle(i);
    setCustomTextColor(PRESET_STYLES[i].textColor);
    setCustomBgColor(PRESET_STYLES[i].bgColor);
  };

  const handleTextColorChange = (color: string) => {
    setCustomTextColor(color);
    setSelectedStyle(null);
  };

  const handleBgColorChange = (color: string) => {
    setCustomBgColor(color);
    setSelectedStyle(null);
  };

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
              className="pointer-events-auto w-[480px] rounded-xl overflow-hidden"
              style={{
                background: 'var(--acrylic-tint)',
                backdropFilter: 'blur(60px) saturate(180%)',
                WebkitBackdropFilter: 'blur(60px) saturate(180%)',
                border: '1px solid var(--acrylic-border)',
                boxShadow: 'var(--shadow-lg)'
              }}
            >
              <div className="px-5 py-4 border-b border-border/50 flex items-center justify-between">
                <h3 className="font-semibold">高亮规则设置 ({rules.length})</h3>
                <div className="flex items-center gap-1">
                  <motion.button
                    whileHover={{ scale: 1.1 }}
                    whileTap={{ scale: 0.9 }}
                    onClick={handleImport}
                    title="导入"
                    className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
                  >
                    <Download className="w-4 h-4" />
                  </motion.button>
                  <motion.button
                    whileHover={{ scale: 1.1 }}
                    whileTap={{ scale: 0.9 }}
                    onClick={handleExport}
                    title="导出"
                    className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
                  >
                    <Upload className="w-4 h-4" />
                  </motion.button>
                  <motion.button
                    whileHover={{ scale: 1.1 }}
                    whileTap={{ scale: 0.9 }}
                    onClick={onClose}
                    className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
                  >
                    <X className="w-4 h-4" />
                  </motion.button>
                </div>
              </div>

              <div className="p-5">
                {mode !== 'idle' ? (
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium mb-1.5">关键词</label>
                      <input
                        autoFocus
                        value={keyword}
                        onChange={e => setKeyword(e.target.value)}
                        placeholder="输入匹配关键词"
                        onKeyDown={e => { if (e.key === 'Enter') handleConfirmAdd(); if (e.key === 'Escape') handleCancelAdd(); }}
                        className="w-full px-3 py-2 rounded-xl bg-input-background border border-border text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium mb-1.5">匹配方式</label>
                      <div className="flex gap-4">
                        <label className="flex items-center gap-2 cursor-pointer">
                          <input
                            type="radio"
                            name="hlMatchType"
                            checked={matchType === 'plain'}
                            onChange={() => setMatchType('plain')}
                            className="w-4 h-4 accent-primary"
                          />
                          <span className="text-sm">纯文本</span>
                        </label>
                        <label className="flex items-center gap-2 cursor-pointer">
                          <input
                            type="radio"
                            name="hlMatchType"
                            checked={matchType === 'regex'}
                            onChange={() => setMatchType('regex')}
                            className="w-4 h-4 accent-primary"
                          />
                          <span className="text-sm">正则</span>
                        </label>
                      </div>
                    </div>
                    <div>
                      <label className="block text-sm font-medium mb-1.5">预设配色</label>
                      <div className="flex gap-2">
                        {PRESET_STYLES.map((s, i) => (
                          <div
                            key={i}
                            onClick={() => handlePresetClick(i)}
                            className={`w-9 h-9 rounded-lg cursor-pointer transition-all hover:scale-110 flex items-center justify-center text-xs font-bold ${
                              selectedStyle === i ? 'ring-2 ring-offset-2 ring-foreground' : ''
                            }`}
                            style={{ backgroundColor: s.bgColor, color: s.textColor }}
                            title={s.label}
                          >
                            Aa
                          </div>
                        ))}
                      </div>
                    </div>
                    <div className="flex gap-4">
                      <div className="flex-1">
                        <label className="block text-sm font-medium mb-1.5">文字颜色</label>
                        <div className="flex items-center gap-2">
                          <input
                            type="color"
                            value={customTextColor}
                            onChange={e => handleTextColorChange(e.target.value)}
                            className="w-9 h-9 rounded-lg cursor-pointer border border-border bg-transparent p-0.5"
                          />
                          <span className="text-xs font-mono text-muted-foreground">{customTextColor}</span>
                        </div>
                      </div>
                      <div className="flex-1">
                        <label className="block text-sm font-medium mb-1.5">背景颜色</label>
                        <div className="flex items-center gap-2">
                          <input
                            type="color"
                            value={customBgColor}
                            onChange={e => handleBgColorChange(e.target.value)}
                            className="w-9 h-9 rounded-lg cursor-pointer border border-border bg-transparent p-0.5"
                          />
                          <span className="text-xs font-mono text-muted-foreground">{customBgColor}</span>
                        </div>
                      </div>
                    </div>
                    <div className="flex justify-end gap-2 pt-2">
                      <Button variant="ghost" onClick={handleCancelAdd}>取消</Button>
                      <Button variant="primary" onClick={handleConfirmAdd} disabled={!keyword.trim()}>
                        {editingId ? '保存' : '确认'}
                      </Button>
                    </div>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {rules.length === 0 ? (
                      <div className="text-sm text-muted-foreground text-center py-8">
                        还没有高亮规则，点击 "+ 增加" 添加
                      </div>
                    ) : (
                      <div className="space-y-1 max-h-[240px] overflow-y-auto">
                        {rules.map(rule => (
                          <div
                            key={rule.id}
                            className="flex items-center justify-between px-3 py-2 rounded-xl hover:bg-accent/20 transition-colors"
                          >
                            <div className="flex items-center gap-2 min-w-0">
                              <div
                                className="w-6 h-5 rounded text-[10px] font-bold flex items-center justify-center shrink-0"
                                style={{ backgroundColor: rule.bgColor, color: rule.textColor }}
                              >
                                Aa
                              </div>
                              <span className="text-sm truncate">{rule.keyword}</span>
                              <span className={`text-xs px-1.5 py-0.5 rounded shrink-0 ${
                                rule.matchType === 'regex' ? 'bg-purple-500/10 text-purple-500' : 'bg-blue-500/10 text-blue-500'
                              }`}>
                                {rule.matchType === 'regex' ? '正则' : '纯文本'}
                              </span>
                            </div>
                            <div className="flex items-center gap-0.5">
                              <motion.button
                                whileHover={{ scale: 1.1 }}
                                whileTap={{ scale: 0.9 }}
                                onClick={() => handleEdit(rule)}
                                className="p-1 rounded-lg hover:bg-accent/30 text-muted-foreground hover:text-foreground transition-colors shrink-0"
                              >
                                <Pencil className="w-3.5 h-3.5" />
                              </motion.button>
                              <motion.button
                                whileHover={{ scale: 1.1 }}
                                whileTap={{ scale: 0.9 }}
                                onClick={() => onDeleteRule(rule.id)}
                                className="p-1 rounded-lg hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors shrink-0"
                              >
                                <Trash2 className="w-3.5 h-3.5" />
                              </motion.button>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                    <Button
                      variant="secondary"
                      icon={<Plus className="w-4 h-4" />}
                      onClick={() => setMode('adding')}
                      className="w-full"
                    >
                      增加
                    </Button>
                  </div>
                )}
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
