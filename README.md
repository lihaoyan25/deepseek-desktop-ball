# DeepSeek Floating Ball (Unofficial)

English | [绠€浣撲腑鏂嘳(README.zh-CN.md)

An unofficial Windows desktop floating client for DeepSeek, built with Tauri. The chat interface is based on [chat-demo](https://github.com/deepseek-ai/chat-demo); the floating ball, window management, and desktop integration are implemented by this project.

> This project is not affiliated with, endorsed by, or sponsored by DeepSeek. DeepSeek brand assets remain the property of their respective owners. See [TRADEMARKS.md](TRADEMARKS.md).

## Download

Download the latest installer from [Releases](https://github.com/lihaoyan25/deepseek-desktop-ball/releases/latest):

- [DeepSeekBall_0.1.0_x64-setup.exe](https://github.com/lihaoyan25/deepseek-desktop-ball/releases/latest/download/DeepSeekBall_0.1.0_x64-setup.exe)
- [SHA256SUMS.txt](https://github.com/lihaoyan25/deepseek-desktop-ball/releases/latest/download/SHA256SUMS.txt)

No Node.js, Rust, or other development environment is required. Windows 10/11 x64 with the WebView2 Runtime is required. On first launch, enter your own DeepSeek API key in Settings.

The installer is unsigned. If Windows SmartScreen appears, choose **Run anyway**.

### Controls

- `Ctrl + Shift + Space`: show or hide the panel.
- Click the floating ball: restore the last mini or maximized mode.
- Drag the floating ball: move it; near a screen edge it tucks into a tail.
- Mini mode, top-right: new chat, maximize, hide.
- Maximized mode, top-right: restore mini mode, hide. The window can be dragged and stays inside the primary monitor work area.

## Features

- Floating ball with drag, edge tucking, and single-instance behavior.
- Compact 400脳640 mini panel, plus a normal maximized window that is not forced on top.
- Markdown, syntax highlighting, KaTeX math, thinking traces, deep thinking, smart search, and source citations.
- Conversation search in the sidebar, full-history message search, pinned sessions, and batch pin/delete.
- Turn navigation rail with prompt/reply previews and click-to-jump navigation.
- Local conversation history with branched replies, editing, regeneration, import, and export.
- Drag-and-drop attachments, voice input, read-aloud replies, system tray, and a global shortcut.
- Simplified Chinese and English UI, light and dark themes, and smooth interface transitions.

## Data and storage

Your API key and settings are stored in local application data. Conversations and attachments are stored in local browser storage. This project does not provide a backend service or embed the official DeepSeek website. Sending messages, uploading attachments, or using speech features sends the relevant content to the configured API service. This project collects no telemetry.

Do not commit API keys, `.env` files, local application data, or logs to Git.

## Development

Requirements: Node.js 24+, pnpm 11+, Rust stable with the Windows GNU toolchain, MSYS2 MinGW-w64, WebView2 Runtime, and NSIS for the installer.

```sh
cd app
pnpm install
pnpm tauri dev
```

The development server runs at `http://localhost:8090` by default.

| Command                        | Purpose                          |
| ------------------------------ | -------------------------------- |
| `pnpm tauri dev`               | Start the desktop dev environment |
| `pnpm tauri build --no-bundle` | Build the release application     |
| `pnpm type`                    | Run TypeScript type checks        |
| `pnpm lint`                    | Lint with oxlint                  |
| `pnpm format`                  | Format with oxfmt                 |

Release artifacts:

```text
app/src-tauri/target/release/app.exe
app/src-tauri/target/release/WebView2Loader.dll
```

Build the Windows installer:

```sh
makensis installer/DeepSeek.nsi
```

The installer is written to `dist-installer/DeepSeekBall_0.1.0_x64-setup.exe` and also installs the license and bilingual third-party/brand notices.

Project structure:

```text
app/
  src/                       React chat UI, floating ball, and settings
  src-tauri/                 Tauri/Rust window, tray, and installer configuration
installer/DeepSeek.nsi        Windows installer script
LICENSE                       Project MIT license
THIRD_PARTY_NOTICES.md        Upstream notices (English)
THIRD_PARTY_NOTICES.zh-CN.md  Upstream notices (Chinese)
TRADEMARKS.md                 Unofficial notice and brand-asset policy (English)
TRADEMARKS.zh-CN.md           Unofficial notice and brand-asset policy (Chinese)
```

## Known limitations

- Windows only.
- Maximized-window placement and edge clamping currently use the primary monitor.
- The mini panel is fixed at 400脳640.
- The application does not include the official account system, official sync, or share links.
- Branded UI is used only to identify the compatible service and does not imply an official implementation.

## License

The source code and project documentation are licensed under [MIT](LICENSE). The upstream chat-demo and `@deepseek-ai/dsh-client-ui-primitives` packages are also MIT-licensed; their notices and full license texts are reproduced in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

The DeepSeek name, logo, wordmark, and related brand assets are not covered by the MIT license. See [TRADEMARKS.md](TRADEMARKS.md). Before commercial distribution or a large public release, confirm brand usage permission or replace the brand assets.

## Acknowledgements

- [DeepSeek chat-demo](https://github.com/deepseek-ai/chat-demo): chat interface and style foundation.
- [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness): some official UI icon and wordmark geometry.
- [Tauri](https://tauri.app/): desktop windows and packaging.
- React, rspack, KaTeX, Mermaid, and all other upstream dependencies.
