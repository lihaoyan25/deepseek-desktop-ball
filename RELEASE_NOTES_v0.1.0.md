# DeepSeekBall v0.1.0

English | [简体中文](#简体中文)

An unofficial Windows desktop floating client for DeepSeek, built with Tauri. This release includes the complete local chat experience, conversation search, batch conversation management, and the turn navigation rail.

## Download

- [DeepSeekBall_0.1.0_x64-setup.exe](https://github.com/lihaoyan25/deepseek-desktop-ball/releases/download/v0.1.0/DeepSeekBall_0.1.0_x64-setup.exe)
- [SHA256SUMS.txt](https://github.com/lihaoyan25/deepseek-desktop-ball/releases/download/v0.1.0/SHA256SUMS.txt)

Installer SHA256:

```text
A17C1AD3B651EC6CFBB0C709442D847E3E27F2C73101C89B852F036DBDDBF12E
```

No command line, Node.js, or Rust is required. Download the installer and run it. Windows 10/11 x64 and the WebView2 Runtime are required.

## What's new

- Conversation search for titles and full message history, opened from the sidebar or the collapsed sidebar capsule.
- Batch selection with pin, unpin, and batch delete actions.
- Pinned conversation group with persistent `pinnedAt` state.
- Turn navigation rail with resting marks, age fading, hover previews, and click-to-jump navigation.
- Softer sidebar collapse/expand, mobile drawer, and button hover transitions.
- Sidebar search and batch-management entry points that match the official desktop layout.
- Fixed context-menu positioning after the trigger loses hover.
- Fixed the maximized window becoming topmost; only the mini window is kept on top.
- Added the DeepSeek usage and API-key management link in Settings.
- Updated installer, bilingual documentation, and release metadata.

## Installation

1. Download `DeepSeekBall_0.1.0_x64-setup.exe`.
2. Run the installer and follow the prompts.
3. Launch DeepSeekBall and enter your own DeepSeek API key in Settings.
4. Windows SmartScreen may warn because the installer is unsigned; choose **Run anyway** only if you downloaded the file from this release.

## Notes

This is an unofficial project. It is not affiliated with, endorsed by, or sponsored by DeepSeek. DeepSeek brand assets remain the property of their respective owners. See [TRADEMARKS.md](https://github.com/lihaoyan25/deepseek-desktop-ball/blob/main/TRADEMARKS.md).

---

## 简体中文

一个非官方的 Windows DeepSeek 桌面悬浮客户端，使用 Tauri 构建。本版本包含完整的本地聊天体验、会话搜索、批量会话管理以及轮次导航。

## 下载

- [DeepSeekBall_0.1.0_x64-setup.exe](https://github.com/lihaoyan25/deepseek-desktop-ball/releases/download/v0.1.0/DeepSeekBall_0.1.0_x64-setup.exe)
- [SHA256SUMS.txt](https://github.com/lihaoyan25/deepseek-desktop-ball/releases/download/v0.1.0/SHA256SUMS.txt)

安装包 SHA256：

```text
A17C1AD3B651EC6CFBB0C709442D847E3E27F2C73101C89B852F036DBDDBF12E
```

无需命令行、Node.js 或 Rust。下载安装包后直接运行即可。系统要求为 Windows 10/11 x64，并需要 WebView2 Runtime。

## 本版内容

- 支持在侧栏或折叠胶囊中打开会话搜索，同时匹配标题和完整消息内容。
- 支持批量选择会话，并执行置顶、取消置顶和批量删除。
- 置顶会话独立分组，并持久化 `pinnedAt` 状态。
- 对话页右侧增加轮次导航：普通 mark、按时间衰减、悬停预览和点击跳转。
- 侧栏收起/展开、移动端抽屉和按钮 hover 使用更柔和的过渡动画。
- 侧栏搜索和批量管理的入口、布局与官方桌面端对齐。
- 修复三点菜单在触发元素失去 hover 后跳到左上角的问题。
- 修复放大模式被强制置顶的问题；现在只有迷你模式保持置顶。
- 设置页增加 DeepSeek 用量与 API KEY 管理链接。
- 更新安装包、双语文档和 Release 元数据。

## 安装

1. 下载 `DeepSeekBall_0.1.0_x64-setup.exe`。
2. 运行安装包并按提示完成安装。
3. 启动 DeepSeekBall，在设置中填写自己的 DeepSeek API Key。
4. 安装包未签名，Windows SmartScreen 可能提示；只有在文件确实来自本 Release 时才选择“仍要运行”。

## 说明

本项目为非官方项目，与 DeepSeek 无隶属、背书或赞助关系。DeepSeek 品牌素材的权利归其权利人所有，详见 [TRADEMARKS.zh-CN.md](https://github.com/lihaoyan25/deepseek-desktop-ball/blob/main/TRADEMARKS.zh-CN.md)。
