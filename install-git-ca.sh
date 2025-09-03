#!/bin/bash

# Git Commit Analyzer 安装脚本
# 此脚本将自动安装 git-ca 插件及其所有依赖

set -e  # 遇到错误时退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查命令是否存在
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 检测操作系统
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [[ -f /etc/debian_version ]] || [[ -f /etc/debian_release ]]; then
            echo "debian"
        elif [[ -f /etc/redhat-release ]] || [[ -f /etc/fedora-release ]]; then
            if command_exists dnf; then
                echo "fedora"
            else
                echo "redhat"
            fi
        elif [[ -f /etc/arch-release ]]; then
            echo "arch"
        elif [[ -f /etc/SuSE-release ]] || [[ -f /etc/suse-release ]]; then
            echo "suse"
        else
            echo "unknown"
        fi
    else
        echo "unsupported"
    fi
}

# 安装包管理器依赖
install_dependencies() {
    local os="$1"
    log_info "检测到操作系统: $os"
    
    case "$os" in
        "macos")
            if ! command_exists brew; then
                log_error "Homebrew 未安装。请先安装 Homebrew："
                log_error "/bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
                exit 1
            fi
            log_info "更新 Homebrew..."
            brew update
            
            log_info "安装 Git..."
            brew install git || true
            
            log_info "安装 Rust..."
            if ! command_exists rustc; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                source ~/.cargo/env
            else
                log_info "Rust 已安装"
            fi
            ;;
            
        "debian")
            log_info "更新包列表..."
            sudo apt update
            
            log_info "安装 Git..."
            sudo apt install -y git curl build-essential
            
            log_info "安装 Rust..."
            if ! command_exists rustc; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                source ~/.cargo/env
            else
                log_info "Rust 已安装"
            fi
            ;;
            
        "fedora")
            log_info "安装依赖..."
            sudo dnf install -y git curl gcc
            
            log_info "安装 Rust..."
            if ! command_exists rustc; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                source ~/.cargo/env
            else
                log_info "Rust 已安装"
            fi
            ;;
            
        "redhat")
            log_info "安装 EPEL 仓库..."
            sudo yum install -y epel-release
            
            log_info "安装依赖..."
            sudo yum install -y git curl gcc
            
            log_info "安装 Rust..."
            if ! command_exists rustc; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                source ~/.cargo/env
            else
                log_info "Rust 已安装"
            fi
            ;;
            
        "arch")
            log_info "更新包数据库..."
            sudo pacman -Syu
            
            log_info "安装依赖..."
            sudo pacman -S --noconfirm git curl base-devel
            
            log_info "安装 Rust..."
            if ! command_exists rustc; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                source ~/.cargo/env
            else
                log_info "Rust 已安装"
            fi
            ;;
            
        "suse")
            log_info "更新包数据库..."
            sudo zypper refresh
            
            log_info "安装依赖..."
            sudo zypper install -y git curl gcc
            
            log_info "安装 Rust..."
            if ! command_exists rustc; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                source ~/.cargo/env
            else
                log_info "Rust 已安装"
            fi
            ;;
            
        *)
            log_error "不支持的操作系统: $os"
            exit 1
            ;;
    esac
}

# 检查和配置 Ollama
setup_ollama() {
    log_info "检查 Ollama 环境..."
    
    if ! command_exists ollama; then
        log_warning "Ollama 未安装"
        echo -e "${YELLOW}请先安装 Ollama:${NC}"
        echo "macOS: brew install ollama"
        echo "Linux: curl -fsSL https://ollama.com/install.sh | sh"
        echo ""
        echo "安装完成后，请启动 Ollama 服务："
        echo "ollama serve"
        echo ""
        read -p "按回车键继续，或 Ctrl+C 退出..."
        
        # 重新检查
        if ! command_exists ollama; then
            log_error "Ollama 仍未安装。请先安装 Ollama 后再运行此脚本。"
            exit 1
        fi
    fi
    
    # 检查 Ollama 服务是否运行
    if ! curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
        log_warning "Ollama 服务未运行"
        echo "请启动 Ollama 服务："
        echo "ollama serve"
        echo ""
        read -p "启动 Ollama 服务后，按回车键继续..."
        
        # 再次检查
        if ! curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
            log_error "无法连接到 Ollama 服务。请确保 Ollama 正在运行。"
            exit 1
        fi
    fi
    
    log_success "Ollama 环境检查通过"
    
    # 检查是否有可用的模型
    local models=$(curl -s http://localhost:11434/api/tags | grep -o '"name":"[^"]*"' | cut -d'"' -f4 | head -5)
    if [[ -z "$models" ]]; then
        log_warning "未找到 Ollama 模型"
        echo "建议至少安装一个模型，例如："
        echo "ollama pull llama3.2"
        echo "ollama pull qwen2.5:7b"
        echo ""
        read -p "按回车键继续..."
    else
        log_info "可用的 Ollama 模型:"
        echo "$models" | sed 's/^/  - /'
    fi
}

# 克隆和构建项目
build_project() {
    local install_dir="$HOME/.git-commit-analyzer-temp"
    
    log_info "创建临时安装目录..."
    rm -rf "$install_dir"
    mkdir -p "$install_dir"
    cd "$install_dir"
    
    log_info "克隆项目仓库..."
    git clone https://github.com/zh30/git-commit-analyzer.git .
    
    log_info "构建项目..."
    if command_exists cargo; then
        cargo build --release
    else
        log_error "Cargo 未找到。请确保 Rust 已正确安装。"
        exit 1
    fi
    
    # 创建插件目录
    local plugin_dir="$HOME/.git-plugins"
    mkdir -p "$plugin_dir"
    
    # 复制二进制文件
    log_info "安装二进制文件..."
    cp target/release/git-ca "$plugin_dir/"
    
    # 清理临时目录
    cd ~
    rm -rf "$install_dir"
    
    log_success "项目构建和安装完成"
}

# 设置环境变量
setup_environment() {
    local shell_config=""
    
    # 检测当前shell
    case "$SHELL" in
        */bash)
            if [[ -f "$HOME/.bashrc" ]]; then
                shell_config="$HOME/.bashrc"
            elif [[ -f "$HOME/.bash_profile" ]]; then
                shell_config="$HOME/.bash_profile"
            fi
            ;;
        */zsh)
            shell_config="$HOME/.zshrc"
            ;;
        */fish)
            shell_config="$HOME/.config/fish/config.fish"
            ;;
    esac
    
    if [[ -n "$shell_config" ]]; then
        # 检查是否已经添加过PATH
        if ! grep -q 'export PATH="$HOME/.git-plugins:$PATH"' "$shell_config"; then
            log_info "添加环境变量到 $shell_config..."
            echo "" >> "$shell_config"
            echo "# Git Commit Analyzer" >> "$shell_config"
            echo 'export PATH="$HOME/.git-plugins:$PATH"' >> "$shell_config"
            log_success "环境变量已添加"
        else
            log_info "环境变量已存在"
        fi
    else
        log_warning "无法检测到shell配置文件，请手动添加以下内容到您的shell配置："
        echo 'export PATH="$HOME/.git-plugins:$PATH"'
    fi
    
    # 为当前会话设置PATH
    export PATH="$HOME/.git-plugins:$PATH"
}

# 配置Git设置
setup_git_config() {
    log_info "配置Git设置..."
    
    # 获取用户信息
    local current_name=$(git config --global user.name 2>/dev/null || echo "")
    local current_email=$(git config --global user.email 2>/dev/null || echo "")
    
    if [[ -z "$current_name" ]]; then
        read -p "请输入您的Git用户名: " git_name
        git config --global user.name "$git_name"
        log_success "Git用户名已设置"
    else
        log_info "Git用户名已设置: $current_name"
    fi
    
    if [[ -z "$current_email" ]]; then
        read -p "请输入您的Git邮箱: " git_email
        git config --global user.email "$git_email"
        log_success "Git邮箱已设置"
    else
        log_info "Git邮箱已设置: $current_email"
    fi
}

# 初始化git-ca配置
initialize_git_ca() {
    if command_exists git-ca; then
        log_info "初始化git-ca配置..."
        
        # 设置默认语言
        echo "en" | git-ca language >/dev/null 2>&1 || true
        
        log_success "git-ca配置初始化完成"
    else
        log_warning "git-ca命令不可用，请重新加载shell或重启终端"
    fi
}

# 验证安装
verify_installation() {
    log_info "验证安装..."
    
    # 检查二进制文件是否存在
    if [[ -f "$HOME/.git-plugins/git-ca" ]]; then
        log_success "二进制文件已安装"
    else
        log_error "二进制文件未找到"
        return 1
    fi
    
    # 检查PATH设置
    if command_exists git-ca; then
        log_success "git-ca命令可用"
    else
        log_warning "git-ca命令当前不可用，请重新加载shell或运行："
        echo "source ~/.bashrc  # 或相应的shell配置文件"
    fi
    
    # 检查版本
    if command_exists git-ca; then
        local version=$(git-ca --version 2>/dev/null || echo "未知版本")
        log_success "Git Commit Analyzer $version 已安装"
    fi
}

# 显示使用说明
show_usage() {
    echo ""
    echo -e "${GREEN}🎉 Git Commit Analyzer 安装完成！${NC}"
    echo ""
    echo "使用方法："
    echo "  1. 在Git仓库中添加文件：git add <file>"
    echo "  2. 生成提交信息：git ca"
    echo "  3. 按照提示操作即可"
    echo ""
    echo "配置命令："
    echo "  git ca model    # 选择默认模型"
    echo "  git ca language # 设置输出语言"
    echo "  git ca --version # 查看版本"
    echo ""
    echo "重要提示："
    echo "  - 请确保Ollama服务正在运行：ollama serve"
    echo "  - 如果git-ca命令不可用，请重新加载shell或重启终端"
    echo "  - 首次运行时会提示选择Ollama模型"
    echo ""
    echo -e "${BLUE}项目地址：https://github.com/zh30/git-commit-analyzer${NC}"
    echo ""
}

# 主函数
main() {
    echo -e "${BLUE}"
    echo "=============================================="
    echo "    Git Commit Analyzer 安装程序"
    echo "=============================================="
    echo -e "${NC}"
    
    # 检测操作系统
    local os
    os=$(detect_os)
    
    if [[ "$os" == "unsupported" ]]; then
        log_error "不支持的操作系统"
        exit 1
    fi
    
    # 安装依赖
    install_dependencies "$os"
    
    # 设置Ollama
    setup_ollama
    
    # 构建项目
    build_project
    
    # 设置环境变量
    setup_environment
    
    # 配置Git
    setup_git_config
    
    # 初始化git-ca
    initialize_git_ca
    
    # 验证安装
    verify_installation
    
    # 显示使用说明
    show_usage
    
    log_success "安装完成！"
}

# 运行主函数
main "$@"