#!/usr/bin/env bash
set -euo pipefail

REPO="tftio-stuff/tools"
INSTALL_DIR="$HOME/.local/bin"
ALL_TOOLS=(prompter unvenv asana-cli todoer gator silent-critic bce)

usage() {
  cat <<EOF
Install tftio tools from GitHub releases.

Usage: install.sh [OPTIONS] [TOOL...]

Options:
  -d, --dir DIR    Install directory (default: ~/.local/bin)
  -l, --list       List available tools and their latest versions
  -h, --help       Show this help

Tools: ${ALL_TOOLS[*]}

If no tools are specified, all are installed.

Examples:
  install.sh                      # install all tools
  install.sh prompter todoer      # install specific tools
  install.sh -d /usr/local/bin    # install to custom directory
  curl -fsSL https://raw.githubusercontent.com/$REPO/main/install.sh | bash
EOF
}

die() { printf 'Error: %s\n' "$1" >&2; exit 1; }

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *)      die "Unsupported OS: $os" ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)             die "Unsupported architecture: $arch" ;;
  esac

  # No x86_64-apple-darwin target in CI
  if [ "$arch" = "x86_64" ] && [ "$os" = "apple-darwin" ]; then
    die "No pre-built binary for x86_64-apple-darwin. Install via: cargo install tftio-<tool>"
  fi

  echo "${arch}-${os}"
}

tag_prefix_for() {
  local tool="$1"
  case "$tool" in
    bce) echo "bsky-comment-extractor" ;;
    *)   echo "$tool" ;;
  esac
}

latest_tag_for() {
  local tool="$1" prefix
  prefix="$(tag_prefix_for "$tool")"
  gh release list --repo "$REPO" --limit 50 --json tagName --jq \
    "[.[] | select(.tagName | startswith(\"${prefix}-v\"))] | first | .tagName" 2>/dev/null
}

install_tool() {
  local tool="$1" target="$2" tag version asset url tmp

  local prefix
  prefix="$(tag_prefix_for "$tool")"
  tag="$(latest_tag_for "$tool")"
  if [ -z "$tag" ] || [ "$tag" = "null" ]; then
    printf '  %-16s skipped (no release found)\n' "$tool"
    return 0
  fi

  version="${tag#"${prefix}-"}"
  asset="${tool}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

  tmp="$(mktemp -d)"
  trap "rm -rf '$tmp'" RETURN

  if ! curl -fsSL -o "${tmp}/${asset}" "$url" 2>/dev/null; then
    printf '  %-16s skipped (no binary for %s)\n' "$tool" "$target"
    return 0
  fi

  tar -xzf "${tmp}/${asset}" -C "$tmp"

  if [ ! -f "${tmp}/${tool}" ]; then
    printf '  %-16s skipped (binary not found in archive)\n' "$tool"
    return 0
  fi

  install -m 755 "${tmp}/${tool}" "${INSTALL_DIR}/${tool}"
  printf '  %-16s %s\n' "$tool" "$version"
}

list_tools() {
  local target
  target="$(detect_target)"
  printf 'Available tools (target: %s):\n\n' "$target"
  for tool in "${ALL_TOOLS[@]}"; do
    local tag
    tag="$(latest_tag_for "$tool")"
    local prefix
    prefix="$(tag_prefix_for "$tool")"
    if [ -z "$tag" ] || [ "$tag" = "null" ]; then
      printf '  %-16s (no release)\n' "$tool"
    else
      printf '  %-16s %s\n' "$tool" "${tag#"${prefix}-"}"
    fi
  done
}

main() {
  local tools=() list_mode=false

  while [ $# -gt 0 ]; do
    case "$1" in
      -d|--dir)  INSTALL_DIR="$2"; shift 2 ;;
      -l|--list) list_mode=true; shift ;;
      -h|--help) usage; exit 0 ;;
      -*)        die "Unknown option: $1" ;;
      *)         tools+=("$1"); shift ;;
    esac
  done

  if ! command -v gh >/dev/null 2>&1; then
    die "gh (GitHub CLI) is required. Install: https://cli.github.com"
  fi

  if $list_mode; then
    list_tools
    exit 0
  fi

  if [ ${#tools[@]} -eq 0 ]; then
    tools=("${ALL_TOOLS[@]}")
  fi

  # Validate tool names
  for tool in "${tools[@]}"; do
    local valid=false
    for known in "${ALL_TOOLS[@]}"; do
      if [ "$tool" = "$known" ]; then valid=true; break; fi
    done
    if ! $valid; then
      die "Unknown tool: $tool (available: ${ALL_TOOLS[*]})"
    fi
  done

  local target
  target="$(detect_target)"

  mkdir -p "$INSTALL_DIR"

  printf 'Installing to %s (target: %s)\n\n' "$INSTALL_DIR" "$target"

  for tool in "${tools[@]}"; do
    install_tool "$tool" "$target"
  done

  printf '\nDone.'

  if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    printf ' Add to PATH: export PATH="%s:$PATH"' "$INSTALL_DIR"
  fi

  printf '\n'
}

main "$@"
