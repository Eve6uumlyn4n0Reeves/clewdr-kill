# 测试目录说明

本目录包含项目的所有测试代码，采用分层测试策略。

## 目录结构

```
tests/
├── unit/               # 单元测试
│   ├── api/           # API 层单元测试
│   ├── services/      # 服务层单元测试
│   ├── db/            # 数据库层单元测试
│   └── utils/         # 工具函数测试
├── integration/       # 集成测试
│   ├── api_tests/     # API 端到端测试
│   ├── db_tests/      # 数据库集成测试
│   └── service_tests/ # 服务集成测试
├── e2e/               # 端到端测试
│   ├── auth_flow.rs   # 认证流程测试
│   └── ban_flow.rs    # 封号流程测试
├── fixtures/          # 测试数据
│   ├── cookies.txt    # 测试用 Cookie
│   ├── configs.toml   # 测试配置
│   └── prompts/       # 测试 prompt 文件
├── mocks/             # Mock 对象
│   ├── claude_api.rs  # Claude API Mock
│   └── db_mock.rs     # 数据库 Mock
└── common/            # 公共测试工具
    ├── mod.rs         # 测试工具模块
    ├── setup.rs       # 测试环境搭建
    └── helpers.rs     # 测试辅助函数
```

## 测试类型

### 1. 单元测试 (Unit Tests)
测试单个函数或模块的功能，不涉及外部依赖。

**运行方式：**
```bash
cargo test --lib
cargo test --lib -- --test-threads=1  # 单线程运行
```

**示例：**
- Cookie 解析逻辑
- 配置验证逻辑
- 错误处理逻辑

### 2. 集成测试 (Integration Tests)
测试多个模块之间的交互，使用真实或接近真实的依赖。

**运行方式：**
```bash
cargo test --test '*'
cargo test --test integration_tests
```

**示例：**
- API 路由 + 数据库交互
- BanFarm + BanQueue 协作
- 认证中间件 + JWT 管理

### 3. 端到端测试 (E2E Tests)
测试完整的业务流程，模拟真实用户操作。

**运行方式：**
```bash
cargo test --test e2e_tests
```

**示例：**
- 完整的登录 → 添加 Cookie → 封号流程
- 配置更新 → Worker 重启流程

## 测试数据管理

### Fixtures
位于 `fixtures/` 目录，包含测试所需的静态数据。

**使用方式：**
```rust
use std::fs;

#[test]
fn test_with_fixture() {
    let cookie = fs::read_to_string("tests/fixtures/cookies.txt")
        .expect("Failed to read fixture");
    // 测试逻辑
}
```

### Mocks
位于 `mocks/` 目录，提供外部依赖的模拟实现。

**使用方式：**
```rust
use crate::mocks::claude_api::MockClaudeApi;

#[tokio::test]
async fn test_with_mock() {
    let mock_api = MockClaudeApi::new();
    // 测试逻辑
}
```

## 测试覆盖率

**生成覆盖率报告：**
```bash
# 使用 tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage

# 或使用 llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

**目标覆盖率：**
- 核心业务逻辑: 80%+
- API 层: 70%+
- 工具函数: 90%+

## 测试最佳实践

### 1. 命名规范
```rust
// 单元测试
#[test]
fn test_parse_valid_cookie() { }

#[test]
fn test_parse_invalid_cookie_returns_error() { }

// 集成测试
#[tokio::test]
async fn test_submit_cookie_api_success() { }
```

### 2. 测试隔离
每个测试应该独立运行，不依赖其他测试的状态。

```rust
#[tokio::test]
async fn test_isolated() {
    // 创建独立的测试数据库
    let db = setup_test_db().await;
    
    // 测试逻辑
    
    // 清理
    cleanup_test_db(db).await;
}
```

### 3. 使用 Setup/Teardown
```rust
use crate::common::setup::TestContext;

#[tokio::test]
async fn test_with_context() {
    let ctx = TestContext::new().await;
    
    // 测试逻辑
    
    // TestContext 实现 Drop，自动清理
}
```

### 4. 断言清晰
```rust
// 好的断言
assert_eq!(result.status, CookieStatus::Banned, 
    "Cookie should be banned after 403 response");

// 避免模糊的断言
assert!(result.is_ok()); // 不清楚期望什么
```

## 性能测试

使用 `criterion` 进行基准测试：

```bash
cargo install cargo-criterion
cargo criterion
```

**示例：**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_cookie(c: &mut Criterion) {
    c.bench_function("parse_cookie", |b| {
        b.iter(|| parse_cookie(black_box("sk-ant-sid01-xxx")))
    });
}

criterion_group!(benches, bench_parse_cookie);
criterion_main!(benches);
```

## CI/CD 集成

在 GitHub Actions 中运行测试：

```yaml
- name: Run tests
  run: |
    cargo test --all-features
    cargo test --doc
```

## 故障排查

### 测试失败常见原因
1. **数据库未初始化**：确保测试前创建测试数据库
2. **端口冲突**：使用随机端口或确保端口未被占用
3. **文件权限**：确保测试目录有读写权限
4. **异步测试超时**：增加超时时间或优化测试逻辑

### 调试测试
```bash
# 显示测试输出
cargo test -- --nocapture

# 运行单个测试
cargo test test_name -- --exact

# 显示忽略的测试
cargo test -- --ignored
```

## 前端测试

前端测试位于 `frontend/src/__tests__/` 目录。

**运行方式：**
```bash
cd frontend
npm test
npm run test:coverage
```

## 贡献指南

添加新功能时，请确保：
1. 编写对应的单元测试
2. 如涉及多模块交互，添加集成测试
3. 更新相关的测试文档
4. 确保测试覆盖率不降低
