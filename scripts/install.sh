#!/usr/bin/env bash
# Install qiui binary + Codex skill from GitHub Releases (CntierTeam/QIUI).
# Also supports --from-source when run inside a git checkout.
set -euo pipefail

REPO="${QIUI_REPO:-CntierTeam/QIUI}"
VERSION="${QIUI_VERSION:-latest}"
PREFIX="${QIUI_PREFIX:-${HOME}/.local}"
BIN_DIR="${PREFIX}/bin"
CODEX_HOME="${CODEX_HOME:-${HOME}/.codex}"
SKILLS_DIR="${CODEX_HOME}/skills"
SKILL_DST="${SKILLS_DIR}/qiui"
API="https://api.github.com/repos/${REPO}"

INSTALL_BIN=1
INSTALL_SKILL=1
FROM_SOURCE=0
FORCE=0
UNINSTALL=0

usage() {
  cat <<EOF
Usage: install.sh [options]

Installs the qiui binary and/or Codex skill from GitHub Releases.

Options:
  --bin-only         Install binary only
  --skill-only       Install Codex skill only
  --from-source      Use local repo (binary via cargo, skill via copy/symlink)
  --symlink-skill    With --from-source: symlink skill (default: copy)
  --prefix DIR       Binary prefix (default: ~/.local) → DIR/bin/qiui
  --version VER      Release tag (default: latest), e.g. v0.1.0
  --repo OWNER/NAME  GitHub repo (default: ${REPO})
  --force            Replace existing install
  --uninstall        Remove binary + skill installed by this script
  -h, --help         Show help

Environment:
  QIUI_REPO / QIUI_VERSION / QIUI_PREFIX / CODEX_HOME / GITHUB_TOKEN

Examples:
  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | bash
  QIUI_VERSION=v0.1.0 ./scripts/install.sh --force
  ./scripts/install.sh --from-source --symlink-skill
EOF
}

SYMLINK_SKILL=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin-only) INSTALL_SKILL=0; shift ;;
    --skill-only) INSTALL_BIN=0; shift ;;
    --from-source) FROM_SOURCE=1; shift ;;
    --symlink-skill) SYMLINK_SKILL=1; shift ;;
    --prefix) PREFIX="$2"; BIN_DIR="${PREFIX}/bin"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --repo) REPO="$2"; API="https://api.github.com/repos/${REPO}"; shift 2 ;;
    --force|-f) FORCE=1; shift ;;
    --uninstall) UNINSTALL=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: missing required command: $1" >&2
    exit 1
  }
}

CURL_OPTS=(--fail --show-error --location --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 20)

http_get() {
  local url="$1" out="$2"
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    curl "${CURL_OPTS[@]}" -H "Authorization: Bearer ${GITHUB_TOKEN}" -H "Accept: application/octet-stream" "$url" -o "$out"
  else
    curl "${CURL_OPTS[@]}" -H "Accept: application/octet-stream" "$url" -o "$out"
  fi
}

http_json() {
  local url="$1"
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    curl "${CURL_OPTS[@]}" -H "Authorization: Bearer ${GITHUB_TOKEN}" -H "Accept: application/vnd.github+json" "$url"
  else
    curl "${CURL_OPTS[@]}" -H "Accept: application/vnd.github+json" "$url"
  fi
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "${os}/${arch}" in
    Linux/x86_64|Linux/amd64) echo "x86_64-unknown-linux-gnu" ;;
    Linux/aarch64|Linux/arm64) echo "aarch64-unknown-linux-gnu" ;;
    Darwin/arm64|Darwin/aarch64) echo "aarch64-apple-darwin" ;;
    Darwin/x86_64) echo "x86_64-apple-darwin" ;;
    MINGW*|MSYS*|CYGWIN*)
      case "${arch}" in
        x86_64|amd64) echo "x86_64-pc-windows-msvc" ;;
        *)
          echo "error: unsupported Windows arch: ${arch}" >&2
          exit 1
          ;;
      esac
      ;;
    *)
      # Git Bash / MSYS report like MINGW64_NT-10.0/x86_64
      case "${os}" in
        MINGW*|MSYS*|CYGWIN*)
          case "${arch}" in
            x86_64|amd64) echo "x86_64-pc-windows-msvc" ;;
            *)
              echo "error: unsupported Windows arch: ${arch}" >&2
              exit 1
              ;;
          esac
          ;;
        *)
          echo "error: unsupported platform: ${os}/${arch}" >&2
          echo "hint: on Windows use scripts/install.ps1" >&2
          exit 1
          ;;
      esac
      ;;
  esac
}

bin_name() {
  case "$(detect_target)" in
    *-windows-*) echo "qiui.exe" ;;
    *) echo "qiui" ;;
  esac
}

uninstall_all() {
  local name
  name="$(bin_name 2>/dev/null || echo qiui)"
  if [[ -e "${BIN_DIR}/${name}" || -L "${BIN_DIR}/${name}" ]]; then
    rm -f "${BIN_DIR}/${name}"
    echo "removed ${BIN_DIR}/${name}"
  fi
  # also remove unix name if present on mixed installs
  if [[ "${name}" != "qiui" && ( -e "${BIN_DIR}/qiui" || -L "${BIN_DIR}/qiui" ) ]]; then
    rm -f "${BIN_DIR}/qiui"
    echo "removed ${BIN_DIR}/qiui"
  fi
  if [[ -e "${SKILL_DST}" || -L "${SKILL_DST}" ]]; then
    rm -rf "${SKILL_DST}"
    echo "removed ${SKILL_DST}"
  fi
}

if [[ "${UNINSTALL}" -eq 1 ]]; then
  uninstall_all
  exit 0
fi

need_cmd curl
need_cmd uname

TMP="$(mktemp -d)"
cleanup() { rm -rf "${TMP}"; }
trap cleanup EXIT

ensure_replace() {
  local path="$1"
  if [[ -e "${path}" || -L "${path}" ]]; then
    if [[ "${FORCE}" -ne 1 ]]; then
      echo "error: already exists: ${path}" >&2
      echo "hint: re-run with --force" >&2
      exit 1
    fi
    rm -rf "${path}"
  fi
}

install_skill_dir() {
  local src="$1"
  mkdir -p "${SKILLS_DIR}"
  ensure_replace "${SKILL_DST}"
  if [[ "${SYMLINK_SKILL}" -eq 1 ]]; then
    ln -s "$(cd "${src}" && pwd)" "${SKILL_DST}"
    echo "skill (symlink): ${SKILL_DST} -> ${src}"
  else
    mkdir -p "${SKILL_DST}"
    cp -a "${src}/." "${SKILL_DST}/"
    echo "skill (copy): ${SKILL_DST}"
  fi
  grep -q '^name: qiui$' "${SKILL_DST}/SKILL.md"
}

install_from_source() {
  local script_dir repo_root built name
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  repo_root="$(cd "${script_dir}/.." && pwd)"
  name="$(bin_name)"

  if [[ "${INSTALL_BIN}" -eq 1 ]]; then
    need_cmd cargo
    (cd "${repo_root}" && cargo build --release)
    if [[ -f "${repo_root}/target/release/qiui.exe" ]]; then
      built="${repo_root}/target/release/qiui.exe"
      name="qiui.exe"
    else
      built="${repo_root}/target/release/qiui"
      name="qiui"
    fi
    mkdir -p "${BIN_DIR}"
    ensure_replace "${BIN_DIR}/${name}"
    install -m 0755 "${built}" "${BIN_DIR}/${name}"
    echo "binary: ${BIN_DIR}/${name}"
  fi

  if [[ "${INSTALL_SKILL}" -eq 1 ]]; then
    install_skill_dir "${repo_root}/.codex/skills/qiui"
  fi
}

asset_url_by_name() {
  local json="$1" name="$2"
  # Prefer browser_download_url (works without auth for public repos)
  python3 - "$json" "$name" <<'PY' 2>/dev/null || true
import json, sys
data = json.load(open(sys.argv[1]))
want = sys.argv[2]
for a in data.get("assets", []):
    if a.get("name") == want:
        print(a.get("browser_download_url") or "")
        break
PY
}

fetch_release_json() {
  local url
  if [[ "${VERSION}" == "latest" ]]; then
    url="${API}/releases/latest"
  else
    url="${API}/releases/tags/${VERSION}"
  fi
  http_json "${url}" > "${TMP}/release.json"
}

install_from_release() {
  need_cmd python3
  fetch_release_json
  local tag
  tag="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["tag_name"])' "${TMP}/release.json")"
  echo "release: ${REPO}@${tag}"

  if [[ "${INSTALL_BIN}" -eq 1 ]]; then
    local target asset url extracted_bin name
    target="$(detect_target)"
    name="$(bin_name)"
    if [[ "${target}" == *-windows-* ]]; then
      asset="qiui-${target}.zip"
      need_cmd unzip
    else
      asset="qiui-${target}.tar.gz"
      need_cmd tar
    fi
    url="$(asset_url_by_name "${TMP}/release.json" "${asset}")"
    if [[ -z "${url}" ]]; then
      echo "error: asset not found in release: ${asset}" >&2
      echo "available assets:" >&2
      python3 -c 'import json,sys; [print(" -", a["name"]) for a in json.load(open(sys.argv[1])).get("assets",[])]' "${TMP}/release.json" >&2
      exit 1
    fi
    echo "downloading ${asset}"
    http_get "${url}" "${TMP}/${asset}"
    if [[ "${asset}" == *.zip ]]; then
      unzip -q "${TMP}/${asset}" -d "${TMP}"
    else
      tar -C "${TMP}" -xzf "${TMP}/${asset}"
    fi
    extracted_bin="$(find "${TMP}" -type f \( -name qiui -o -name qiui.exe \) | head -n1)"
    [[ -n "${extracted_bin}" ]] || { echo "error: qiui binary missing in archive" >&2; exit 1; }
    name="$(basename "${extracted_bin}")"
    mkdir -p "${BIN_DIR}"
    ensure_replace "${BIN_DIR}/${name}"
    install -m 0755 "${extracted_bin}" "${BIN_DIR}/${name}"
    echo "binary: ${BIN_DIR}/${name}"
  fi

  if [[ "${INSTALL_SKILL}" -eq 1 ]]; then
    local skill_asset skill_url
    skill_asset="qiui-skill.tar.gz"
    skill_url="$(asset_url_by_name "${TMP}/release.json" "${skill_asset}")"
    if [[ -z "${skill_url}" ]]; then
      echo "error: asset not found in release: ${skill_asset}" >&2
      exit 1
    fi
    need_cmd tar
    echo "downloading ${skill_asset}"
    http_get "${skill_url}" "${TMP}/${skill_asset}"
    mkdir -p "${TMP}/skill"
    tar -C "${TMP}/skill" -xzf "${TMP}/${skill_asset}"
    install_skill_dir "${TMP}/skill/qiui"
  fi
}

if [[ "${FROM_SOURCE}" -eq 1 ]]; then
  install_from_source
else
  install_from_release
fi

echo
echo "Done."
if [[ "${INSTALL_BIN}" -eq 1 ]]; then
  case ":${PATH}:" in
    *":${BIN_DIR}:"*) ;;
    *)
      echo "note: add to PATH → export PATH=\"${BIN_DIR}:\$PATH\""
      ;;
  esac
  echo "try: qiui profiles"
fi
if [[ "${INSTALL_SKILL}" -eq 1 ]]; then
  echo "Codex skill: \$qiui (restart Codex / new session if already running)"
  echo "Codex home: ${CODEX_HOME}"
fi
