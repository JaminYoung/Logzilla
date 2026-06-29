import { ButtonHTMLAttributes, ReactNode } from 'react';
import { motion } from 'motion/react';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost';
  icon?: ReactNode;
  children: ReactNode;
}

export function Button({
  variant = 'primary',
  icon,
  children,
  className = '',
  disabled,
  ...props
}: ButtonProps) {
  const baseStyles = "px-4 py-2 rounded-lg font-medium transition-all duration-200 flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed";

  const variants = {
    primary: "bg-primary text-primary-foreground hover:brightness-110 shadow-sm",
    secondary: "bg-secondary text-secondary-foreground hover:bg-muted border border-border",
    ghost: "bg-transparent text-foreground hover:bg-muted/50"
  };

  return (
    <motion.button
      whileHover={!disabled ? { scale: 1.02 } : undefined}
      whileTap={!disabled ? { scale: 0.98 } : undefined}
      transition={{ duration: 0.16, ease: [0.2, 0.8, 0.2, 1] }}
      className={`${baseStyles} ${variants[variant]} ${className}`}
      disabled={disabled}
      onClick={props.onClick}
    >
      {icon}
      {children}
    </motion.button>
  );
}
