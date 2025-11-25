# 管理员 API 手册（简版）

所有接口均需管理员鉴权，请在请求头携带：

```
Authorization: Bearer <admin_password>
Content-Type: application/json
```

默认基地址：`http://127.0.0.1:8484/api`

## 1. 鉴权与版本

- `GET /auth`：校验管理员密码是否有效（HTTP 200/401）。
- `GET /version`：返回当前版本字符串。

## 2. Cookie 队列

- `POST /cookie`
  - Body：`{ "cookie": "<cookie_string>" }`
  - 功能：将 Cookie 加入封号队列。
- `GET /cookies`
  - 返回：`{ pending: [...], banned: [...], total_requests: number }`
  - 功能：查询队列状态与累计请求数。
- `DELETE /cookie`
  - Body：`{ "cookie": "<cookie_string>" }`
  - 功能：从 pending/banned 中删除指定 Cookie。
- `POST /cookie/check`
  - Body：`{ "cookie": "<cookie_string>" }`
  - 返回：`{ alive: bool, banned: bool, last_checked: RFC3339, error?: string }`
  - 功能：立即测活，判断是否已被封禁。

## 3. 统计

- `GET /stats/system`
  - 返回全局统计：总 Cookie、pending/banned 数、总请求数、请求速率、成功率、平均响应、活跃 worker 数、性能指标等。
- `GET /stats/cookies`
  - 返回每个 Cookie 的请求数、平均响应等指标。
- `POST /stats/historical`
  - Body：`{ "interval_minutes": number, "points"?: number }`
  - 返回按时间序列的请求数、成功率、响应时间、错误率。
- `POST /stats/reset`
  - 功能：重置统计并清空策略指标。

## 4. 配置（ban 配置）

- `GET /config`
  - 返回：`{ ban_config, server_config, network_config }`
- `POST /config`
  - Body：`{ "ban_config": { ... } }`（支持部分字段）
  - 功能：更新 ban 配置并落盘。
- `POST /config/reset`
  - 功能：恢复默认配置。
- `POST /config/validate`
  - Body：同上，返回校验结果（errors/warnings）。
- `GET /config/export`
  - 返回：去敏的配置导出。
- `POST /config/import`
  - Body：导出的配置 JSON，功能与 `POST /config` 一致。
- `GET /config/templates`
  - 返回：预置的场景模板（aggressive/stealth/balanced）。

## 5. 管理动作

- `POST /admin/action`
  - Body：`{ "action": "<action_name>" }`
  - 允许的 action：
    - `pause_all`：暂停 ban worker
    - `resume_all`：恢复 ban worker
    - `reset_stats`：重置统计
    - `clear_queue`：清空 pending
    - `clear_banned`：清空 banned
    - `emergency_stop`：紧急停止（设置维护态）
- `GET /admin/status`
  - 返回当前系统状态（状态、活跃 worker、队列大小、总请求等）。
- `GET /admin/health`
  - 返回组件健康信息（队列等）。

## 6. 数据持久化说明

- 目前仅使用本地文件快照 `queue_state.json`（存放在配置同级目录）来恢复队列与统计；默认 `ban.prompts_dir = "./ban_prompts"`（项目根目录）。
- RateLimiter 默认关闭（max_requests=0，自用不做限流），如需限流需自行改代码。

## 常见错误

- 401：管理员密码错误或缺失。
- 400：请求体格式不合法（如 Cookie 无效）。
- 500：内部错误，查看后台日志获取具体原因。
