import React from 'react';
import { Bars3Icon } from '@heroicons/react/24/outline';
import Logout from '../auth/Logout';
import { useAppContext } from '../../context/AppContext';

interface HeaderProps {
  onMenuClick?: () => void;
}

export const Header: React.FC<HeaderProps> = ({ onMenuClick }) => {
  const { version } = useAppContext();

  return (
    <header className="sticky top-0 z-40 flex items-center justify-between gap-3 border-b border-gray-200 bg-white/80 px-4 py-3 backdrop-blur dark:border-gray-800 dark:bg-gray-900/80 lg:px-8">
      <div className="flex items-center gap-3">
        <button
          type="button"
          className="inline-flex items-center justify-center rounded-md p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-primary-500 dark:text-gray-300 dark:hover:bg-gray-800 lg:hidden"
          onClick={onMenuClick}
          aria-label="打开侧边栏"
        >
          <Bars3Icon className="h-6 w-6" />
        </button>
        <div className="text-sm text-gray-500 dark:text-gray-400">
          {version ? `版本 ${version}` : '正在加载版本信息'}
        </div>
      </div>
      <div className="flex items-center gap-3">
        <Logout variant="secondary" />
      </div>
    </header>
  );
};

export default Header;
