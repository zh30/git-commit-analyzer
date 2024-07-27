#!/bin/bash

# 定义下载URL
DOWNLOAD_URL="https://scripts.zhanghe.dev/git-ca"

# 定义安装目录
if [[ "$OSTYPE" == "linux-gnu"* || "$OSTYPE" == "darwin"* ]]; then
    INSTALL_DIR="$HOME/.local/bin"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    INSTALL_DIR="$USERPROFILE/AppData/Local/Programs/git-ca"
else
    echo "Unsupported operating system"
    exit 1
fi

# 创建安装目录（如果不存在）
mkdir -p "$INSTALL_DIR"

# 下载插件
if command -v curl &> /dev/null; then
    curl -L "$DOWNLOAD_URL" -o "$INSTALL_DIR/git-ca"
elif command -v wget &> /dev/null; then
    wget "$DOWNLOAD_URL" -O "$INSTALL_DIR/git-ca"
else
    echo "Neither curl nor wget is available. Please install one of them and try again."
    exit 1
fi

# 设置执行权限（仅针对Unix-like系统）
if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "win32" ]]; then
    chmod +x "$INSTALL_DIR/git-ca"
fi

# 添加到PATH（仅针对Unix-like系统，Windows用户需要手动添加）
if [[ "$OSTYPE" == "linux-gnu"* || "$OSTYPE" == "darwin"* ]]; then
    echo "export PATH=\$PATH:$INSTALL_DIR" >> "$HOME/.bashrc"
    echo "export PATH=\$PATH:$INSTALL_DIR" >> "$HOME/.zshrc"
    echo "Please restart your terminal or run 'source ~/.bashrc' (or ~/.zshrc) to update your PATH."
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    echo "Installation complete. Please add $INSTALL_DIR to your system PATH manually."
fi

echo "git-ca plugin installed successfully!"