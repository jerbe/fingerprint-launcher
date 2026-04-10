# Fingerprint Launcher 开发文档

## 1. 开发环境

- Rust >= 1.75
- 依赖 crate:
  - `eframe` - egui 桌面框架
  - `rusqlite` (features: bundled) - SQLite
  - `serde` / `serde_json` - 序列化
  - `aes-gcm` - 密码加密
  - `chrono` - 时间处理
  - `uuid` - 生成指纹ID
  - `rfd` - 文件选择对话框

## 2. 开发阶段

### Phase 1: 基础框架
- [x] 项目初始化 (Cargo.toml, 目录结构)
- [x] 数据库初始化与表创建
- [x] 基础 egui 窗口

### Phase 2: 核心功能
- [ ] 浏览器管理 (系统设置)
- [ ] 平台与账号管理
- [ ] 启动项 CRUD
- [ ] 启动项分页

### Phase 3: 启动与集成
- [ ] 浏览器进程启动
- [ ] 启动项关联账号
- [ ] 启动项浏览器参数校验 (不允许为空)

### Phase 4: 完善
- [ ] 密码加密存储
- [ ] 搜索功能
- [ ] UI 美化

## 3. 构建与运行

```bash
cargo run          # 开发模式运行
cargo build --release  # 发布构建
```

## 4. 数据库文件

默认存储在应用目录下 `fingerprint_launcher.db`。
