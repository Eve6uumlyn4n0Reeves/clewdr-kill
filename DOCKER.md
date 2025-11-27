# Docker 部署指南

ClewdR Kill Edition 的 Docker 容器化部署文档。

## 快速开始

### 1. 构建镜像

```bash
# 克隆仓库
git clone https://github.com/Eve6uumlyn4n0Reeves/clewdr-kill.git
cd clewdr-kill

# 构建镜像
docker compose build
```

### 2. 准备配置

```bash
# 创建必要目录
mkdir -p ban_prompts data

# 添加提示词文件(必须)
echo "Your ban prompt here" > ban_prompts/prompt1.txt

# (可选)自定义配置
cp clewdr.toml.example clewdr.toml
```

### 3. 启动服务

```bash
# 后台启动
docker compose up -d

# 查看日志
docker compose logs -f

# 查看管理员密码(首次启动)
docker compose logs | grep "Generated admin password"
```

### 4. 访问控制台

- 浏览器访问: http://localhost:8484
- 使用生成的管理员密码登录

---

## 配置说明

### 环境变量

所有配置可通过 `docker-compose.yml` 中的环境变量覆盖:

```yaml
environment:
  # 日志配置
  - RUST_LOG=info              # 日志级别: debug/info/warn/error
  - CLEWDR_LOG_FORMAT=json     # JSON格式日志(便于解析)
  
  # 封号策略 - 针对5000个Cookie优化
  - CLEWDR_BAN_CONCURRENCY=10      # 并发worker数(默认10)
  - CLEWDR_BAN_PAUSE_SECONDS=30    # 请求间隔秒数(默认30)
  - CLEWDR_BAN_MAX_TOKENS=1024     # 单次请求最大tokens(默认1024)
  - CLEWDR_BAN_REQUEST_TIMEOUT=30000  # 请求超时毫秒(默认30秒)
  
  # 数据清理 - 自动清理过期数据
  - CLEWDR_CLEANUP_ENABLED=true           # 启用自动清理
  - CLEWDR_CLEANUP_INTERVAL_HOURS=1       # 清理间隔(小时)
  
  # 数据库路径
  - DATABASE_PATH=/app/data/clewdr.db
  
  # (可选)代理配置
  # - CLEWDR_PROXY=http://proxy:8080
  # - CLEWDR_PROXY=socks5://127.0.0.1:1080
  
  # (可选)管理员密码(建议通过clewdr.toml配置)
  # - CLEWDR_ADMIN_PASSWORD=your_secure_password
```

### 资源限制

当前配置针对 **5000个Cookie + 10并发** 优化:

```yaml
deploy:
  resources:
    limits:
      cpus: "1.5"     # 最大1.5个CPU核心
      memory: 768M    # 最大768MB内存
    reservations:
      cpus: "0.5"     # 保证0.5个CPU核心
      memory: 384M    # 保证384MB内存
```

**如果Cookie数量更多,建议调整**:
- 10,000个Cookie: `cpus: 2.0`, `memory: 1G`
- 50,000个Cookie: `cpus: 3.0`, `memory: 2G`

---

## 数据持久化

### 卷挂载

```yaml
volumes:
  # 提示词目录(只读)
  - ./ban_prompts:/app/ban_prompts:ro
  
  # 数据目录(读写,存储数据库)
  - clewdr_data:/app/data
  
  # 临时文件(内存)
  - type: tmpfs
    target: /tmp
```

### 数据备份

```bash
# 备份数据库
docker compose exec clewdr-kill cp /app/data/clewdr.db /app/data/clewdr.db.backup

# 从容器复制到宿主机
docker cp clewdr-kill:/app/data/clewdr.db ./backup-$(date +%Y%m%d).db

# 恢复数据库
docker cp ./backup-20241128.db clewdr-kill:/app/data/clewdr.db
docker compose restart
```

---

## 监控和日志

### 查看日志

```bash
# 实时日志(JSON格式)
docker compose logs -f

# 查看最近100行
docker compose logs --tail=100

# 过滤特定类型日志
docker compose logs | grep "audit=true"     # 审计日志
docker compose logs | grep "alert"          # 告警日志
docker compose logs | grep "cleanup"        # 清理任务日志
```

### 健康检查

```bash
# 检查容器状态
docker compose ps

# 手动健康检查
curl http://localhost:8484/api/health

# 查看死信队列
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8484/api/admin/dead-letters
```

### 监控指标

```bash
# Prometheus metrics
curl http://localhost:8484/metrics

# 系统状态
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8484/api/admin/status
```

---

## 常见操作

### 更新镜像

```bash
# 拉取最新代码
git pull origin master

# 重新构建并启动
docker compose up -d --build
```

### 重启服务

```bash
# 优雅重启
docker compose restart

# 强制重启
docker compose down && docker compose up -d
```

### 清理数据

```bash
# 清空所有数据(危险操作!)
docker compose down -v
rm -rf data/*

# 仅清理日志
docker compose exec clewdr-kill rm -f /app/data/*.log
```

### 调试模式

```bash
# 启用debug日志
docker compose down
# 编辑 docker-compose.yml, 修改 RUST_LOG=debug
docker compose up -d

# 查看详细日志
docker compose logs -f
```

---

## 故障排查

### 1. 容器无法启动

**症状**: `docker compose up` 失败

**检查**:
```bash
# 查看容器日志
docker compose logs

# 检查端口占用
lsof -i :8484

# 检查磁盘空间
df -h
```

### 2. 提示词缺失

**症状**: 日志显示 "No prompts found"

**解决**:
```bash
# 检查提示词目录
ls -la ban_prompts/

# 添加提示词
echo "test prompt" > ban_prompts/test.txt

# 重新加载(无需重启)
curl -X POST -H "Authorization: Bearer $TOKEN" \
     http://localhost:8484/api/prompts/reload
```

### 3. 数据库锁定

**症状**: 日志显示 "database is locked"

**解决**:
```bash
# 检查数据库连接数
docker compose exec clewdr-kill \
    sqlite3 /app/data/clewdr.db "PRAGMA max_page_count;"

# 重启服务释放连接
docker compose restart
```

### 4. 内存不足

**症状**: 容器被OOM killed

**解决**:
```bash
# 检查内存使用
docker stats clewdr-kill

# 增加内存限制(编辑 docker-compose.yml)
memory: 1G  # 从768M增加到1G

# 重新启动
docker compose up -d
```

### 5. 死信队列堆积

**症状**: `/api/admin/dead-letters` 返回大量记录

**解决**:
```bash
# 查看死信队列
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8484/api/admin/dead-letters | jq

# 分析失败原因
docker compose logs | grep "CRITICAL.*mark_processed"

# 清空死信队列(确认问题已修复后)
curl -X POST -H "Authorization: Bearer $TOKEN" \
     http://localhost:8484/api/admin/dead-letters/clear
```

---

## 性能优化

### 针对不同规模调整

#### 50个Cookie (默认)
```yaml
environment:
  - CLEWDR_BAN_CONCURRENCY=5
resources:
  limits:
    cpus: "1.0"
    memory: 512M
```

#### 5,000个Cookie (推荐)
```yaml
environment:
  - CLEWDR_BAN_CONCURRENCY=10
resources:
  limits:
    cpus: "1.5"
    memory: 768M
```

#### 50,000个Cookie
```yaml
environment:
  - CLEWDR_BAN_CONCURRENCY=20
resources:
  limits:
    cpus: "3.0"
    memory: 2G
```

### 数据库优化

```bash
# 进入容器
docker compose exec clewdr-kill sh

# 手动VACUUM
sqlite3 /app/data/clewdr.db "VACUUM;"

# 查看数据库大小
du -h /app/data/clewdr.db

# 查看统计
sqlite3 /app/data/clewdr.db "SELECT status, COUNT(*) FROM cookies GROUP BY status;"
```

---

## 安全建议

### 1. 修改默认密码

```bash
# 首次启动后立即修改
# 通过Web控制台: Settings -> Change Password

# 或通过配置文件(需要bcrypt哈希)
# 编辑 clewdr.toml:
# admin_password = "$2b$12$..."
```

### 2. 限制网络访问

```yaml
# docker-compose.yml 修改端口绑定
ports:
  - "127.0.0.1:8484:8484"  # 仅本地访问
```

### 3. 定期备份

```bash
# 设置每日备份 cron job
0 2 * * * docker cp clewdr-kill:/app/data/clewdr.db /backup/clewdr-$(date +\%Y\%m\%d).db
```

### 4. 日志轮转

```bash
# 配置 Docker 日志驱动
# docker-compose.yml 添加:
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
```

---

## 生产部署检查清单

- [ ] 修改管理员密码
- [ ] 添加提示词文件
- [ ] 配置资源限制(根据Cookie数量)
- [ ] 设置日志轮转
- [ ] 配置数据备份
- [ ] 限制网络访问(仅内网)
- [ ] 启用HTTPS(通过反向代理)
- [ ] 配置监控告警
- [ ] 测试健康检查
- [ ] 验证数据持久化

---

## 技术细节

### 镜像大小

- 最终镜像: ~150MB (Alpine Linux + 静态编译二进制)
- 构建缓存: ~2GB (Rust工具链 + Node依赖)

### 构建优化

- ✅ 多阶段构建减少镜像大小
- ✅ 依赖预构建利用Docker层缓存
- ✅ 静态编译(musl)无运行时依赖
- ✅ 压缩二进制(strip + link-arg=-s)

### 安全特性

- ✅ 非特权用户运行(UID 1001)
- ✅ 只读文件系统(除数据目录)
- ✅ no-new-privileges安全选项
- ✅ 最小化运行时依赖

---

## 参考链接

- 项目仓库: https://github.com/Eve6uumlyn4n0Reeves/clewdr-kill
- API文档: [API_zh.md](./API_zh.md)
- README: [README.md](./README.md)
