# ClewdR Kill Edition

一个高效的Claude Cookie封号工具。专注于最大化封号效率，采用Rust后端和React前端，使用SQLite进行数据持久化。

## 🚀 极简部署（推荐）

**三步完成部署**：

```bash
# 1. 克隆并进入项目
git clone https://github.com/Eve6uumlyn4n0Reeves/clewdr-kill.git
cd clewdr-kill

# 2. 创建提示词文件（必需）
mkdir -p ban_prompts
echo "你的封号提示词内容" > ban_prompts/prompt1.txt

# 3. 启动服务
docker compose up -d
```

**访问控制台**：浏览器打开 http://localhost:8484

**查看密码**：`docker compose logs | grep "Generated admin password"`

**停止服务**：`docker compose down`

**更新版本**：`git pull && docker compose up -d --build`

详细配置说明见 [DOCKER.md](./DOCKER.md)

---

## 特性

- ⚡ **高效封号** - 默认使用 Haiku 模型，成本效益最大化
- 🚀 **Aggressive 模式** - 20并发，30秒延迟，最快封号速度
- 💰 **成本优化** - 优先使用便宜模型，每次请求2048 tokens
- 📊 **实时监控** - 系统状态、封号进度实时展示
- 🎯 **批量处理** - 支持批量添加和管理Cookie
- ⏱️ **年龄加权重试** - 越接近48小时越高频轰炸，rate-limit 冷却自动缩短
- 🌙 **现代界面** - 暗色主题，响应式设计
- 💾 **SQLite存储** - 无需外部数据库，开箱即用

## 技术栈

### 后端 (Rust)
- **框架**: Axum + Tokio - 高性能异步框架
- **数据库**: SQLite (sqlx) - 轻量级，零配置
- **HTTP客户端**: wreq - 简单可靠的HTTP库
- **日志**: tracing - 结构化日志系统

### 前端 (TypeScript/React)
- **框架**: React 18 + TypeScript
- **UI库**: Tailwind CSS + Headless UI
- **图标**: Heroicons
- **图表**: Recharts
- **状态管理**: React Context
- **构建工具**: Vite

## 项目结构

```
clewdr-kill/
├── backend/                 # 后端源代码（4,465行）
│   ├── db/                 # 数据库模块
│   │   ├── connection.rs   # 数据库连接和迁移
│   │   ├── models.rs       # 数据模型
│   │   └── queries.rs      # 优化的SQL查询
│   ├── api/                # RESTful API接口
│   ├── services/           # 核心业务逻辑
│   │   ├── ban_farm.rs     # 高效封号引擎
│   │   ├── ban_queue.rs    # Cookie队列管理
│   │   └── ban_strategy.rs # 封号策略
│   └── config/             # 配置管理
├── frontend/               # 前端源代码
│   └── src/
│       ├── components/
│       │   ├── ui/         # 基础UI组件
│       │   ├── auth/       # 认证组件
│       │   ├── cookies/    # Cookie管理
│       │   ├── stats/      # 统计展示
│       │   └── layout/     # 布局组件
│       └── ...
├── ban_prompts/            # 禁用提示词目录
├── clewdr.toml            # 配置文件
└── Cargo.toml             # Rust依赖
```

## 快速开始

### 环境要求

- Rust 1.70+
- Node.js 18+
- npm 或 yarn

### 安装和运行

1. **克隆仓库**
   ```bash
   git clone https://github.com/Xerxes-2/clewdr.git
   cd clewdr
   ```

2. **构建并运行后端**
   ```bash
   cargo build --release
   cargo run
   ```

   服务器将在 `http://127.0.0.1:8484` 启动

   > 首次启动会自动执行 `./migrations` 建表，即使还没有任何业务数据也需要这一步。若自定义数据库路径或手动初始化，可执行：
   > `DATABASE_PATH=/your/path.db sqlx migrate run -D sqlite://$DATABASE_PATH`

3. **安装并运行前端**
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

   如需自定义前端调用后端的 API 基地址（例如通过反向代理或子路径部署），可以在 `frontend/.env` 中设置：

   ```bash
   VITE_API_BASE_URL=http://127.0.0.1:8484/api
   ```

   未配置时，前端默认使用 `/api` 作为基路径。

4. **访问控制台**
   - 打开浏览器访问 `http://127.0.0.1:8484`
   - 使用控制台显示的管理员密码登录

## 配置说明

### 基础配置 (clewdr.toml)

```toml
# 服务器设置
ip = "0.0.0.0"
port = 8484

# 管理员密码（务必自行设置）
# 可通过环境变量 CLEWDR_ADMIN_PASSWORD 或在此文件显式配置
# 留空时启动会自动生成强密码并写回 clewdr.toml，但不会再打印明文
admin_password = ""

# 封号策略配置（默认为Aggressive模式）
[ban]
concurrency = 20              # 高并发线程数
pause_seconds = 30           # 快速请求间隔
prompts_dir = "./ban_prompts" # 提示词目录
models = [                   # 优先使用便宜模型
    "claude-3-5-haiku-20241022",
    "claude-3-7-sonnet-20250219"
]
max_tokens = 2048             # 大tokens确保prompt完整执行
```

- 密码始终以 Bcrypt 哈希形式写入磁盘；通过 API 或 `CLEWDR_ADMIN_PASSWORD` 设置的明文仅用于计算哈希并立即丢弃。
- 若 `admin_password` 留空，第一次启动会生成随机高强度密码并只在控制台打印一次，请及时记录并更改。
- 通过前端保存配置修改 `ban.concurrency` 时，后端会热重启 worker 以套用新并发；若设置了 `CLEWDR_DISABLE_WORKERS`，并发变更会记录警告但不会生效，需重启后端。

### 环境变量

- `RUST_LOG`: 设置日志级别 (debug/info/warn/error)
- `DATABASE_PATH`: 自定义SQLite数据库路径（可选）
- `CLEWDR_DISABLE_WORKERS`: 启动时暂停 BanFarm worker，适用于离线测试；此时修改并发不会自动生效，需要重启或移除该变量。
- `CLEWDR_*`: 环境变量优先于 `clewdr.toml`，Docker/Docker Compose 配置应与文件保持一致，避免混用不同来源导致配置漂移。

## API文档

### 认证
- `POST /api/auth/login`：提交管理员密码，获取一次性 JWT 令牌，响应中包含 `token` 与 `expires_at`。
- 所有后续 API 请求需在请求头中携带 `Authorization: Bearer <token>`。
- `GET /api/auth`：校验 JWT 是否仍然有效，可用于前端心跳与自动续期。

### 核心接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/auth` | 校验当前JWT是否有效 |
| POST | `/api/auth/login` | 使用管理员密码获取JWT |
| POST | `/api/cookie` | 添加Cookie到队列 |
| GET | `/api/cookies` | 获取所有Cookie状态 |
| DELETE | `/api/cookie` | 删除指定Cookie |
| POST | `/api/cookie/check` | 检测Cookie状态 |
| GET | `/api/stats/system` | 获取系统统计 |

详细API文档请参考 [API_zh.md](API_zh.md)

## 使用指南

### 0. 管理员登录
- 启动后访问控制台，输入配置中的管理员密码或首次启动时控制台打印的随机密码。
- 系统会获取 JWT 并保存在浏览器，随时可在设置中注销/更换。

### 1. 添加Cookie
- 在控制台访问Cookie管理页面
- 粘贴或输入Cookie（每行一个）
- 系统自动验证格式并添加到队列

### 2. 监控状态
- 仪表板显示实时统计信息
- 查看待处理和已封禁的Cookie数量
- 监控请求成功率和平均响应时间

### 3. 配置策略
- **Aggressive模式**：20并发，30秒延迟 - 最大化封号速度
- **Balanced模式**：5并发，2分钟延迟 - 平衡速度和稳定性
- **Stealth模式**：1并发，10分钟延迟 - 避免触发限制
- 优先使用 Haiku 模型以降低成本
- 2048 tokens 确保提示词完整执行

## 数据管理

### 数据存储
- Cookie/队列/统计/配置都存储在 SQLite 中，启动或测试会自动运行 `./migrations` 建表。
- 数据库文件位置：`clewdr.db`（配置目录下，可通过 `DATABASE_PATH` 重定向）。
- 适用场景：单机或低至中等规模（数万级 Cookie）以内，SQLite 结合 WAL 模式和 20 连接池足够；高并发/分布式再考虑 Postgres/Redis。

### 数据备份
```bash
# 备份数据库
cp clewdr.db clewdr.db.backup

# 恢复数据库
cp clewdr.db.backup clewdr.db
```

### 架构要点（适用于 SQLite 单机部署）
- 后端：Axum + Tokio，单进程内含 BanFarm worker 池；`ban.concurrency` 可热更新并重启 worker；全局限流（登录 5/min，Cookie 接口 60/min，按 IP）。
- 队列：存于 SQLite，`BanQueue` 负责出入队与状态回写；`mark_processed` 失败会记录告警避免静默。
- Prompt：从 `ban_prompts` 加载，可在配置中切换目录并热加载；目录为空会暂停 worker 并在前端提示，补齐提示词后自动恢复。
- 观测：`/stats/*` 提供系统/历史/队列指标；前端对系统统计做了短期缓存，减少轮询压力。
- 维护：`/admin/action` 支持暂停/恢复/清空/重置统计/紧急停止；`CLEWDR_DISABLE_WORKERS=1` 便于离线测试，此时并发调整仅写配置不启动 worker。
- 迁移：`sqlx::migrate!` 自动执行，首次即需建表，即便还没有业务数据。
- 数据生命周期：如需定期清理长期封禁的 Cookie，可调用 `Queries::cleanup_old_cookies(pool, days)`（删除 `banned` 且 `updated_at` 超出天数的记录）。
- 策略解耦：封号策略实现了 `StrategyExecutor` trait，BanFarm 仅依赖接口，后续可替换策略或注入 mock 进行测试。

## 观测与审计

- **日志格式**：设置 `CLEWDR_LOG_FORMAT=json` 可输出 JSON 结构化日志（含 target/file/line），默认文本。
- **审计日志**：登录成功/失败、配置更新、提示词新增/删除、管理员操作均记录 `audit=true` 字段。
- **Metrics**：`GET /metrics` 暴露 Prometheus 文本指标（队列长度、total_requests、worker 数等），可被 Prometheus/Grafana 抓取。
- **OpenAPI**：`GET /api/docs/openapi.json` 提供 OpenAPI 3 文档（核心接口）。
- **健康检查**：`GET /api/health` 基础存活检查；提示词缺失会暂停 worker 并在日志/前端提示。
- **错误返回**：统一格式 `{ success:false, error:{ message, code } }`，错误码见 `frontend/src/types/api.types.ts` 与后端 `ErrorCode`。

## 调度与重试策略

- **出队顺序**：严格 FIFO（按 `created_at ASC`），最早录入优先。
- **年龄加权**：距创建 ≥24h 时，将请求间隔缩短为原配置的 1/2；≥40h 缩短为 1/3（下限 2s），实现“逼近48小时越猛”。
- **rate-limit 冷却**：遇到 429/限流时，为该 Cookie 设置冷却：
  - ≥40h：10 分钟；≥24h：20 分钟；其他：30 分钟。冷却期内不会出队，期满自动重试。
- **封禁判定**：检测到 banned/401/403 即标记为 `banned`，不再重放。
- **提示词缺失**：提示词为空时暂停 worker 并提示，保存/导入提示词后自动重载并恢复。

## 测试与工具

- 单元/集成：`cargo test --tests`（含 BanFarm 热重启、队列、配置校验等）。
- 前端：`cd frontend && npm install && npm test -- --runInBand`（vitest）。
- 端到端：提供 Playwright 脚本 `tests/e2e/playwright.spec.ts`，需安装 `@playwright/test` 后运行 `npx playwright test tests/e2e/playwright.spec.ts`。
- 压测：`tests/perf/k6-smoke.js`，运行示例：
  ```bash
  BASE_URL=http://127.0.0.1:8484/api ADMIN_TOKEN=your_token k6 run tests/perf/k6-smoke.js
  ```

## Docker部署

1. **构建镜像**
   ```bash
   docker build -t clewdr:latest .
   ```

2. **运行容器**
   ```bash
   docker run -d \
     --name clewdr \
     -p 8484:8484 \
     -v $(pwd)/ban_prompts:/app/ban_prompts \
     -v $(pwd)/data:/app/data \
     clewdr:latest
   ```

3. **使用Docker Compose**
   ```yaml
   version: '3.8'
   services:
     clewdr:
       build: .
       ports:
         - "8484:8484"
       volumes:
         - ./ban_prompts:/app/ban_prompts
         - ./data:/app/data
       environment:
         - RUST_LOG=info
   ```

## 故障排除

### 常见问题

1. **数据库初始化失败**
   - 检查目录权限
   - 确保有足够的磁盘空间

2. **Cookie验证失败**
   - 确保Cookie格式正确
   - 检查是否包含特殊字符

3. **连接超时**
   - 检查网络配置
   - 调整请求超时设置

### 日志查看

```bash
# 查看详细日志
RUST_LOG=debug cargo run

# 查看错误日志
tail -f logs/clewdr.log
```

## 开发指南

### 开发环境设置

1. **安装开发依赖**
   ```bash
   rustup component add rustfmt clippy
   npm install -g eslint prettier
   ```

2. **代码格式化**
   ```bash
   # Rust代码
   cargo fmt
   cargo clippy

   # TypeScript代码
   cd frontend
   npm run lint
   npm run format
   ```

3. **运行测试**
   ```bash
   cargo test
   cd frontend && npm test
   ```

### 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

本项目采用 AGPL-3.0 许可证。详见 [LICENSE](LICENSE) 文件。

## 致谢

- [wreq](https://github.com/0x676e67/wreq) - HTTP客户端库
- [Clewd](https://github.com/teralomaniac/clewd) - 参考实现
- [Tailwind CSS](https://tailwindcss.com/) - CSS框架
- [Axum](https://github.com/tokio-rs/axum) - Web框架

## 更新日志

### v0.11.27 (最新) - 极简高效版本
- ⚡ **默认Aggressive模式**：20并发，30秒延迟，默认 Haiku
- 📊 **统计缓存**：前端 `useSystemStats` 启用缓存与错误分支处理
- 🛡️ **全局错误兜底**：ErrorBoundary 支持 onError，可接入上报
- 🧭 **性能监测**：Dashboard/统计/列表接入 `usePerfMonitor`
- 🔍 **体验优化**：Cookie 列表防抖搜索 + 骨架屏加载态
- ✅ **测试增强**：前端单测覆盖到缓存/部分成功/重置失败等场景；启用覆盖率门槛

### v0.11.26
- ✨ 重构前端为现代化UI
- 🗃️ 集成SQLite数据库
- 🎨 支持暗色主题
- 📱 优化移动端适配

---

## 效率说明

- **Haiku模型成本**：约为 Sonnet 的 1/10
- **默认配置效率**：20并发，每分钟最多 20 次请求
- **日均封号能力**：28,800 次（理论最大值）

---

⚠️ **免责声明**：本工具仅供学习和研究使用，请遵守相关法律法规。
