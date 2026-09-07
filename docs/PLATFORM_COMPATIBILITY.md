# 跨平台兼容性与验证基线

本仓库默认需要同时考虑以下平台：

1. Windows；
2. 最新 Linux；
3. 公司内部 CentOS 7 服务器；
4. 外部普通 CentOS 7 服务器。

核心原则：**当前环境能测什么就测什么，不能测的平台只做静态检查，不允许把静态检查说成动态验证。**

---

## 1. 当前环境自判

Agent 在执行启动 DR、dry-run、启动验证、打包验证前，必须先判断当前环境。

Linux / WSL2 / CentOS 环境执行：

```bash
echo "===== OS ====="
uname -a
cat /etc/os-release 2>/dev/null || true

echo "===== libc ====="
getconf GNU_LIBC_VERSION 2>/dev/null || true
ldd --version 2>/dev/null | head -1 || true

echo "===== display ====="
echo "DISPLAY=${DISPLAY:-}"
echo "WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-}"

if command -v xdpyinfo >/dev/null 2>&1 && [ -n "${DISPLAY:-}" ]; then
    xdpyinfo | sed -n '1,80p'
    xdpyinfo | grep -i XKEYBOARD || echo "NO_XKEYBOARD"
fi
```

Windows 环境至少确认：

```powershell
$PSVersionTable
[System.Environment]::OSVersion.VersionString
$env:PATH
```

---

## 2. 动态验证与静态检查边界

| 当前环境         | 可以动态验证                        | 不能冒充验证                          |
| ------------ | ----------------------------- | ------------------------------- |
| Windows      | Windows 启动、路径、权限、打包           | Linux / CentOS7 启动              |
| 最新 Linux     | 新 Linux 启动、基础依赖、GUI 基础功能      | CentOS7 ABI、公司内部 Exceed/X11     |
| 外部普通 CentOS7 | glibc 2.17、ldd、启动脚本、基础运行      | 公司内部 Exceed/X11                 |
| WSL2 CentOS  | CentOS 用户态、脚本、部分 ABI/依赖       | 真实服务器 GUI、X11、驱动、system service |
| 公司内部 CentOS7 | 内部服务器真实运行、DISPLAY、X Server 扩展 | 外部普通 CentOS7 的完整代表性             |

最终报告必须区分：

```text
已动态验证：
已静态检查：
未在该平台验证：
已知风险：
```

禁止写法：

```text
已支持所有 Linux
已验证 CentOS7
跨平台完成
```

除非四类平台均已完成对应动态验证。

---

## 3. 公司内部 CentOS7 基线

已确认的公司内部服务器基线：

| 项目     | 值                             |
| ------ | ----------------------------- |
| 发行版    | CentOS Linux 7 Core           |
| Kernel | `3.10.0-1160.95.1.el7.x86_64` |
| glibc  | `2.17`                        |
| GCC    | `4.8.2`                       |
| 架构     | `x86_64`                      |

GUI / X11 重点限制：

| 项目             | 值                             |
| -------------- | ----------------------------- |
| `DISPLAY` 示例   | `10.236.101.88:11.0`          |
| X Server       | Hummingbird / OpenText Exceed |
| Vendor release | `138120`                      |
| X11 version    | `11.0`                        |
| 关键限制           | 无 `XKEYBOARD` / XKB 扩展        |

重点结论：

* 公司内部问题不一定是 CentOS7 本身的问题。
* GUI 程序真正连接的是 `DISPLAY` 指向的 X Server。
* 即使服务器本地安装了 Xorg，也不代表当前会话正在使用本地 Xorg。
* 对公司内部 GUI 场景，必须检查 `xdpyinfo`，尤其是 `XKEYBOARD`。

---

## 4. 公司内部 CentOS7 与外部普通 CentOS7 的区别

| 项目        | 公司内部 CentOS7             | 外部普通 CentOS7                        |
| --------- | ------------------------ | ----------------------------------- |
| OS 基线     | CentOS7 / glibc 2.17     | CentOS7 / glibc 2.17                |
| 权限        | 不应假设有 root               | 可能有 root，也可能没有                      |
| 安装依赖      | 不应依赖现场 `yum install`     | 可以作为辅助方案                            |
| GUI       | 可能通过 Exceed 等远端 X Server | 常见为 Xorg、VNC、Xming、VcXsrv、MobaXterm |
| `DISPLAY` | 可能指向第三台 X Server         | 常见 `:0` 或 `localhost:10.0`          |
| XKB       | 可能缺失                     | 通常存在，但不能假设                          |
| 验证重点      | X Server 扩展、无 root、路径、旧库 | glibc 2.17、动态库、启动脚本                 |

结论：

```text
外部 CentOS7 启动成功，不等于公司内部 CentOS7 启动成功。
公司内部 CentOS7 失败，也不一定代表普通 CentOS7 失败。
```

---

## 5. CentOS7 与 Ubuntu 24.04 的区别

CentOS 7 已在 2024-06-30 EOL；Ubuntu 24.04 LTS 是现代 Linux 基线，官方 release notes 中包含 Linux 6.8、glibc 2.39、GCC 14、Python 3.12 等新版本。([红帽][1]) ([Ubuntu Community Hub][2])

| 项目      | CentOS7       | Ubuntu 24.04                |
| ------- | ------------- | --------------------------- |
| glibc   | `2.17`        | `2.39`                      |
| Kernel  | 常见 `3.10.x`   | 6.8 系列                      |
| GCC     | 常见 `4.8.x`    | 14 系列                       |
| Python  | 系统 Python 很旧  | 默认 Python 3.12              |
| OpenSSL | 常见 1.0.2k     | OpenSSL 3 系列                |
| GUI 栈   | 老 X11 为主      | X11 / Wayland / XWayland 并存 |
| 构建产物    | 可作为旧 Linux 目标 | 不能直接证明兼容 CentOS7            |

规则：

* Ubuntu 24.04 上编译成功，不能证明支持 CentOS7。
* 面向 CentOS7 的 Linux 二进制，应以 glibc 2.17 为运行基线。
* Rust / C / C++ 项目必须检查最终 ELF 的动态库和符号版本。

---

## 6. Rust GUI 技术选择

### 默认选择：egui / eframe

`egui` 是 pure Rust immediate-mode GUI；`eframe` 是官方应用框架，README 明确支持 Web、Linux、Mac、Windows、Android。([GitHub][3])

适合：

```text
内部工具
配置界面
调试界面
启动器
日志查看器
轻量桌面程序
```

典型例子：

```text
AgentDock 会话列表
环境探测工具
TOML/JSON 配置编辑器
日志 tail + 关键字过滤工具
```

### 复杂状态型应用备选：iced

`iced` README 定位为 Rust 跨平台 GUI，支持 Windows、macOS、Linux、Web，并采用受 Elm 启发的 State / Message / View / Update 模型；它还强调 async actions。([GitHub][4])

只有满足以下情况时才考虑 iced：

```text
多页面长期维护
状态流转复杂
后台异步任务多
需要清晰 Message / Update / View 架构
未来会发展成较完整桌面应用
```

典型例子：

```text
多步骤安装器
复杂 Agent 控制台
项目管理器
任务队列 + 后台扫描 + 进度展示工具
```

统一限制：

```text
egui/eframe 或 iced 支持 Windows/Linux，不等于天然支持公司内部 CentOS7 + Exceed。
只要 Linux GUI 底层涉及 X11、winit、xkbcommon、OpenGL、EGL，都必须专项验证公司内部 DISPLAY 和 XKEYBOARD 场景。
```

---

## 7. Rust 终端技术选择

### 完整终端应用：参考 WezTerm

WezTerm README 定位为 GPU-accelerated cross-platform terminal emulator and multiplexer，并且由 Rust 实现。([GitHub][5])

适合参考：

```text
完整终端窗口
分屏
mux
PTY 管理
键盘输入
字体渲染
终端渲染
跨平台终端行为
```

### 嵌入终端核心：wezterm-term + portable-pty

`wezterm-term` 是 WezTerm 的虚拟终端核心，提供 escape sequence parsing、screen cells、scrollback、sixel、iTerm2 image、OSC 8 hyperlinks 等能力，但不提供 GUI，也不直接管理 PTY。([GitHub][6])

`portable_pty` 提供跨平台 PTY API，并允许在运行时选择不同系统实现。([Docs.rs][7])

推荐拆分：

```text
终端状态机 / escape sequence / scrollback：wezterm-term
跨平台 PTY：portable-pty
完整终端应用参考：WezTerm
```

特别规则：

```text
WezTerm 是优秀参考，但不能假设所有 X Server 都有 XKEYBOARD。
公司内部 CentOS7 + Exceed 场景下，XKB 必须 optional + fallback，不能作为 mandatory extension。
```

---

## 8. 动态库与启动脚本检查

Linux 发布包必须检查所有真实 ELF 文件，不能只检查顶层脚本。

```bash
for f in bin/*; do
    [ -x "$f" ] || continue
    echo "===== $f ====="
    LD_LIBRARY_PATH="$PWD/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
        ldd "$f" | grep -E "not found|=>"
done
```

要求：

```text
不能有 not found
不能假设用户有 root
不能要求用户手工配置复杂 LD_LIBRARY_PATH
启动脚本必须支持路径包含空格
```

推荐启动脚本：

```bash
#!/usr/bin/env bash
set -e

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export LD_LIBRARY_PATH="$HERE/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

exec "$HERE/bin/app" "$@"
```

---

## 9. glibc / GLIBCXX 检查

面向 CentOS7 的二进制必须检查符号版本。

```bash
for f in bin/*; do
    [ -x "$f" ] || continue
    echo "===== $f ====="
    objdump -T "$f" | grep -o 'GLIBC_[0-9.]*' | sort -Vu | tail -1
done
```

C++ 或间接依赖 C++ 的项目还要检查：

```bash
for f in bin/*; do
    [ -x "$f" ] || continue
    echo "===== $f ====="
    objdump -T "$f" | grep -o 'GLIBCXX_[0-9.]*' | sort -Vu | tail -1
done
```

规则：

```text
CentOS7 目标产物不能依赖高于目标系统可用版本的 glibc 符号。
如果符号版本过高，说明构建环境太新，不能直接发布给 CentOS7。
```

---

## 10. GUI / X11 检查规则

GUI 程序不能把以下能力未经验证设为强制依赖：

```text
XKEYBOARD / XKB
RANDR
RENDER
XFIXES
XInputExtension
MIT-SHM
DAMAGE
Present
DRI2 / DRI3
GLX
Wayland
```

规则：

```text
可选扩展缺失时应降级。
必要扩展缺失时应清晰报错。
不允许 panic。
不允许静默退出。
不允许把现代 Xorg 能力假设为所有 X Server 都有。
```

公司内部重点：

```bash
xdpyinfo | grep -i XKEYBOARD || echo "NO_XKEYBOARD"
```

若输出 `NO_XKEYBOARD`，程序仍应做到：

```text
能启动，或给出清晰错误；
不能在 XCB/X11 初始化阶段 panic；
键盘路径应支持 fallback。
```

---

## 11. Agent 修改前检查清单

只要改动涉及以下内容，必须执行本文件相关检查：

```text
构建脚本
发布打包
动态库
启动脚本
GUI / X11 / Wayland
键盘输入
IME
剪贴板
OpenGL / EGL / GLX
PTY / 终端
Rust / C / C++ native 二进制
路径处理
```

提交或总结时必须说明：

```text
当前环境是什么
动态验证了哪些平台
静态检查了哪些平台
哪些平台未验证
是否影响 Windows
是否影响最新 Linux
是否影响外部 CentOS7
是否影响公司内部 CentOS7 + Exceed
是否新增动态库依赖
是否依赖 root / yum / apt / systemd / Wayland / XKB
```

---

## 12. 最低验收标准

如果声明“支持四类平台”，至少满足：

```text
Windows 已动态启动验证
最新 Linux 已动态启动验证
外部普通 CentOS7 已动态启动验证
公司内部 CentOS7 已动态启动验证
CentOS7 glibc / ldd / 符号检查通过
公司内部 DISPLAY / XKEYBOARD 场景检查通过
无 root 权限可运行
路径包含空格时可运行
错误信息可定位
```

如果只完成部分验证，必须写：

```text
已在当前环境动态验证。
其他平台仅完成静态检查。
公司内部 CentOS7 + Exceed 场景尚未动态验证。
```

[1]: https://www.redhat.com/en/topics/linux/centos-linux-eol "What to know about CentOS Linux EOL"
[2]: https://discourse.ubuntu.com/t/ubuntu-24-04-lts-noble-numbat-release-notes/39890?utm_source=chatgpt.com "Ubuntu 24.04 LTS (Noble Numbat) Release Notes"
[3]: https://github.com/emilk/egui "GitHub - emilk/egui: egui: an easy-to-use immediate mode GUI in Rust that runs on both web and native · GitHub"
[4]: https://github.com/iced-rs/iced "GitHub - iced-rs/iced: A cross-platform GUI library for Rust, inspired by Elm · GitHub"
[5]: https://github.com/wezterm/wezterm "GitHub - wezterm/wezterm: A GPU-accelerated cross-platform terminal emulator and multiplexer written by @wez and implemented in Rust · GitHub"
[6]: https://github.com/wez/wezterm/blob/main/term/README.md "wezterm/term/README.md at main · wezterm/wezterm · GitHub"
[7]: https://docs.rs/portable-pty "portable_pty - Rust"
