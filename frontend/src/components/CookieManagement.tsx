import React, { useState } from "react";
import {
  CpuChipIcon,
  CommandLineIcon,
  RocketLaunchIcon,
} from "@heroicons/react/24/outline";
import { CookieInput, VirtualizedCookieList } from "./cookies";
import { Card, CardHeader, CardTitle, CardContent } from "./ui/Card";
import { usePerfMonitor } from "../hooks";

const CookieManagement: React.FC = () => {
  const [refreshKey, setRefreshKey] = useState(0);
  usePerfMonitor("cookie-management");

  const handleCookieSubmitted = () => {
    setRefreshKey((prev) => prev + 1);
  };

  return (
    <div className="min-h-screen space-y-8 animate-fade-in">
      {/* 页面标题 */}
      <div className="relative">
        <div className="absolute inset-0 bg-gradient-to-r from-primary/10 via-transparent to-info/10 rounded-2xl blur-xl" />
        <Card variant="glass-strong" className="relative">
          <CardContent className="py-8">
            <div className="flex flex-col lg:flex-row items-center justify-between gap-6">
              <div className="text-center lg:text-left">
                <h1 className="text-4xl lg:text-5xl font-bold text-gradient mb-2">
                  Cookie 管理中心
                </h1>
                <p className="text-lg text-muted">
                  批量管理和监控 Claude Cookie 状态
                </p>
                <div className="flex items-center gap-4 mt-4 justify-center lg:justify-start">
                  <div className="flex items-center gap-2 text-sm text-muted">
                    <div className="pulse-dot-info" />
                    <span>实时监控</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-muted">
                    <div className="pulse-dot-success" />
                    <span>高性能渲染</span>
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-4">
                <div className="text-center">
                  <div className="data-value text-info text-2xl">
                    <CpuChipIcon className="h-8 w-8 mx-auto" />
                  </div>
                  <div className="data-label">虚拟滚动</div>
                </div>
                <div className="w-px h-12 bg-border" />
                <div className="text-center">
                  <div className="data-value text-primary text-2xl">
                    <RocketLaunchIcon className="h-8 w-8 mx-auto" />
                  </div>
                  <div className="data-label">高效处理</div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Cookie 输入区域 */}
      <Card variant="glass" glow="primary">
        <CardHeader>
          <CardTitle className="text-gradient">
            <CommandLineIcon className="h-6 w-6" />
            批量添加 Cookie
          </CardTitle>
        </CardHeader>
        <CardContent>
          <CookieInput onSubmit={handleCookieSubmitted} />
        </CardContent>
      </Card>

      {/* Cookie 列表区域 - 使用虚拟化列表 */}
      <div className="h-[600px]">
        <VirtualizedCookieList refreshKey={refreshKey} />
      </div>
    </div>
  );
};

export default CookieManagement;
