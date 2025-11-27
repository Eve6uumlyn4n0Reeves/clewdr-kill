// frontend/src/context/AppContext.tsx
import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import { apiClient } from "../api";

interface AppContextType {
  version: string;
  isAuthenticated: boolean;
  setIsAuthenticated: (status: boolean) => void;
}

const defaultContext: AppContextType = {
  version: "",
  isAuthenticated: false,
  setIsAuthenticated: () => {},
};

const AppContext = createContext<AppContextType>(defaultContext);

interface AppProviderProps {
  children: ReactNode;
}

export const AppProvider: React.FC<AppProviderProps> = ({ children }) => {
  const [version, setVersion] = useState("");
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    // Fetch and set the version when component mounts
    apiClient.getVersion().then((v) => setVersion(v));

    // Check for authentication status
    const checkAuth = async () => {
      const storedToken = localStorage.getItem("authToken");
      if (storedToken) {
        const ok = await apiClient.validateToken(storedToken);
        if (ok) {
          setIsAuthenticated(true);
          return;
        }
        localStorage.removeItem("authToken");
      }
      setIsAuthenticated(false);
    };

    checkAuth();
  }, []);

  return (
    <AppContext.Provider
      value={{
        version,
        isAuthenticated,
        setIsAuthenticated,
      }}
    >
      {children}
    </AppContext.Provider>
  );
};

export const useAppContext = () => useContext(AppContext);
