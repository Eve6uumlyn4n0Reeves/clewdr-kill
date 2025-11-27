import React from 'react';
import { cn } from '../../utils/cn';
import {
  CheckCircleIcon,
  ExclamationTriangleIcon,
  XCircleIcon,
  InformationCircleIcon,
} from '@heroicons/react/24/outline';

export interface MessageProps {
  type: 'success' | 'error' | 'warning' | 'info';
  children: React.ReactNode;
  className?: string;
}

const Message: React.FC<MessageProps> = ({ type, children, className }) => {
  const icons = {
    success: CheckCircleIcon,
    error: XCircleIcon,
    warning: ExclamationTriangleIcon,
    info: InformationCircleIcon,
  };

  const colors = {
    success: 'text-success-600 bg-success-50 border-success-200 dark:bg-success-900/20 dark:border-success-800 dark:text-success-400',
    error: 'text-error-600 bg-error-50 border-error-200 dark:bg-error-900/20 dark:border-error-800 dark:text-error-400',
    warning: 'text-warning-600 bg-warning-50 border-warning-200 dark:bg-warning-900/20 dark:border-warning-800 dark:text-warning-400',
    info: 'text-primary-600 bg-primary-50 border-primary-200 dark:bg-primary-900/20 dark:border-primary-800 dark:text-primary-400',
  };

  const Icon = icons[type];

  return (
    <div
      className={cn(
        'flex items-center gap-2 px-4 py-3 rounded-lg border',
        colors[type],
        className
      )}
    >
      <Icon className="h-5 w-5 flex-shrink-0" />
      <div className="text-sm font-medium">{children}</div>
    </div>
  );
};

export default Message;