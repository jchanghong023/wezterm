#!/usr/bin/env bash
# 打包 CentOS 7 自包含发布包: 把构建机上非系统目录的 ldd 闭包(ssl/crypto/xcb-image 等)
# 拷贝进 lib/, 顶层 wezterm 包装脚本导出 LD_LIBRARY_PATH 后再 exec bin/wezterm。
# 用法: bash ci/package-custom-centos7.sh [版本, 默认 1.3]
# 不依赖 patchelf/chrpath, 不修改 RPATH, 不安装系统包。
set -euo pipefail

VER="${1:-1.3}"
HERE="$(cd -- "$(dirname -- "$0")/.." && pwd)"
STAGE="$HERE/dist/wezterm-custom-$VER"
TARBALL="$HERE/dist/wezterm-custom-$VER-centos7-x86_64.tar.gz"
BINS=(wezterm wezterm-gui wezterm-mux-server)

for b in "${BINS[@]}"; do
  [[ -x "$HERE/target/release/$b" ]] || { echo "缺少 target/release/$b, 先跑 cargo build --release --locked" >&2; exit 1; }
done
command -v ldd >/dev/null || { echo "需要 ldd" >&2; exit 1; }

rm -rf "$STAGE"
mkdir -p "$STAGE/bin" "$STAGE/lib" "$HERE/dist"
for b in "${BINS[@]}"; do
  cp -a "$HERE/target/release/$b" "$STAGE/bin/$b"
done

# 收集三个二进制的 ldd 并集: 只捆非系统目录的库(/lib64、/usr/lib64 等不动),
# 构建机上即 /opt/micromamba/envs/dev/lib 闭包, 含 libssl.so.3、libcrypto.so.3、
# libxcb-image.so.0 及其传递依赖。
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
for b in "${BINS[@]}"; do
  ldd "$STAGE/bin/$b"
done | awk '{for (i=1;i<=NF;i++) if ($i ~ /^\//) print $i}' | sort -u > "$tmp"
while IFS= read -r src; do
  case "$src" in
    /lib/*|/lib64/*|/usr/lib/*|/usr/lib64/*) continue ;;
  esac
  cp -L "$src" "$STAGE/lib/$(basename "$src")"
done < "$tmp"
rm -f "$tmp"

for need in libssl.so.3 libcrypto.so.3 libxcb-image.so.0; do
  [[ -f "$STAGE/lib/$need" ]] || { echo "断言失败: lib/$need 缺失" >&2; exit 1; }
done

cat > "$STAGE/wezterm" <<'EOF'
#!/usr/bin/env bash
HERE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
export LD_LIBRARY_PATH="$HERE/lib:${LD_LIBRARY_PATH:-}"
exec "$HERE/bin/wezterm" "$@"
EOF
chmod +x "$STAGE/wezterm"

cp -a "$HERE/assets/wezterm.desktop" "$STAGE/wezterm.desktop"

cat > "$STAGE/安装说明.txt" <<'EOF'
定制版 WezTerm 自包含包(CentOS 7 x86_64, glibc 2.17 基线)

目录:
  bin/  wezterm、wezterm-gui、wezterm-mux-server
  lib/  随包依赖(含 libssl.so.3、libcrypto.so.3、libxcb-image.so.0 等),
        与系统 /usr/lib64 同名库不冲突
  wezterm  包装脚本, 自动把 lib/ 加入 LD_LIBRARY_PATH 后启动 bin/wezterm
  wezterm.desktop  桌面项

运行:
  ./wezterm --version
  ./wezterm start
  ./wezterm connect --help

说明:
  不要直接跑 bin/ 下的二进制, 它们依赖 lib/, 必须经顶层 ./wezterm 启动,
  否则会报 libssl.so.3 / libcrypto.so.3 / libxcb-image.so.0 找不到。
  GUI 另需 X11 显示、字体与系统 libm/libc 等, 缺失时用
  LD_LIBRARY_PATH=$PWD/lib ldd bin/wezterm-gui 查 not found 项。
EOF

# 自检: 经 lib/ 解析后三个二进制都不许有 not found
for b in "${BINS[@]}"; do
  if LD_LIBRARY_PATH="$STAGE/lib" ldd "$STAGE/bin/$b" | grep -q "not found"; then
    echo "自检失败: $b 仍有 not found" >&2
    LD_LIBRARY_PATH="$STAGE/lib" ldd "$STAGE/bin/$b" | grep "not found" >&2 || true
    exit 1
  fi
done

tar -czf "$TARBALL" -C "$HERE/dist" "wezterm-custom-$VER"
sha256sum "$TARBALL" | tee "$TARBALL.sha256"
echo "OK: $STAGE"
echo "OK: $TARBALL"
