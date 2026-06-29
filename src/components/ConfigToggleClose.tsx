import { motion } from 'motion/react';

interface ConfigToggleCloseProps {
  onClick: () => void;
  className?: string;
}

export function ConfigToggleClose({ onClick, className = "fixed right-0 top-1/2 -translate-y-1/2 z-30 outline-none pl-6 py-4" }: ConfigToggleCloseProps) {
  return (
    <motion.button
      variants={{
        hidden: { opacity: 0 },
        hover: { opacity: 1 },
      }}
      initial="hidden"
      whileHover="hover"
      transition={{ duration: 0.2 }}
      onClick={onClick}
      className={className}
    >
      <svg width="80" height="44" viewBox="-8 0 80 44">
        <g
          fill="none"
          stroke="#60cdff"
          strokeWidth="6"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <motion.g
            variants={{ hidden: { x: 0 }, hover: { x: -4 } }}
            transition={{ type: "spring", stiffness: 350, damping: 5 }}
          >
            <path d="M28 8 L44 22 L28 36" opacity="0.35" />
          </motion.g>
          <g>
            <path d="M10 8 L26 22 L10 36" opacity="0.35" />
          </g>
          <motion.g
            variants={{ hidden: { x: 0 }, hover: { x: 4 } }}
            transition={{ type: "spring", stiffness: 350, damping: 5 }}
          >
            <path d="M-8 8 L8 22 L-8 36" opacity="0.35" />
          </motion.g>
        </g>
      </svg>
    </motion.button>
  );
}
