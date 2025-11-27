import React, { useState } from 'react';
import { Outlet } from 'react-router-dom';
import { Header } from './Header';
import { Sidebar } from './Sidebar';
import { useAppContext } from '../../context/AppContext';

interface MainLayoutProps {
  children?: React.ReactNode;
  version?: string;
}

const MainLayout: React.FC<MainLayoutProps> = ({ children, version }) => {
  const { isAuthenticated } = useAppContext();
  const [sidebarOpen, setSidebarOpen] = useState(false);

  if (!isAuthenticated) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4">
        <div className="w-full max-w-md">
          {children}
          {version && (
            <div className="mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
              版本 {version}
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Sidebar for mobile */}
      <Sidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />

      {/* Static sidebar for desktop */}
      <div className="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-72 lg:flex-col">
        <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-white dark:bg-gray-800 px-6 pb-4 shadow-sm border-r border-gray-200 dark:border-gray-700">
          <div className="flex h-16 shrink-0 items-center">
            <h1 className="text-xl font-bold text-gray-900 dark:text-white">
              封号控制台
            </h1>
          </div>
          <Sidebar />
        </div>
      </div>

      {/* Main content */}
      <div className="lg:pl-72">
        <Header onMenuClick={() => setSidebarOpen(true)} />

        <main className="py-6">
          <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
            {children || <Outlet />}
          </div>
        </main>

        {version && (
          <footer className="border-t border-gray-200 dark:border-gray-700 py-4">
            <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
              <p className="text-center text-sm text-gray-500 dark:text-gray-400">
                版本 {version}
              </p>
            </div>
          </footer>
        )}
      </div>
    </div>
  );
};

export default MainLayout;