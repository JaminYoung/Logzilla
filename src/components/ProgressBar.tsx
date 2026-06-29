import { motion } from 'motion/react';

interface ProgressBarProps {
  progress: number;
  status?: string;
}

export function ProgressBar({ progress, status }: ProgressBarProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.26, ease: [0.2, 0.8, 0.2, 1] }}
      className="px-4 py-3 rounded-xl"
      style={{
        background: 'var(--acrylic-tint)',
        backdropFilter: 'blur(40px) saturate(150%)',
        WebkitBackdropFilter: 'blur(40px) saturate(150%)',
        border: '1px solid var(--acrylic-border)',
        boxShadow: 'var(--shadow-sm)'
      }}
    >
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium">{status || '烧录进度'}</span>
        <span className="text-sm font-semibold text-primary">{progress}%</span>
      </div>
      <div className="h-2 bg-muted/50 rounded-full overflow-hidden">
        <motion.div
          initial={{ width: 0 }}
          animate={{ width: `${progress}%` }}
          transition={{ duration: 0.4, ease: [0.2, 0.8, 0.2, 1] }}
          className="h-full bg-gradient-to-r from-primary to-[#5B4FD6] rounded-full"
        />
      </div>
    </motion.div>
  );
}
