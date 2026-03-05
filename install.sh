#!/bin/sh
# Octav CLI installer
# Usage: curl -sSf https://raw.githubusercontent.com/Octav-Labs/octav-cli/main/install.sh | sh

set -eu

REPO="Octav-Labs/octav-cli"
BINARY="octav"

main() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin) os="apple-darwin" ;;
        Linux)  os="unknown-linux-gnu" ;;
        *)
            echo "Error: unsupported OS: $os" >&2
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            echo "Error: unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac

    target="${arch}-${os}"

    echo "Detected platform: ${target}"

    # Get latest release tag
    version="$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"tag_name": *"//;s/".*//')"

    if [ -z "$version" ]; then
        echo "Error: could not determine latest release version" >&2
        exit 1
    fi

    echo "Latest version: ${version}"

    url="https://github.com/${REPO}/releases/download/${version}/${BINARY}-${target}.tar.gz"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    echo "Downloading ${url}..."
    curl -sSfL "$url" -o "${tmpdir}/octav.tar.gz"

    tar -xzf "${tmpdir}/octav.tar.gz" -C "$tmpdir"

    # Install to /usr/local/bin if writable, otherwise ~/.local/bin
    if [ -w /usr/local/bin ]; then
        install_dir="/usr/local/bin"
    elif command -v sudo >/dev/null 2>&1; then
        install_dir="/usr/local/bin"
        echo "Installing to ${install_dir} (requires sudo)..."
        sudo mv "${tmpdir}/${BINARY}" "${install_dir}/${BINARY}"
        sudo chmod +x "${install_dir}/${BINARY}"
        echo "Installed octav ${version} to ${install_dir}/${BINARY}"
        return
    else
        install_dir="${HOME}/.local/bin"
        mkdir -p "$install_dir"
    fi

    mv "${tmpdir}/${BINARY}" "${install_dir}/${BINARY}"
    chmod +x "${install_dir}/${BINARY}"

    echo "Installed octav ${version} to ${install_dir}/${BINARY}"

    # Check if install_dir is in PATH
    case ":${PATH}:" in
        *":${install_dir}:"*) ;;
        *)
            echo ""
            echo "NOTE: ${install_dir} is not in your PATH."
            echo "Add it with:  export PATH=\"${install_dir}:\$PATH\""
            ;;
    esac
}

main
