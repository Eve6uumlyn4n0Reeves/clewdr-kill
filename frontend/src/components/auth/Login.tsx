import React, { useState } from 'react';
import { KeyIcon, EyeIcon, EyeSlashIcon } from '@heroicons/react/24/outline';
import { Button, Input, Card, CardHeader, CardTitle, CardDescription, CardContent, Message } from '../ui';
import { useAuth } from '../../hooks/useAuth';
import { useAppContext } from '../../context/AppContext';
import { getErrorMessage } from '../../utils/errors';

interface LoginProps {
  onAuthenticated: (success: boolean) => void;
}

const Login: React.FC<LoginProps> = ({ onAuthenticated }) => {
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { login } = useAuth();
  const { setIsAuthenticated } = useAppContext();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!password.trim()) {
      setError('请输入管理员密码');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const success = await login(password);
      if (success) {
        setIsAuthenticated(true);
        onAuthenticated(true);
      } else {
        setIsAuthenticated(false);
        setError('密码错误，请重试');
      }
    } catch (err) {
      setIsAuthenticated(false);
      setError(getErrorMessage(err, '登录失败，请稍后重试'));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card className="w-full max-w-md">
      <CardHeader className="text-center">
        <CardTitle className="text-2xl font-bold text-gray-900 dark:text-white">
          管理员登录
        </CardTitle>
        <CardDescription>
          请输入管理员密码以访问控制台
        </CardDescription>
      </CardHeader>

      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Input
              type={showPassword ? 'text' : 'password'}
              label="管理员密码"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="请输入密码"
              icon={<KeyIcon className="h-5 w-5 text-gray-400" />}
              required
            />
            <button
              type="button"
              className="mt-2 text-sm text-primary hover:text-primary-600"
              onClick={() => setShowPassword(!showPassword)}
            >
              {showPassword ? '隐藏密码' : '显示密码'}
            </button>
          </div>

          {error && (
            <Message type="error">
              {error}
            </Message>
          )}

          <Button
            type="submit"
            className="w-full"
            loading={loading}
            disabled={!password.trim()}
          >
            {loading ? '登录中...' : '登录'}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
};

export default Login;
