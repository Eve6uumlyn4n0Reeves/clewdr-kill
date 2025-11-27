import { useState, useEffect } from 'react';
import { apiClient } from '../api';
import { maskToken } from '../utils/formatters';
import { useAppContext } from '../context/AppContext';

export const useAuth = (onAuthenticated?: (status: boolean) => void) => {
  const { isAuthenticated, setIsAuthenticated } = useAppContext();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [savedToken, setSavedToken] = useState('');

  // Check for existing token on mount
  useEffect(() => {
    const storedToken = localStorage.getItem('authToken');
    if (storedToken) {
      setSavedToken(maskToken(storedToken));
      checkToken(storedToken);
    }
  }, []);

  const checkToken = async (tokenFromStorage?: string) => {
    setIsLoading(true);
    setError('');

    try {
      const token = tokenFromStorage ?? localStorage.getItem('authToken') ?? '';
      const isValid = await apiClient.validateToken(token);

      if (isValid) {
        if (tokenFromStorage) {
          setSavedToken(maskToken(tokenFromStorage));
        }
        setIsAuthenticated(true);
        onAuthenticated?.(true);
        return true;
      } else {
        localStorage.removeItem('authToken');
        setSavedToken('');
        setIsAuthenticated(false);
        setError('认证失败，请重新登录');
        onAuthenticated?.(false);
        return false;
      }
    } catch (err) {
      setIsAuthenticated(false);
      setError(err instanceof Error ? err.message : '认证失败');
      onAuthenticated?.(false);
      return false;
    } finally {
      setIsLoading(false);
    }
  };

  const login = async (password: string) => {
    if (!password.trim()) {
      setError('请输入密码');
      return false;
    }

    setIsLoading(true);
    setError('');

    try {
      const response = await apiClient.login(password);

      localStorage.setItem('authToken', response.token);
      setSavedToken(maskToken(response.token));
      setIsAuthenticated(true);
      onAuthenticated?.(true);
      return true;
    } catch (err) {
      setIsAuthenticated(false);
      setError(err instanceof Error ? err.message : '登录失败');
      onAuthenticated?.(false);
      return false;
    } finally {
      setIsLoading(false);
    }
  };

  const logout = () => {
    localStorage.removeItem('authToken');
    setSavedToken('');
    setIsAuthenticated(false);
    onAuthenticated?.(false);
  };

  return {
    isAuthenticated,
    isLoading,
    error,
    savedToken,
    login,
    logout,
  };
};
