# ClewdR


ClewdR Kill Edition 将原版 ClewdR 改造成了一个“Claude Cookie 封号机”。  
现在只暴露一个管理员 API：提交 Claude Cookie，内置的 ban farm 立刻用预设文本持续请求这些账号直至触发封禁。React 界面被精简为仅保留鉴权与封号队列视图。

---

## 核心特点

- 自动化封号：提交 Claude Cookie 后，ban farm 即刻用随机组合的违禁文本持续请求。
- 内置 Prompt Loader（默认读取项目根目录下的 `./ban_prompts`，可改为 `eat_claude/txt` 等目录），复刻 @eat_claude 脚本的随机策略。
- 通过 `ban.concurrency` 设置全局工作线程，遇到限流/过载等错误时按 `ban.pause_seconds` 触发“全局退避”暂停所有 worker。
- 自用默认不做限流（RateLimiter 上限为 0），需要限流可自行调整代码。
- 管理面板仅剩登录页 + Cookie 队列视图（提交、列表、删除、单 Cookie 立即测活）。
- 数据持久化：默认使用本地文件快照 `queue_state.json`（随配置目录存放），重启后恢复队列与统计；当前未内置数据库存储。

> 如需对接你自己的“上游项目”，详见根目录的 `API_zh.md` 管理员 API 文档。

## 管理员 API 概览（Ban 相关）

所有管理员 API 默认挂在 `http://127.0.0.1:8484/api` 下，并要求请求头携带：

```text
Authorization: Bearer <admin_password>
Content-Type: application/json
```

核心接口（详情见 `API_zh.md`）：

| 功能         | 方法 & 路径                | 说明                                  |
|--------------|----------------------------|---------------------------------------|
| 鉴权检测     | `GET /api/auth`            | 校验管理员密码是否有效               |
| 版本信息     | `GET /api/version`         | 返回当前运行版本字符串               |
| 提交 Cookie  | `POST /api/cookie`         | 将 Cookie 加入封号队列               |
| 队列状态     | `GET /api/cookies`         | 获取 pending/banned 队列与统计       |
| 立即测活     | `POST /api/cookie/check`   | 直接向 Claude 测试单个 Cookie 状态   |
| 删除 Cookie  | `DELETE /api/cookie`       | 从队列与 banned 集合中移除           |

## 快速开始

1. 从 GitHub Releases 下载对应平台的最新版。  
   Linux/macOS 示例：
   ```bash
   curl -L -o clewdr.tar.gz https://github.com/Xerxes-2/clewdr/releases/latest/download/clewdr-linux-x64.tar.gz
   tar -xzf clewdr.tar.gz && cd clewdr-linux-x64
   chmod +x clewdr
   ```
2. 运行二进制：
   ```bash
   ./clewdr
   ```
3. 打开 `http://127.0.0.1:8484`，使用控制台（或 Docker 容器日志）显示的管理员密码登录。

## Web 管理界面（封号控制台）

- 登录页：输入管理员密码（控制台首次启动时自动生成），通过后进入封号控制台。
- Cookie 提交表单：支持一次粘贴多行 Cookie，逐行入队，前端实时展示成功/失败数。
- Cookie 队列视图：
  - Pending 区：展示待封禁 Cookie 的提交时间、最后使用时间、发送请求数，可删除、可“立即测活”。
  - Banned 区：展示已判定封禁的 Cookie，同样支持删除与再次测活。
  - 顶部汇总：总 Cookie 数、Pending/Banned 数量、ban farm 累计发送请求数，以及最近一次刷新时间。

如忘记密码，删除 `clewdr.toml` 再启动即可。Docker 建议挂载该文件所在目录以持久化。

## 上游项目接入流程（概要）

一个典型的上游服务可以按下面的模式接入：

1. 首次接入：从 ClewdR 控制台或 `clewdr.toml` 中拿到管理员密码。  
2. 每当有新号需要封禁时：
   - 调用 `POST /api/cookie` 提交 Cookie，仅根据 HTTP 状态判断“是否接单成功”。  
3. 每天（或你自定义的周期）对每个 Cookie 调用：
   - `POST /api/cookie/check`  
   - 如果 `alive == true`：说明号还活着，下一天继续问。  
   - 如果 `alive == false && banned == true`：说明大概率已封，对应调用 `DELETE /api/cookie` 将其从队列中移除。  
4. 队列和统计可选：
   - 需要可视化时调用 `GET /api/cookies`，接入你自己的后台面板。

## Ban Farm 配置

ban worker 会从配置的 `ban.prompts_dir` 目录读取 `.txt` 文本，随机组合后追加噪声并发送到每个 Cookie。默认目录为项目根目录下的 `./ban_prompts`（不是 frontend 内部），你可以改成任意本地路径（例如兼容 `eat_claude/txt`）。

`clewdr.toml` 示例：

```toml
[ban]
concurrency = 50           # 同时运行的后台线程上限（默认 50，可按需调整）
pause_seconds = 18000      # 检测到限流/过载后的全局休眠时长（秒）
prompts_dir = "./ban_prompts"
models = [
  "claude-3-7-sonnet-20250219",
  "claude-sonnet-4-20250514"
]
```

向目录里新增 `.txt` 文件即可扩充攻击语料，重启后自动生效。

## 持久化与限流

- 队列快照：`queue_state.json` 位于配置文件同级目录，保存 pending/banned/total_requests，重启会自动加载。
- 配置：`clewdr.toml`（可通过 API 读写），忘记密码可删除该文件后重启生成新密码。
- RateLimiter：默认关闭（max_requests = 0），如需开启请修改 `default_rate_limiter` 或创建自定义实例。

## Docker 部署

已提供多阶段 `Dockerfile`（前端构建 + 后端构建 + 运行时）：

```bash
# 构建镜像（构建上下文已忽略 ban_prompts）
docker build -t clewdr:latest .

# 运行：挂载根目录的 ban_prompts 与数据目录
docker run -d --name clewdr \
  -p 8484:8484 \
  -v /your/path/ban_prompts:/app/ban_prompts \
  -v /your/path/data:/app/data \
  clewdr:latest
```

- ban_prompts 必须挂载在容器 `/app/ban_prompts`（默认配置 `./ban_prompts` 指向根目录）。  
- `clewdr.toml` 与 `queue_state.json` 可放在 `/app/data`，也可用 `--config` 指向宿主路径（自行挂载）。  
- 如需自定义配置文件，先在宿主创建，再通过 `-v /your/path/clewdr.toml:/app/data/clewdr.toml` 挂载。

## 资源

- Wiki：<https://github.com/Xerxes-2/clewdr/wiki>  
  - 数据库持久化指南（中文）：`wiki/database.md`

## 致谢

- [wreq](https://github.com/0x676e67/wreq) 提供指纹识别能力。  
- [Clewd](https://github.com/teralomaniac/clewd) 提供参考实现。  
- [Clove](https://github.com/mirrorange/clove) 提供 Claude Code 相关逻辑。
