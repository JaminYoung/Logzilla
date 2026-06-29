import { motion, AnimatePresence } from 'motion/react';
import { X } from 'lucide-react';

export interface Toast {
  id: string;
  message: string;
}

interface ToastContainerProps {
  toasts: Toast[];
  onDismiss: (id: string) => void;
}

export function ToastContainer({ toasts, onDismiss }: ToastContainerProps) {
  return (
    <div className="fixed bottom-20 right-6 z-[99999] flex flex-col-reverse gap-2 pointer-events-none">
      <AnimatePresence mode="popLayout">
        {toasts.map((toast) => (
          <motion.div
            key={toast.id}
            layout
            initial={{ opacity: 0, x: 80, scale: 0.95 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: 80, scale: 0.95 }}
            transition={{ duration: 0.25, ease: [0.2, 0.8, 0.2, 1] }}
            className="pointer-events-auto rounded-xl overflow-hidden max-w-[360px] min-w-[240px]"
            style={{
              background: 'var(--acrylic-tint)',
              backdropFilter: 'blur(40px) saturate(150%)',
              WebkitBackdropFilter: 'blur(40px) saturate(150%)',
              border: '1px solid var(--acrylic-border)',
              boxShadow: 'var(--shadow-md)'
            }}
          >
            <div className="px-4 py-3 flex items-start gap-3">
              <span className="text-sm flex-1 leading-relaxed">{toast.message}</span>
              <motion.button
                whileHover={{ scale: 1.1 }}
                whileTap={{ scale: 0.9 }}
                onClick={() => onDismiss(toast.id)}
                className="p-0.5 rounded-lg hover:bg-accent/30 transition-colors shrink-0 -mr-1 -mt-1"
              >
                <X className="w-3.5 h-3.5 text-muted-foreground" />
              </motion.button>
            </div>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
