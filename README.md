# DeepSeek Floating Ball (Unofficial)

English | [简体中文](README.zh-CN.md)

An unofficial Windows desktop floating client for DeepSeek, built with Tauri. The
chat interface is based on [chat-demo](https://github.com/deepseek-ai/chat-demo);
the floating ball, window management, and desktop integration are implemented by
this project.

> This is an unofficial project. It is not affiliated with, endorsed by, or
> sponsored by DeepSeek. The DeepSeek name, whale logo, wordmark, and related
> brand assets remain the property of their respective owners. See
> [TRADEMARKS.md](TRADEMARKS.md).

## Getting started

Download the latest `DeepSeekBall_0.1.0_x64-setup.exe` from
[Releases](../../releases) and run it. No Node.js, Rust, or other development
environment is required.

Make sure the WebView2 Runtime is installed. Windows 11 usually includes it; on
Windows 10, install Microsoft Edge WebView2 Runtime if it is missing. On first
launch, enter your own DeepSeek API key in Settings.

The installer is unsigned, so Windows SmartScreen may show a warning. Choose
"Run anyway" to continue.

Common controls:

- `Ctrl + Shift + Space`: show or hide the panel.
- Click the floating ball: restore the mini or maximized mode that was active
  when the panel was last hidden.
- Drag the floating ball: move it; near a screen edge it collapses into a tail.
- Mini mode, top-right: new chat, maximize, hide.
- Maximized mode, top-right: exit maximize, hide. The maximized window can be
  dragged and stays inside the monitor work area.

## Features

- A floating ball that stays on top, can be dragged, and tucks into the screen
  edge; clicking it restores the previous mode.
- A 400×640 borderless mini panel with a compact single-line composer.
- A maximized mode that uses 90% of the monitor work area and the desktop
  typography, composer, and disclaimer from chat-demo. The maximized window is a
  normal window, is not forced on top, and appears in the taskbar.
- Markdown, syntax highlighting, KaTeX math, thinking traces, deep thinking,
  smart search, and a source sidebar.
- Drag-and-drop attachments, single-instance behavior, a system tray icon, and a
  global shortcut.
- Simplified Chinese and English UI, with light and dark themes.

## Data

The API key and settings are stored in local application data. Conversations and
attachments are stored in the local browser storage. This project does not provide
a backend service and does not embed the official DeepSeek website. Sending
messages, uploading attachments, or using speech features submits the relevant
content to the configured API service. This project collects no telemetry.

Do not commit API keys, `.env` files, local application data, or logs to Git.

## Development

Requirements: Node.js 24+, pnpm 11+, Rust stable with the Windows GNU toolchain,
MSYS2 MinGW-w64, and the WebView2 Runtime. Building the installer also requires
NSIS.

```sh
cd app
pnpm install
pnpm tauri dev
```

The development server runs at `http://localhost:8090` by default.

| Command                        | Purpose                         |
| ------------------------------ | ------------------------------- |
| `pnpm tauri dev`               | Start the desktop dev environment |
| `pnpm tauri build --no-bundle` | Build the release application    |
| `pnpm type`                    | Run TypeScript type checks       |
| `pnpm lint`                    | Lint with oxlint                 |
| `pnpm format`                  | Format with oxfmt                |

Release artifacts:

```text
app/src-tauri/target/release/app.exe
app/src-tauri/target/release/WebView2Loader.dll
```

Build the Windows installer:

```sh
makensis installer/DeepSeek.nsi
```

The output is `dist-installer/DeepSeekBall_0.1.0_x64-setup.exe`. The installer
also installs `LICENSE.txt`, `THIRD_PARTY_NOTICES.txt`, and `TRADEMARKS.txt`.

## Project structure

```text
app/
  src/                       React chat UI, floating ball, and settings
  src/assets/                Brand assets and application icons
  src-tauri/                 Tauri/Rust window, tray, and installer configuration
installer/DeepSeek.nsi        Windows installer script
LICENSE                       Project MIT license
THIRD_PARTY_NOTICES.md        Upstream notices for chat-demo, DSH, and others
TRADEMARKS.md                 Unofficial notice and brand-asset policy
PUBLISHING.md                 GitHub release instructions
```

## Known limitations

- Windows only.
- Maximized-window placement and edge clamping currently use the primary monitor.
- The mini panel is fixed at 400×640.
- The application does not include the official account system, official sync, or
  batch conversation management.
- Branded UI is used only to identify the compatible service and does not imply
  an official implementation.

## License

The source code and project documentation are licensed under [MIT](LICENSE). The
upstream chat-demo and `@deepseek-ai/dsh-client-ui-primitives` packages are also
MIT-licensed; their copyright notices and full license texts are reproduced in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

The DeepSeek name, logo, wordmark, and related brand assets are not covered by the
MIT license. Their rights remain with their respective owners. See
[TRADEMARKS.md](TRADEMARKS.md). Before commercial distribution or a large public
release, confirm brand usage permission or replace the brand assets.

## Acknowledgements

- [DeepSeek chat-demo](https://github.com/deepseek-ai/chat-demo): chat interface and style foundation.
- [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness): some official UI icon and wordmark geometry.
- [Tauri](https://tauri.app/): desktop windows and packaging.
- React, rspack, KaTeX, Mermaid, and all other upstream dependencies.
