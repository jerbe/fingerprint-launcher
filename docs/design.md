# Fingerprint Launcher 设计文档

## 1. 技术选型

| 组件 | 选型 |
|------|------|
| 语言 | Rust |
| GUI框架 | egui (eframe) |
| 数据库 | SQLite (rusqlite) |
| 序列化 | serde + serde_json |
| 加密 | aes-gcm |

## 2. 数据库设计

### browsers 表 (浏览器配置)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| name | TEXT NOT NULL | 浏览器名称 |
| exe_path | TEXT NOT NULL | 可执行文件路径 |
| icon_path | TEXT | 图标路径 |
| created_at | TEXT | 创建时间 |
| updated_at | TEXT | 更新时间 |

### platforms 表 (平台)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| name | TEXT NOT NULL | 平台名称 (GitHub, OpenAI等) |
| icon | TEXT | 图标标识 |
| created_at | TEXT | 创建时间 |

### accounts 表 (账号)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| platform_id | INTEGER FK | 关联平台 |
| username | TEXT NOT NULL | 用户名 |
| password | TEXT NOT NULL | 加密后的密码 |
| remark | TEXT | 备注 |
| created_at | TEXT | 创建时间 |
| updated_at | TEXT | 更新时间 |

### profiles 表 (启动项)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| name | TEXT NOT NULL | 启动项名称 |
| fingerprint_id | TEXT NOT NULL | 指纹ID |
| user_data_dir | TEXT NOT NULL | 用户数据文件夹 |
| created_at | TEXT | 创建时间 |
| updated_at | TEXT | 更新时间 |

### profile_accounts 表 (启动项-账号关联)

| 字段 | 类型 | 说明 |
|------|------|------|
| profile_id | INTEGER FK | 启动项ID |
| account_id | INTEGER FK | 账号ID |
| PRIMARY KEY | (profile_id, account_id) | 联合主键 |

### profile_browsers 表 (启动项-浏览器配置)

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| profile_id | INTEGER FK | 启动项ID |
| browser_id | INTEGER FK | 浏览器ID |
| launch_args | TEXT NOT NULL | 启动参数 (不允许为空) |
| created_at | TEXT | 创建时间 |
| updated_at | TEXT | 更新时间 |

## 3. 模块架构

```
fingerprint-launcher/
  src/
    main.rs            # 入口
    app.rs             # egui App 主结构
    db/
      mod.rs           # 数据库初始化与连接
      browser.rs       # 浏览器 CRUD
      platform.rs      # 平台 CRUD
      account.rs       # 账号 CRUD
      profile.rs       # 启动项 CRUD
    models/
      mod.rs           # 模型导出
      browser.rs       # Browser 结构体
      platform.rs      # Platform 结构体
      account.rs       # Account 结构体
      profile.rs       # Profile 及关联结构体
    ui/
      mod.rs           # UI 模块导出
      main_view.rs     # 主界面 (启动项列表)
      profile_edit.rs  # 启动项编辑弹窗
      settings.rs      # 系统设置 (浏览器管理)
      accounts.rs      # 账号管理界面
    crypto.rs          # 密码加密/解密
    launcher.rs        # 浏览器进程启动逻辑
```

## 4. 界面设计

### 主界面布局
- 顶部: 工具栏 (新建启动项、搜索、设置入口)
- 左侧: 导航栏 (启动项列表、账号管理、设置)
- 中央: 启动项卡片列表 (分页)，每个卡片显示名称、指纹ID、浏览器图标按钮

### 启动项编辑
- 弹窗/面板形式
- 表单: 名称、指纹ID、用户文件夹
- 浏览器参数区: 系统中配置的每个浏览器都显示一行，必须填写参数
- 关联账号: 多选列表

### 账号管理
- 左侧: 平台树形列表
- 右侧: 选中平台的账号列表
- 支持视图切换 (树形/列表)
