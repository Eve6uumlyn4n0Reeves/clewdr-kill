import React from "react";
// import { useTranslation } from "react-i18next";

export type TabType = "dashboard" | "cookies" | "stats" | "config";

interface TabNavigationProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
}

const TabNavigation: React.FC<TabNavigationProps> = ({ activeTab, onTabChange }) => {
  // const { t } = useTranslation();

  const tabs: Array<{
    id: TabType;
    label: string;
    icon: string;
    description?: string;
  }> = [
    {
      id: "dashboard",
      label: "📊 仪表板",
      icon: "📈",
      description: "系统概览和实时状态",
    },
    {
      id: "cookies",
      label: "🍪 Cookie管理",
      icon: "🎯",
      description: "Cookie队列和批量操作",
    },
    {
      id: "stats",
      label: "📈 统计分析",
      icon: "📊",
      description: "详细统计和历史数据",
    },
    {
      id: "config",
      label: "🛠 配置",
      icon: "⚙️",
      description: "封号参数与工作时间",
    },
  ];

  return (
    <div className="border-b border-gray-700">
      <nav className="flex space-x-1 px-2" aria-label="Tabs">
        {tabs.map((tab) => {
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`
                group relative min-w-0 flex-1 overflow-hidden bg-gray-800 py-4 px-1 text-center text-sm font-medium hover:bg-gray-700 focus:z-10 rounded-t-lg transition-all duration-200
                ${isActive
                  ? 'border-b-2 border-blue-500 text-blue-400 bg-gray-900'
                  : 'text-gray-400 hover:text-gray-300'
                }
              `}
              title={tab.description}
            >
              <div className="flex flex-col items-center space-y-1">
                <span className="text-lg">{tab.icon}</span>
                <span className="hidden sm:block whitespace-nowrap">{tab.label}</span>
                <span className="sm:hidden text-xs">{tab.label.split(' ')[1]}</span>
              </div>

              {/* Active indicator */}
              {isActive && (
                <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500"></div>
              )}

              {/* Hover tooltip for mobile */}
              <div className="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none whitespace-nowrap sm:hidden">
                {tab.description}
              </div>
            </button>
          );
        })}
      </nav>

      {/* Tab description (desktop only) */}
      <div className="hidden sm:block px-4 py-2 bg-gray-900 text-sm text-gray-400 border-t border-gray-800">
        {tabs.find(tab => tab.id === activeTab)?.description}
      </div>
    </div>
  );
};

export default TabNavigation;
