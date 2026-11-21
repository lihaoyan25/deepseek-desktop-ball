# DeepSeek 悬浮球（非官方）

[English](README.md) | 简体中文

一个面向 Windows 的 DeepSeek 桌面悬浮客户端，使用 Tauri 构建。聊天界面基于
[chat-demo](https://github.com/deepseek-ai/chat-demo)，悬浮球、窗口管理和桌面集成由本项目实现。

> 本仓库为非官方项目，与 DeepSeek 官方无隶属、合作或背书关系。DeepSeek 名称、
> 鲸鱼标志、文字标志及相关品牌素材的权利归其权利人所有。详见
> [TRADEMARKS.md](TRADEMARKS.md)。

## 运行

前往 [Releases](../../releases) 下载最新的 `DeepSeekBall_0.1.0_x64-setup.exe`，
双击安装。使用者不需要安装 Node.js、Rust 或其他开发环境。

首次运行前请确认系统已安装 WebView2 Runtime。Windows 11 通常已包含；Windows 10
如未安装，请先安装 Microsoft Edge WebView2 Runtime。首次打开应用时，在设置中填写
自己的 DeepSeek API Key。

安装包未签名，Windows SmartScreen 可能提示。选择“仍要运行”即可继续。

常用操作：

- `Ctrl + Shift + Space`：显示或隐藏面板。
- 点击悬浮球：恢复上次收起时的迷你或放大模式。
- 拖动悬浮球：移动位置；靠近屏幕边缘时收起为尾巴。
- 迷你模式右上角：新对话、放大、收起。
- 放大模式右上角：退出放大、收起；放大窗口可拖动，并保持在屏幕工作区内。

## 功能

- 悬浮球置顶显示，可拖动和贴边收起；点击后按上次的模式恢复。
- 迷你模式为 400×640 无边框面板，输入框采用单行紧凑布局。
- 放大模式按屏幕工作区的 90% 显示，使用 chat-demo 的桌面字号、输入框和免责声明；
  放大窗口不强制置顶，并出现在任务栏。
- 支持 Markdown、代码高亮、KaTeX 公式、思考过程、深度思考、智能搜索和来源侧栏。
- 支持文件拖放、单实例运行、系统托盘和全局快捷键。
- 提供简体中文和英文界面，以及浅色和深色主题。

## 数据

API Key 和设置保存在本机应用数据中，会话和附件保存在本地浏览器存储中。本项目不
提供后端服务，也不内嵌 DeepSeek 官方网页。发送消息、上传附件或使用语音功能时，
相应内容会提交给所配置的 API 服务。本项目不收集遥测数据。

请不要将 API Key、`.env` 文件、应用本地数据或日志提交到 Git。

## 开发

需要 Node.js 24+、pnpm 11+、Rust stable（Windows GNU toolchain）、MSYS2
MinGW-w64 和 WebView2 Runtime。构建安装包还需要 NSIS。

```sh
cd app
pnpm install
pnpm tauri dev
```

开发服务器默认运行在 `http://localhost:8090`。

| 命令                           | 用途                       |
| ------------------------------ | -------------------------- |
| `pnpm tauri dev`               | 启动桌面开发环境           |
| `pnpm tauri build --no-bundle` | 构建 release 产物          |
| `pnpm type`                    | TypeScript 类型检查        |
| `pnpm lint`                    | 使用 oxlint 检查           |
| `pnpm format`                  | 使用 oxfmt 格式化          |

构建产物位于：

```text
app/src-tauri/target/release/app.exe
app/src-tauri/target/release/WebView2Loader.dll
```

生成 Windows 安装包：

```sh
makensis installer/DeepSeek.nsi
```

输出为 `dist-installer/DeepSeekBall_0.1.0_x64-setup.exe`。安装包会同时安装
`LICENSE.txt`、`THIRD_PARTY_NOTICES.txt` 和 `TRADEMARKS.txt`。

## 项目结构

```text
app/
  src/                       React 聊天界面、悬浮球和设置
  src/assets/                品牌素材与应用图标
  src-tauri/                 Tauri/Rust 窗口、托盘和安装配置
installer/DeepSeek.nsi        Windows 安装包脚本
LICENSE                       项目 MIT 许可证
THIRD_PARTY_NOTICES.md        chat-demo、DSH 等上游许可
TRADEMARKS.md                 非官方声明与品牌素材说明
PUBLISHING.md                 GitHub Release 发布说明
```

## 已知限制

- 仅支持 Windows。
- 放大窗口和边界收束目前以主显示器为准。
- 迷你面板固定为 400×640。
- 不包含官方客户端的账号体系、官方同步或会话批量管理。
- 品牌相关界面用于识别所兼容的服务，不代表官方实现。

## 许可证

源代码和项目文档使用 [MIT](LICENSE) 许可证。上游 chat-demo 和
`@deepseek-ai/dsh-client-ui-primitives` 同样使用 MIT 许可证，完整版权与许可文本见
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

DeepSeek 名称、logo、文字标志及相关品牌素材不属于 MIT 许可范围，权利归其权利人
所有，详见 [TRADEMARKS.md](TRADEMARKS.md)。商业分发或大规模公开发布前，请自行
确认品牌使用权限，或替换品牌素材。

## 致谢

- [DeepSeek chat-demo](https://github.com/deepseek-ai/chat-demo)：聊天界面和样式基础。
- [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness)：部分官方 UI 图标与字标几何。
- [Tauri](https://tauri.app/)：桌面窗口和打包。
- React、rspack、KaTeX、Mermaid 及其他上游依赖的作者。
