#!/usr/bin/env bash
# install.sh — skm 安装脚本
# 用法：curl -fsSL https://raw.githubusercontent.com/mocikadev/mocika-skills-cli/main/install.sh | bash
# 或：   SKM_VERSION=v0.2.0 bash install.sh

set -euo pipefail

REPO="mocikadev/mocika-skills-cli"
BINARY="skm"
INSTALL_DIR="${SKM_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${SKM_VERSION:-latest}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
RESET='\033[0m'

info()  { printf "${BOLD}info${RESET}  %s\n" "$*"; }
ok()    { printf "${GREEN}ok${RESET}    %s\n" "$*"; }
warn()  { printf "${YELLOW}warn${RESET}  %s\n" "$*"; }
die()   { printf "${RED}error${RESET} %s\n" "$*" >&2; exit 1; }

detect_target() {
  local os arch
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Linux)
      case "$arch" in
        x86_64)          echo "x86_64-unknown-linux-musl" ;;
        aarch64 | arm64) echo "aarch64-unknown-linux-musl" ;;
        *)               die "不支持的架构：$arch（Linux）" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64)          echo "x86_64-apple-darwin" ;;
        arm64)           echo "aarch64-apple-darwin" ;;
        *)               die "不支持的架构：$arch（macOS）" ;;
      esac
      ;;
    *)
      die "不支持的操作系统：$os（仅支持 Linux / macOS）"
      ;;
  esac
}

download() {
  local url="$1" dest="$2"
  if command -v curl > /dev/null 2>&1; then
    curl -fsSL --retry 3 --retry-delay 2 "$url" -o "$dest"
  elif command -v wget > /dev/null 2>&1; then
    wget -q --tries=3 "$url" -O "$dest"
  else
    die "需要 curl 或 wget 之一，但均未找到。"
  fi
}

resolve_url() {
  local target="$1"
  if [ "$VERSION" = "latest" ]; then
    echo "https://github.com/$REPO/releases/latest/download/$BINARY-$target"
  else
    echo "https://github.com/$REPO/releases/download/$VERSION/$BINARY-$target"
  fi
}

main() {
  printf "\n${BOLD}安装 skm — AI Agent 技能包管理器${RESET}\n\n"

  local target url tmp
  target=$(detect_target)
  url=$(resolve_url "$target")

  info "平台    ：$target"
  info "版本    ：$VERSION"
  info "安装目录：$INSTALL_DIR"
  printf "\n"

  mkdir -p "$INSTALL_DIR"

  info "下载中..."
  tmp=$(mktemp)
  # shellcheck disable=SC2064
  trap "rm -f '$tmp'" EXIT

  download "$url" "$tmp" || die "下载失败，请检查网络或版本号是否正确。"

  chmod +x "$tmp"
  mv "$tmp" "$INSTALL_DIR/$BINARY"

  ok "已安装：$INSTALL_DIR/$BINARY"

  if ! echo ":${PATH}:" | grep -qF ":${INSTALL_DIR}:"; then
    printf "\n"
    warn "$INSTALL_DIR 不在 \$PATH 中，请将以下内容加入 ~/.bashrc 或 ~/.zshrc："
    printf "\n  ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${RESET}\n"
  fi

  printf "\n${GREEN}${BOLD}完成！${RESET} 运行 ${BOLD}skm --help${RESET} 开始使用。\n\n"
}

main "$@"
