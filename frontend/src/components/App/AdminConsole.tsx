// frontend/src/components/App/AdminConsole.tsx
import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import TabNavigation, { TabType } from "../layout/TabNavigation";
import MainLayout from "../layout/MainLayout";
import Card from "../common/Card";
import AuthGatekeeper from "../auth/AuthGatekeeper";
import CookieInputPanel from "../claude/CookieInputPanel";
import CookieManagerPanel from "../claude/CookieManagerPanel";
import RealTimeStats from "../claude/RealTimeStats";
import Button from "../common/Button";
import StatusMessage from "../common/StatusMessage";
import ErrorBoundary from "../common/ErrorBoundary";
import SystemControls from "../claude/SystemControls";
import AdvancedConfig from "../config/AdvancedConfig";
import { useAppContext } from "../../context/AppContext";

const CookieManagementPanel: React.FC = () => {
  const [refreshKey, setRefreshKey] = useState(0);

  const handleCookieSubmitted = () => {
    setRefreshKey(prev => prev + 1);
  };

  return (
    <div className="space-y-6">
      <CookieInputPanel onSubmit={handleCookieSubmitted} />
      <CookieManagerPanel
        refreshKey={refreshKey}
        onExport={(cookies) => {
          const data = JSON.stringify(cookies, null, 2);
          const blob = new Blob([data], { type: "application/json" });
          const url = URL.createObjectURL(blob);
          const a = document.createElement("a");
          a.href = url;
          a.download = `cookies_export_${new Date().toISOString().split("T")[0]}.json`;
          a.click();
          URL.revokeObjectURL(url);
        }}
      />
    </div>
  );
};

const AdminConsole: React.FC = () => {
  const { t } = useTranslation();
  const { version, isAuthenticated, setIsAuthenticated } = useAppContext();

  const [activeTab, setActiveTab] = useState<TabType>("dashboard");
  const [passwordChanged, setPasswordChanged] = useState(false);

  useEffect(() => {
    // Check if redirected due to password change
    const params = new URLSearchParams(window.location.search);
    if (params.get("passwordChanged") === "true") {
      setPasswordChanged(true);
      window.history.replaceState({}, document.title, window.location.pathname);
    }
  }, []);

  // Function to handle successful authentication
  const handleAuthenticated = (status: boolean) => {
    setIsAuthenticated(status);
  };

  // Function to handle logout
  const handleLogout = () => {
    localStorage.removeItem("authToken");
    setIsAuthenticated(false);
    setActiveTab("dashboard");
  };

  // Render content based on active tab
  const renderTabContent = () => {
    switch (activeTab) {
      case "dashboard":
        return (
          <div className="space-y-6">
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
              <div className="lg:col-span-2 space-y-4">
                <RealTimeStats />
                <SystemControls />
              </div>
              <Card>
                <CookieInputPanel />
              </Card>
            </div>
          </div>
        );
      case "cookies":
        return <CookieManagementPanel />;
      case "stats":
        return <RealTimeStats />;
      case "config":
        return <AdvancedConfig />;
      default:
        return <RealTimeStats />;
    }
  };

  if (!isAuthenticated) {
    return (
      <MainLayout version={version}>
        <Card className="w-full max-w-md sm:max-w-lg md:max-w-xl mx-auto">
          <h2 className="text-xl font-semibold text-center mb-6">
            {t("auth.title")}
          </h2>

          {passwordChanged && (
            <StatusMessage type="info" message={t("auth.passwordChanged")} />
          )}

          <p className="text-gray-400 text-sm mb-6 text-center">
            {t("auth.description")}
          </p>

          <ErrorBoundary>
            <AuthGatekeeper onAuthenticated={handleAuthenticated} />
          </ErrorBoundary>
        </Card>
      </MainLayout>
    );
  }

  return (
    <MainLayout version={version}>
      <ErrorBoundary>
        <div className="w-full max-w-7xl mx-auto space-y-6">
          {/* Header with tab navigation */}
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
            <div>
              <h1 className="text-2xl font-bold text-white">
                控制台
              </h1>
              <p className="text-sm text-gray-400">
                封号策略与管理系统
              </p>
            </div>
            <Button
              onClick={handleLogout}
              variant="secondary"
              className="w-full sm:w-auto"
            >
              {t("auth.logout")}
            </Button>
          </div>

          {/* Tab Navigation */}
          <TabNavigation activeTab={activeTab} onTabChange={setActiveTab} />

          {/* Tab Content */}
          <div className="min-h-[600px]">
            <ErrorBoundary>
              {renderTabContent()}
            </ErrorBoundary>
          </div>
        </div>
      </ErrorBoundary>
    </MainLayout>
  );
};

export default AdminConsole;
