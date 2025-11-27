import React from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import { Dialog, Transition } from '@headlessui/react';
import { Fragment } from 'react';
import {
  XMarkIcon,
  ChartBarIcon,
  QueueListIcon,
  Cog6ToothIcon,
  ChartPieIcon,
  PowerIcon,
} from '@heroicons/react/24/outline';
import { cn } from '../../utils/cn';

interface SidebarProps {
  open?: boolean;
  onClose?: () => void;
}

interface NavigationItem {
  name: string;
  href: string;
  icon: React.ComponentType<{ className?: string }>;
  current?: boolean;
}

const navigation: NavigationItem[] = [
  {
    name: '仪表板',
    href: '/',
    icon: ChartPieIcon,
  },
  {
    name: 'Cookie 管理',
    href: '/cookies',
    icon: QueueListIcon,
  },
  {
    name: '统计图表',
    href: '/stats',
    icon: ChartBarIcon,
  },
  {
    name: '系统配置',
    href: '/config',
    icon: Cog6ToothIcon,
  },
];

export const Sidebar: React.FC<SidebarProps> = ({ open, onClose }) => {
  const location = useLocation();

  const NavContent = () => (
    <nav className="flex flex-1 flex-col">
      <ul role="list" className="flex flex-1 flex-col gap-y-7">
        <li>
          <ul role="list" className="-mx-2 space-y-1">
            {navigation.map((item) => {
              const isActive = location.pathname === item.href ||
                             (item.href !== '/' && location.pathname.startsWith(item.href));

              return (
                <li key={item.name}>
                  <NavLink
                    to={item.href}
                    onClick={onClose}
                    className={cn(
                      isActive
                        ? 'bg-primary-50 text-primary-600 dark:bg-primary-900/20 dark:text-primary-400'
                        : 'text-gray-700 hover:text-primary-600 hover:bg-gray-50 dark:text-gray-300 dark:hover:text-primary-400 dark:hover:bg-gray-800',
                      'group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold transition-colors'
                    )}
                  >
                    <item.icon
                      className={cn(
                        isActive ? 'text-primary-600 dark:text-primary-400' : 'text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-400',
                        'h-6 w-6 shrink-0 transition-colors'
                      )}
                      aria-hidden="true"
                    />
                    {item.name}
                  </NavLink>
                </li>
              );
            })}
          </ul>
        </li>

        <li className="mt-auto">
          <button
            onClick={() => {
              // Admin action - emergency stop
              if (window.confirm('确定要执行紧急停止吗？这将暂停所有工作线程。')) {
                // Implement emergency stop
                console.log('Emergency stop triggered');
              }
            }}
            className="group -mx-2 flex gap-x-3 rounded-md p-2 text-sm font-semibold leading-6 text-gray-700 hover:bg-red-50 hover:text-red-600 dark:text-gray-300 dark:hover:bg-red-900/20 dark:hover:text-red-400 transition-colors w-full"
          >
            <PowerIcon
              className="h-6 w-6 shrink-0 text-gray-400 group-hover:text-red-600 dark:group-hover:text-red-400"
              aria-hidden="true"
            />
            紧急停止
          </button>
        </li>
      </ul>
    </nav>
  );

  // Mobile sidebar (as modal)
  if (open !== undefined) {
    return (
      <Transition.Root show={open} as={Fragment}>
        <Dialog as="div" className="relative z-50 lg:hidden" onClose={onClose}>
          <Transition.Child
            as={Fragment}
            enter="transition-opacity ease-linear duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="transition-opacity ease-linear duration-300"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div className="fixed inset-0 bg-gray-900/80" />
          </Transition.Child>

          <div className="fixed inset-0 flex">
            <Transition.Child
              as={Fragment}
              enter="transition ease-in-out duration-300 transform"
              enterFrom="-translate-x-full"
              enterTo="translate-x-0"
              leave="transition ease-in-out duration-300 transform"
              leaveFrom="translate-x-0"
              leaveTo="-translate-x-full"
            >
              <Dialog.Panel className="relative mr-16 flex w-full max-w-xs flex-1">
                <Transition.Child
                  as={Fragment}
                  enter="ease-in-out duration-300"
                  enterFrom="opacity-0"
                  enterTo="opacity-100"
                  leave="ease-in-out duration-300"
                  leaveFrom="opacity-100"
                  leaveTo="opacity-0"
                >
                  <div className="absolute left-full top-0 flex w-16 justify-center pt-5">
                    <button
                      type="button"
                      className="-m-2.5 p-2.5"
                      onClick={onClose}
                    >
                      <span className="sr-only">Close sidebar</span>
                      <XMarkIcon
                        className="h-6 w-6 text-white"
                        aria-hidden="true"
                      />
                    </button>
                  </div>
                </Transition.Child>

                {/* Sidebar content */}
                <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-white dark:bg-gray-800 px-6 pb-4">
                  <div className="flex h-16 shrink-0 items-center">
                    <h1 className="text-xl font-bold text-gray-900 dark:text-white">
                      封号控制台
                    </h1>
                  </div>
                  <NavContent />
                </div>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </Dialog>
      </Transition.Root>
    );
  }

  // Desktop sidebar (static)
  return <NavContent />;
};