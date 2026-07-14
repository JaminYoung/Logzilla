import { motion, AnimatePresence } from 'motion/react';
import { X, FileText } from 'lucide-react';
import { useEffect } from 'react';
import { Button } from './Button';

export interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  /** 主说明文字 */
  message?: string;
  /** 可选：大号文件名 */
  fileName?: string;
  /** 可选：次要路径 / 细节 */
  detail?: string;
  /** 底部补充提示（如“将在同目录生成 .cfa”） */
  hint?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

/**
 * 应用内确认弹窗：毛玻璃半透明 + 淡蓝主色，风格对齐过滤设置等对话框。
 */
export function ConfirmDialog({
  isOpen,
  title,
  message,
  fileName,
  detail,
  hint,
  confirmLabel = '确认',
  cancelLabel = '取消',
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  useEffect(() => {
    if (!isOpen) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onCancel();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [isOpen, onCancel]);

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 bg-black/25 backdrop-blur-sm z-[500]"
            onClick={onCancel}
          />
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            transition={{ duration: 0.2, ease: [0.2, 0.8, 0.2, 1] }}
            className="fixed inset-0 z-[501] flex items-center justify-center pointer-events-none p-4"
          >
            <div
              className="pointer-events-auto w-full max-w-[440px] rounded-xl overflow-hidden"
              style={{
                background: 'var(--acrylic-tint)',
                backdropFilter: 'blur(60px) saturate(180%)',
                WebkitBackdropFilter: 'blur(60px) saturate(180%)',
                border: '1px solid var(--acrylic-border)',
                boxShadow: 'var(--shadow-lg)',
              }}
              role="dialog"
              aria-modal="true"
              aria-labelledby="confirm-dialog-title"
              onClick={e => e.stopPropagation()}
            >
              {/* Header */}
              <div className="px-5 py-4 border-b border-border/50 flex items-center justify-between">
                <h3 id="confirm-dialog-title" className="font-semibold text-foreground">
                  {title}
                </h3>
                <motion.button
                  whileHover={{ scale: 1.1 }}
                  whileTap={{ scale: 0.9 }}
                  onClick={onCancel}
                  className="p-1 rounded-lg hover:bg-accent/30 transition-colors"
                  aria-label="关闭"
                >
                  <X className="w-4 h-4 text-muted-foreground" />
                </motion.button>
              </div>

              {/* Body */}
              <div className="px-5 py-4 space-y-3">
                {message && (
                  <p className="text-sm text-foreground/90 leading-relaxed">{message}</p>
                )}

                {(fileName || detail) && (
                  <div
                    className="rounded-xl px-3.5 py-3 space-y-1.5"
                    style={{
                      background: 'var(--input-background)',
                      border: '1px solid var(--border)',
                    }}
                  >
                    {fileName && (
                      <div className="flex items-start gap-2 min-w-0">
                        <FileText className="w-4 h-4 text-primary shrink-0 mt-0.5" />
                        <span className="text-sm font-medium text-foreground break-all">
                          {fileName}
                        </span>
                      </div>
                    )}
                    {detail && (
                      <p
                        className="text-xs text-muted-foreground break-all leading-relaxed pl-6"
                        title={detail}
                      >
                        {detail}
                      </p>
                    )}
                  </div>
                )}

                {hint && (
                  <p className="text-xs text-muted-foreground leading-relaxed">{hint}</p>
                )}
              </div>

              {/* Footer：两按钮同宽、文字居中 */}
              <div className="px-5 py-3.5 border-t border-border/50 flex justify-end gap-2">
                <Button
                  variant="ghost"
                  onClick={onCancel}
                  className="w-[88px] justify-center gap-0"
                >
                  {cancelLabel}
                </Button>
                <Button
                  variant="primary"
                  onClick={onConfirm}
                  className="w-[88px] justify-center gap-0"
                >
                  {confirmLabel}
                </Button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
