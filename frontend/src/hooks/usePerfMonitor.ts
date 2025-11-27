import { useEffect } from 'react';

export const usePerfMonitor = (name: string) => {
  useEffect(() => {
    const start = performance.now();
    return () => {
      const duration = performance.now() - start;
      // 简单输出，后续可接入埋点服务
      console.info(`[perf] ${name} unmounted after ${duration.toFixed(1)}ms`);
    };
  }, [name]);
};

export default usePerfMonitor;
