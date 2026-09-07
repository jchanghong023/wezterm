# WezTerm 启动失败问题复现与定位

结论先行，证据随后。

## 1. 为什么 `./wezterm start` 失败

**根因：当前 `DISPLAY` 指向的 X Server 没有 `XKEYBOARD (XKB)` 扩展，而 WezTerm 的 X11 后端把 XKB 作为强制依赖，缺失时会直接 panic。**

### Panic 现场

```text
ERROR env bootstrap > panic at xcb-1.7.1/src/ext.rs:306:13
mandatory extension XKEYBOARD is not present on this system

xcb::ext::cache_extensions_data
<xcb::base::Connection>::connect_with_xlib_display_and_extensions
<window::os::x11::connection::XConnection>::create_new
wezterm_gui::frontend::try_new
```

退出码为 `101`。

这是 Rust `xcb` crate `1.7.1` 在 `cache_extensions_data` 中查询 XKB 扩展时发现其不存在，随后直接 `panic!`。WezTerm 没有捕获该异常，因此进程终止。

### 证据

针对：

```text
DISPLAY=10.236.101.88:11.0
```

执行 `xdpyinfo` 得到：

```text
vendor string: Hummingbird - Open Text
vendor release: 138120
X11 version: 11.0
```

共约 22 个扩展，包括：

```text
RANDR
XFIXES
XInputExtension
RENDER
DAMAGE
MIT-SHM
SHAPE
SYNC
RECORD
XTEST
DOUBLE-BUFFER
SECURITY
X-Resource
XFree86-Bigfont
XFree86-VidModeExtension
BIG-REQUESTS
XC-APPGROUP
XC-MISC
MIT-SUNDRY-NONSTANDARD

HCL_CompressedImage
HCL_ThinPrint
HCL_ThinX
```

其中 `HCL` 为 Hummingbird 相关私有扩展。

但不存在：

```text
XKEYBOARD
```

执行：

```text
xdpyinfo | grep -i XKEYBOARD
```

结果为空。

### 补充澄清

直接执行：

```text
./bin/wezterm-gui
```

出现：

```text
libxcb-image.so.0 not found
```

是因为绕过了顶层包装脚本。

正常执行：

```text
./wezterm
```

时，脚本会正确设置：

```text
LD_LIBRARY_PATH=$HERE/lib
```

`ldd` 显示所有依赖库，包括：

```text
libxcb-image
libxcb-xkb
libssl.so.3
...
```

都能解析到 WezTerm 自带的 bundled library。

因此：

> **库依赖不是问题，失败纯粹发生在 X Server 连接阶段。**

---

## 2. 为什么其他 CentOS 7 能启动，这台机器有什么特别

CentOS 7 本身没有问题：

```text
glibc 2.17
```

满足当前 WezTerm 的运行基线，并且 WezTerm 自带依赖也能正常加载。

真正的差异在于：

> **不同机器的 `DISPLAY` 指向了不同的 X Server。**

| 项目              | 其他能启动的 CentOS 7                                                                                 | 本机                            |
| --------------- | ----------------------------------------------------------------------------------------------- | ----------------------------- |
| `DISPLAY`       | 通常为 `localhost:10`，即 SSH X11 转发到本地 Linux Xorg / Xming / MobaXterm / VcXsrv / Cygwin-X；或者本机 `:0` | `10.236.101.88:11.0`          |
| X Server Vendor | X.Org 或带 XKB 的第三方 X Server                                                                      | Hummingbird / OpenText Exceed |
| `XKEYBOARD` 扩展  | 有                                                                                               | 无                             |
| 结果              | WezTerm 正常启动                                                                                    | Panic                         |

### 本机的特殊之处

当前通过 SSH 登录到：

```text
10.239.91.203
```

但：

```text
DISPLAY=10.236.101.88:11.0
```

也就是说，当前 GUI 并不是标准 SSH X11 转发。

标准 SSH X11 转发一般类似：

```text
localhost:10.0
```

当前环境则把 `DISPLAY` 显式指向了第三台机器：

```text
10.236.101.88:11.0
```

对应 TCP 端口：

```text
6011
```

该端口实际处于开放状态。

这台 Hummingbird / OpenText Exceed X Server 当前没有提供 XKB 扩展。

而 WezTerm 使用的 `xcb` crate 将 XKB 列为 mandatory extension，在：

```text
xcb-1.7.1/src/ext.rs:306
```

处直接 `panic!`，不存在可选降级路径。

本机实际上安装了带 XKB 支持的本地 Xorg：

```text
xorg-x11-server-Xorg-1.20.4-22.el7_9
```

并且：

```text
strings /usr/bin/Xorg
```

可以看到：

```text
XKEYBOARD
```

但当前会话并没有使用本地 Xorg。

因此可以简化为：

> **不是 CentOS 7 的问题，而是当前使用的 Exceed X Server 缺少 XKB。其他机器能够启动，是因为它们连接的 X Server 支持 XKB。**

---

## 3. 系统详细信息及支持的 GUI 技术与版本

### 3.1 系统基线

| 项目     | 值                             |
| ------ | ----------------------------- |
| 发行版    | CentOS Linux 7 (Core)         |
| Kernel | `3.10.0-1160.95.1.el7.x86_64` |
| glibc  | `2.17`                        |
| GCC    | `4.8.2`                       |
| 架构     | `x86_64`                      |
| 主机     | `dgg4cd203-hs`                |

### 3.2 系统 GUI 库

系统目录：

```text
/usr/lib64
```

相关 RPM / Library 版本：

| 技术           | 版本                | 备注                                                                                                   |
| ------------ | ----------------- | ---------------------------------------------------------------------------------------------------- |
| Xorg Server  | `1.20.4-22.el7_9` | 本地二进制存在且包含 XKB，但当前 `DISPLAY` 未使用它                                                                    |
| Mesa         | `18.3.4-10.el7`   | 包含 `libGL` / `libEGL` / `libGLESv2` / `libgbm` / DRI drivers，例如 swrast、llvmpipe、Intel、AMDGPU、Nouveau |
| libdrm       | `2.4.97-2.el7`    | 包含 AMDGPU / Intel / Nouveau                                                                          |
| GTK 3        | `3.22.30-5.el7`   |                                                                                                      |
| GTK 2        | `2.24.31-1.el7`   |                                                                                                      |
| Qt 4         | `4.8.7`           | 无 Qt5                                                                                                |
| fontconfig   | `2.13.0-4.3.el7`  |                                                                                                      |
| FreeType     | `2.8-14.el7`      |                                                                                                      |
| libxkbcommon | `0.7.1-3.el7`     | 包含 `libxkbcommon-x11`                                                                                |
| xcb-util     | `0.4.0-2.el7`     | 系统未安装 `xcb-util-keysyms` / `xcb-util-wm` / `xcb-util-image`，但 WezTerm 自带                             |
| Wayland      | 未安装               | WezTerm 自带 `libwayland-client` / `libwayland-egl`                                                    |
| OpenSSL      | `1.0.2k-26.el7_9` | WezTerm 自带 OpenSSL 3.x                                                                               |

### 3.3 WezTerm 自带依赖

`lib/` 下包含：

```text
libssl / libcrypto 3
libxcb
libxcb-image
libxcb-shm
libxcb-util
libxcb-xkb
libX11-xcb
libwayland-client
libwayland-egl
libfontconfig
libfreetype
libpng16
libexpat
libffi
libgcc_s
libz
...
```

WezTerm 版本：

```text
wezterm 20260906-191018-a23a2614
```

### 3.4 当前实际生效的 X Server

```text
DISPLAY=10.236.101.88:11.0
```

| 项目          | 值                                                                            |
| ----------- | ---------------------------------------------------------------------------- |
| Vendor      | Hummingbird / OpenText Exceed                                                |
| Release     | `138120`                                                                     |
| X11 Version | `11.0`                                                                       |
| 扩展数量        | 约 22                                                                         |
| 关键扩展        | 有 RANDR / XFIXES / XInput / RENDER / DAMAGE / MIT-SHM / SHAPE / SYNC / XTEST |
| XKB         | **无 `XKEYBOARD`**                                                            |
| 私有扩展        | `HCL_CompressedImage`、`HCL_ThinPrint`、`HCL_ThinX`                            |

---

## 4. 可行的解决方向

按成本从低到高：

### 方案 1：更换 X Server

让 `DISPLAY` 指向支持 XKB 的 X Server，例如：

```text
本机 Xorg
Xming
MobaXterm
VcXsrv
Cygwin-X
```

如果使用本机 Xorg，还需要确保 X Server 实际启动，并且当前环境能够连接，例如配合 VNC。

### 方案 2：在 Exceed 侧启用 XKB

如果当前 Exceed 版本支持，可以检查配置中是否能够启用：

```text
XKEYBOARD / XKB
```

当前 `release 138120` 是否支持以及具体配置方法，需要进一步确认。

### 方案 3：Wayland

当前不可直接使用。

系统没有安装：

```text
wayland
wayland-protocols
```

也没有可用的 Wayland compositor。

WezTerm 虽然自带部分 Wayland client library，但这不足以提供完整的 Wayland 图形环境。

### 方案 4：关闭 WezTerm 的 XKB 检查

当前没有发现可以通过环境变量关闭 XKB 强制检查的方法。

该行为来自 `xcb` crate 的 extension 初始化逻辑，属于编译后的强制检查，并不是普通运行参数能够关闭的。

---

## 5. 最低成本验证方法

找一台明确支持 XKB 的 X Server，然后执行：

```text
DISPLAY=<支持XKB的X服务器>:0 ./wezterm start
```

如果 WezTerm 能正常启动，即可进一步反向坐实：

> **当前启动失败的根因就是 Exceed X Server 缺少 `XKEYBOARD (XKB)` 扩展。**
