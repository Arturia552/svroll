# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目架构

这是一个 Tauri 应用，结合了 Vue 3 前端和 Rust 后端：

### 前端架构 (Vue 3)
- **框架**: Vue 3 + TypeScript + Vite
- **UI 库**: Element Plus (中文本地化)
- **状态管理**: Pinia with persistence
- **路由**: Vue Router
- **样式**: SCSS + Element Plus themes
- **代码编辑器**: Monaco Editor
- **构建工具**: Vite with auto-imports

### 后端架构 (Rust/Tauri)
- **核心模块**:
  - `mqtt/`: MQTT 客户端管理，设备数据处理，连接集成
  - `tcp/`: TCP 客户端，Modbus 协议解析
  - `task/`: 任务管理，文件处理，命令处理
  - `config/`: 配置管理
  - `model/`: 数据模型，数据库交互
  - `context/`: 应用状态初始化
  - `utils/`: 工具函数

### 核心通信机制
- **rs2js**: Rust 到 JavaScript 的消息传递系统
- **AsyncProcInputTx**: 异步处理输入数据的通道
- **事件发射器**: 通过 Tauri 的 emit 系统进行前后端通信

## 开发命令

```bash
# 前端开发
pnpm dev                    # 启动开发服务器
pnpm build                  # 构建前端 (包含 TypeScript 检查)
pnpm preview                # 预览构建结果
pnpm lint:fix              # 运行 ESLint 并修复问题

# Tauri 开发
pnpm tauri dev             # 启动 Tauri 开发模式
pnpm tauri build           # 构建 Tauri 应用
```

## 数据库
- **SQLite**: 使用 sqlx 进行异步数据库操作
- **配置历史**: 支持配置的历史记录管理
- **初始化**: 应用启动时自动初始化数据库

## 配置系统
- **多层配置**: 基础配置、客户端配置、数据模型配置
- **持久化**: 前端状态通过 Pinia 持久化，后端配置存储在数据库
- **实时更新**: 配置变更通过事件系统实时同步

## 重要特性
- **MQTT 集成**: 完整的 MQTT 客户端管理和设备数据处理
- **TCP/Modbus**: 支持 TCP 连接和 Modbus 协议解析
- **任务管理**: 异步任务处理系统
- **文件操作**: 支持文件读写和配置管理
- **主题切换**: 支持明暗主题切换

## 技术栈重点
- **异步编程**: Rust 端大量使用 Tokio 进行异步处理
- **类型安全**: 前后端都使用 TypeScript/Rust 确保类型安全
- **模块化**: 清晰的模块划分，便于维护和扩展
- **国际化**: 使用中文作为主要语言，Element Plus 中文本地化