# 第三方声明

[English](THIRD_PARTY_NOTICES.md) | 简体中文

本项目包含来自以下开源项目的代码和/或素材。相关许可证见下方。

## chat-demo

- 来源：https://github.com/deepseek-ai/chat-demo
- 版本：0.1.3
- 许可证：MIT
- 版权：Copyright (c) 2026 DeepSeek

应用界面、聊天渲染组件、样式和部分应用结构改编自 `chat-demo`。

完整 MIT 许可证原文见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md#chat-demo)；法律上以英文许可证原文为准。

## @deepseek-ai/dsh-client-ui-primitives

- 来源：https://github.com/deepseek-ai/deepseek-harness
- 包版本：0.2.2-rc.5
- 许可证：MIT
- 版权：Copyright (c) 2026 DeepSeek

本项目使用的全屏/退出全屏图标路径，以及部分 DeepSeek 字标几何，改编自该包。

完整 MIT 许可证原文见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md#deepseek-aidsh-client-ui-primitives)；法律上以英文许可证原文为准。

## 其他依赖

其余 JavaScript 和 Rust 依赖声明于 `app/package.json`、`app/pnpm-lock.yaml` 和 `app/src-tauri/Cargo.toml`，分别遵循其自身许可证。重新分发构建产物时请保留相应的许可证声明。可运行以下命令查看 JavaScript 依赖许可证：

```bash
cd app
pnpm licenses list
```

## 品牌素材

DeepSeek 的 logo、文字标志、产品图标和 DeepSeek 名称**不**在上述 MIT 许可证范围内，详见 [TRADEMARKS.zh-CN.md](TRADEMARKS.zh-CN.md)。
