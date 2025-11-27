# 管理员 API 手册（简版）

所有接口均需管理员鉴权：

1. 调用 `POST /auth/login`，在 body 中提交 `{ "password": "<管理员密码>" }` 获取 JWT。
2. 所有后续请求都要携带：

```
Authorization: Bearer <token>
Content-Type: application/json
```

默认基地址：`http://127.0.0.1:8484/api`

## 1. 鉴权与版本

- `POST /auth/login`：校验管理员密码，返回 `{ token, expires_at }`。
- `GET /auth`：校验 JWT 是否有效（HTTP 200/401）。
- `GET /version`：返回当前版本字符串。

## 2. Cookie 队列

- `POST /cookie`
  - Body：`{ "cookie": "<cookie_string>" }`
  - 功能：将 Cookie 加入封号队列。
- `GET /cookies`
  - 返回：`{ pending: [...], processing: [...], banned: [...], total_requests: number }`
  - 功能：查询队列状态与累计请求数（processing 列表便于观察正在处理的 Cookie）。
- `DELETE /cookie`
  - Body：`{ "cookie": "<cookie_string>" }`
  - 功能：从 pending/banned 中删除指定 Cookie。
- `POST /cookie/check`
  - Body：`{ "cookie": "<cookie_string>" }`
  - 返回：`{ alive: bool, banned: bool, last_checked: RFC3339, error?: string }`
  - 功能：立即测活，判断是否已被封禁。

## 3. 统计

- `GET /stats/system`
  - 返回全局统计：队列规模、总请求数、请求速率、成功率、平均响应、活跃 worker 数、性能指标等。
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
  - Body：`{ "ban_config": { ... }, "server_config"?: { ... }, "network_config"?: { ... } }`
  - 功能：更新配置并落盘；ban 部分支持并发/模型/提示词目录等，server/network 支持 IP/端口/代理/密码。
- `POST /config/reset`
  - 功能：恢复默认配置。
- `POST /config/validate`
  - Body：同上，返回校验结果（errors/warnings）。
- `GET /config/export`
  - 返回：去敏的配置导出。
- `POST /config/import`
  - Body：导出的配置 JSON，支持 `merge_mode`（replace/merge）。
- `GET /config/templates`
  - 返回：预置的场景模板（aggressive/stealth/balanced）。

## 5. 管理动作

- `POST /admin/action`
  - Body：`{ "action": "<action_name>" }`
  - 允许的 action：
    - `pause_all`：暂停 ban worker
    - `resume_all`：恢复 ban worker
    - `reset_stats`：重置统计（队列计数+策略指标）
    - `clear_all`：清空 pending 与 banned 队列
    - `emergency_stop`：紧急停止（维护态，停止 worker）
- `GET /admin/status`
  - 返回当前系统状态（状态、活跃 worker、队列大小、总请求等）。
- `GET /admin/health`
  - 返回组件健康信息（队列等）。

## 6. 限流与并发

- 限流：
  - 登录接口：每 IP 每分钟 5 次
  - Cookie 接口：每 IP 每分钟 60 次
- 并发：
  - `ban.concurrency` 支持前端/配置接口动态修改，后端会热重启 worker。若设置 `CLEWDR_DISABLE_WORKERS=1`，并发变更仅写配置不生效（日志会提示）。

## 7. 数据持久化说明

- 队列、统计和配置全部持久化在 SQLite `clewdr.db`，初始化时自动创建表、索引及 WAL 设置。
- 管理员密码仅以 Bcrypt 哈希形式写入 `clewdr.toml`；留空时启动会生成一次随机强密码并在控制台打印，请立即修改。
- 迁移：启动/测试自动执行 `./migrations`，即便暂无数据也必须存在以保证表结构。

## 常见错误

- 401：JWT 无效或缺失，或登录密码错误导致未获取令牌。
- 400：请求体格式不合法（如 Cookie 无效）。
- 500：内部错误，查看后台日志获取具体原因。
