# Drill

[English](README.md) | **简体中文**

Drill 是一个自托管的刷题工具，开箱即用。题库就是纯 Markdown 文件，可以非常方便的与 Obsidian 和 各种 AI 工具配合使用。一题一屏的全屏沉浸刷题，也可以导出 PDF 供打印或分发。

## 功能

**题库**

- 题库即纯 Markdown 文件：一题一个 `.md` 文件，支持公式、图片。
- 题目格式请参考 [template.md](template.md)。
- 支持多个题库文件夹（题源），在应用内管理。

**刷题**

- 全屏沉浸刷题，带计时，快捷键可重新绑定。
- 对答案模式：答案常显，适用于对打印出来的纸质题本进行快速核对。

- 可对题目进行收藏、标记为熟练等。

**题本**

- 支持单科目、多科目题库，无需配置，只要在题目的元信息中包含科目即可。
- 可按指定顺序或乱序生成题本。
- 可以管理多个题本。

**导出**

- PDF 导出：便于打印、分发，提供丰富的排版选项。
- Markdown 导出：便于粘贴到其它位置，或与 Obsidian 或 AI 工具配合使用。

**界面**

- 浅色/深色/跟随系统主题
- 方便的整体界面缩放
- 可配合 SVG 使图片同时适应浅色和深色主题。
- 方便的复制 LaTeX 和 Markdown 源码。
- 触屏、键鼠友好。
- 支持 PWA，在浏览器中“添加到主屏幕”可获得近乎原生应用的体验。

## 使用

从 [GitHub Releases](https://github.com/wzh19960613/drill/releases) 下载发布包——Windows x64（zip）、Linux x64（tar.gz）或 macOS Apple Silicon（tar.gz）——解压后在目录内运行：

```bash
DRILL_DIST=frontend/dist DRILL_ROOT=/path/to/bank DRILL_DATA=/path/to/data ./drill-backend
# Windows (PowerShell)：
#   $env:DRILL_DIST="frontend\dist"; $env:DRILL_ROOT="C:\path\to\bank"; $env:DRILL_DATA="C:\path\to\data"; .\drill-backend.exe
```

浏览器打开 http://127.0.0.1:8787；包内 `RELEASE.md` 有更详细的说明。也可以直接运行预构建的 Docker 镜像，见 [Docker](#docker)。

首次启动时，服务端会把内置示例题库和演示题本写入数据目录——不配置任何东西，所有页面也能开箱即用。要刷自己的题，把 Markdown 文件放进一个文件夹、让 `DRILL_ROOT` 指向它，或在应用内通过题库页的题源管理（题库 → 题源）添加；内容实时从磁盘重读。

环境变量（均可选）：

| 变量         | 默认值           | 用途                                                       |
| ------------ | ---------------- | ---------------------------------------------------------- |
| `DRILL_HOST` | `0.0.0.0`        | 监听地址                                                    |
| `DRILL_PORT` | `8787`           | 监听端口                                                    |
| `DRILL_ROOT` | 仓库根目录       | 题库根；显式设置时，首次启动将其本身注册为默认题源            |
| `DRILL_DATA` | `backend/data`   | 数据目录：做题记录、题本、题源、熟练标记、导出缓存          |
| `DRILL_DIST` | `frontend/dist`  | 后端托管的前端静态资源目录                                  |

## PWA

Drill 可以像原生应用一样安装：安装后从主屏幕或桌面图标启动，以独立窗口打开，没有浏览器的地址栏和标签页。

**iOS / iPadOS（Safari）**

1. 在 Safari 中打开 Drill。
2. 点击工具栏中的「分享」按钮。
3. 选择「添加到主屏幕」，再点「添加」。

**Android（Chrome）**

1. 在 Chrome 中打开 Drill。
2. 点右上角「⋮」菜单（或地址栏中的安装图标）。
3. 选择「安装应用」。

**桌面浏览器**

- Chrome / Edge：点击地址栏右侧的安装图标，或通过菜单选择「安装应用」/「将此站点作为应用安装」。
- Safari（macOS）：菜单栏「文件」→「添加到 Dock」。

安装的快捷方式指向当时访问的服务器地址，使用时需保证后端正在运行。

## Docker

```bash
docker run -d --name drill -p 8787:8787 -v /your/data:/data ghcr.io/wzh19960613/drill:latest
```

把你的 Markdown 题库放进容器内的 `/data/bank`（镜像已设 `DRILL_ROOT=/data/bank`）；做题记录、题本与导出都持久化在挂载的 `/data` 卷里。镜像在每次 GitHub 发布时推送到 GHCR。

## 构建

从源码构建：

```bash
# 前端；产物在 frontend/dist，由后端托管
cd frontend && npm ci && npm run build

# 后端
cd backend && cargo build --release --locked
```

开发时直接运行后端，并配合前端开发服务器：

```bash
cd backend && cargo run --release    # 在 0.0.0.0:8787 托管构建好的前端
cd frontend && npm run dev           # 热更新，/api 代理到 8787
```

测试：`backend/` 下 `cargo test`，`frontend/` 下 `npm test`。

Docker 构建镜像：

```bash
docker build -t drill .
```

在仓库根目录执行。