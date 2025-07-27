# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 Tauri 的桌面应用程序 "svroll"，提供 MQTT 和 TCP 客户端功能，前端使用 Vue.js。应用支持协议测试、数据采集和配置管理，主要用于物联网/网络场景。

## 架构设计

### 前端 (Vue.js + TypeScript)
- **框架**: Vue 3 组合式 API，TypeScript，Element Plus UI 组件库
- **状态管理**: Pinia 状态管理，支持数据持久化
- **构建工具**: Vite 配置自定义插件
- **核心服务**:
  - `configManager.ts`: 配置验证和类型转换
  - `eventManager.ts`: Tauri 事件处理和回调管理
  - `tauriService.ts`: 前后端通信服务

### 后端 (Rust + Tauri)
- **核心模块**:
  - `mqtt/`: MQTT 客户端实现，连接管理
  - `tcp/`: TCP 客户端，支持 Modbus 协议
  - `task/`: 任务管理和文件处理
  - `model/`: 数据库操作 (SQLite) 和数据结构
  - `config/`: 配置管理和验证

### 通信模式
- 前端通过 Tauri 命令与 Rust 后端通信
- 后端通过 `rs2js` 事件向前端发送实时更新
- 使用 tokio 通道进行异步处理和并发操作

## 开发命令

### 前端开发
```bash
# 启动开发服务器（仅前端）
pnpm dev

# 构建前端
pnpm build

# 类型检查
vue-tsc --noEmit

# 代码规范检查和修复
pnpm lint:fix
```

### Tauri 开发
```bash
# 启动完整 Tauri 开发环境（前端 + 后端）
pnpm tauri dev

# 构建生产版本应用
pnpm tauri build
```

### 后端开发
```bash
# 在 src-tauri 目录下执行
cargo check
cargo test
cargo build
```

## 核心实现模式

### 配置管理
- 使用 `ConfigManager` 类处理所有配置操作
- 通过 `convertType()` 工具进行类型验证
- 使用 `createDefaultConfig()` 创建默认配置

### 事件处理
- 后端通过 `lib.rs:31` 中的 `rs2js()` 函数发送消息
- 前端通过 `EventManager` 类处理事件
- 消息类型：`counter`、`clientInfo`、`terminal`

### 数据库操作
- SQLite 数据库在 `model/database.rs` 中管理
- 配置历史记录存储和检索
- 使用 sqlx 进行异步数据库操作

### 协议实现
- MQTT：使用 `rumqttc` 库进行客户端连接
- TCP：自定义实现，支持 Modbus 协议解析
- 两者都支持并发客户端连接和连接池

## 测试

目前未配置测试框架。添加测试时：
- Rust 代码：使用标准的 `cargo test`
- 前端代码：在 package.json 中配置 Jest 或 Vitest

## 文件结构说明

- `src-tauri/src/`: Rust 后端代码
- `src/`: Vue.js 前端代码
- `vite/plugins/`: 自定义 Vite 插件，用于自动导入和组件注册
- 配置文件使用 JSON 和自定义解析
- 启用 TypeScript 严格模式，配置路径别名 (`@/` → `src/`)