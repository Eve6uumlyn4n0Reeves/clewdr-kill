import React from 'react';
import { ArrowRightOnRectangleIcon } from '@heroicons/react/24/outline';
import { Button } from '../ui';
import { useAuth } from '../../hooks/useAuth';
import { useAppContext } from '../../context/AppContext';

interface LogoutProps {
  className?: string;
  variant?: 'primary' | 'secondary' | 'ghost';
}

const Logout: React.FC<LogoutProps> = ({ className, variant = 'ghost' }) => {
  const { logout } = useAuth();
  const { setIsAuthenticated } = useAppContext();

  const handleLogout = () => {
    logout();
    setIsAuthenticated(false);
  };

  return (
    <Button
      variant={variant}
      onClick={handleLogout}
      icon={<ArrowRightOnRectangleIcon className="h-5 w-5" />}
      className={className}
    >
      退出登录
    </Button>
  );
};

export default Logout;
