import http from 'k6/http';
import { check, sleep } from 'k6';

// 目标：低强度冒烟，用于 SQLite 部署验证接口可用性
export const options = {
  vus: 5,
  duration: '1m',
};

const BASE = __ENV.BASE_URL || 'http://127.0.0.1:8484/api';
const TOKEN = __ENV.ADMIN_TOKEN || '';

export default function () {
  const headers = TOKEN ? { Authorization: `Bearer ${TOKEN}` } : {};

  // 1) 版本
  const version = http.get(`${BASE}/version`, { headers });
  check(version, { 'version ok': (r) => r.status === 200 });

  // 2) 统计
  const stats = http.get(`${BASE}/stats/system`, { headers });
  check(stats, { 'stats ok': (r) => r.status === 200 });

  // 3) 提交空 cookie（预期 400）
  const bad = http.post(`${BASE}/cookie`, JSON.stringify({ cookie: '' }), {
    headers: { ...headers, 'Content-Type': 'application/json' },
  });
  check(bad, { 'bad request handled': (r) => r.status === 400 });

  sleep(1);
}
