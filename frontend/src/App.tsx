import { useEffect } from "react";
import {
  BrowserRouter as Router,
  Routes,
  Route,
  Navigate,
} from "react-router-dom";
import ErrorBoundary from "./components/common/ErrorBoundary";
import { useCallback } from "react";
import { useAppContext } from "./context/AppContext";
import MainLayout from "./components/layout/Main";
import { Login } from "./components/auth";
import Dashboard from "./components/Dashboard";
import CookieManagement from "./components/CookieManagement";
import StatsView from "./components/StatsView";
import ConfigView from "./components/ConfigView";
import ThemeTest from "./test-theme";
import { Toaster } from "react-hot-toast";

// Initialize dark mode - now handled in main.tsx
const initDarkMode = () => {
  // Force dark mode for cyberpunk theme
  document.documentElement.classList.add("dark");
};

function App() {
  const handleGlobalError = useCallback((error: Error) => {
    console.error("Global error captured:", error);
  }, []);

  useEffect(() => {
    initDarkMode();
  }, []);

  const { isAuthenticated, setIsAuthenticated } = useAppContext();

  const RequireAuth = ({ children }: { children: React.ReactNode }) => {
    if (!isAuthenticated) {
      return <Navigate to="/login" replace />;
    }
    return children;
  };

  return (
    <ErrorBoundary onError={handleGlobalError}>
      <Router>
        <MainLayout>
          <Routes>
            <Route
              path="/login"
              element={
                isAuthenticated ? (
                  <Navigate to="/" replace />
                ) : (
                  <Login onAuthenticated={() => setIsAuthenticated(true)} />
                )
              }
            />
            <Route
              path="/"
              element={
                <RequireAuth>
                  <Dashboard />
                </RequireAuth>
              }
            />
            <Route
              path="/cookies"
              element={
                <RequireAuth>
                  <CookieManagement />
                </RequireAuth>
              }
            />
            <Route
              path="/stats"
              element={
                <RequireAuth>
                  <StatsView />
                </RequireAuth>
              }
            />
            <Route
              path="/config"
              element={
                <RequireAuth>
                  <ConfigView />
                </RequireAuth>
              }
            />
            <Route path="/theme-test" element={<ThemeTest />} />
            <Route
              path="*"
              element={
                <Navigate to={isAuthenticated ? "/" : "/login"} replace />
              }
            />
          </Routes>
        </MainLayout>
        <Toaster
          position="top-right"
          toastOptions={{
            duration: 4000,
            style: {
              background: "var(--card)",
              color: "var(--foreground)",
              border: "1px solid var(--border)",
            },
          }}
        />
      </Router>
    </ErrorBoundary>
  );
}

export default App;
