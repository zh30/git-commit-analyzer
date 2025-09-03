# Git Commit Analyzer 安装脚本部署指南

## 部署步骤

### 1. 上传安装脚本

将 `install-git-ca.sh` 文件上传到你的 CDN 服务器或静态文件托管服务。

#### 支持的托管服务：
- **GitHub Raw**: `https://raw.githubusercontent.com/zh30/git-commit-analyzer/main/install-git-ca.sh`
- **GitHub Pages**: `https://zh30.github.io/git-commit-analyzer/install-git-ca.sh`
- **CDN 服务**: Cloudflare, AWS CloudFront, 阿里云 CDN 等
- **对象存储**: AWS S3, 腾讯云 COS, 阿里云 OSS 等

### 2. 更新 README 文件中的 URL

将所有 README 文件中的 `https://cdn.example.com/install-git-ca.sh` 替换为你的实际 URL：

```bash
# 在项目根目录执行
find . -name "README*.md" -exec sed -i '' 's|https://cdn.example.com/install-git-ca.sh|https://sh.zhanghe.dev/install-git-ca.sh|g' {} \;
```

### 3. 测试安装脚本

在测试环境中验证安装脚本是否正常工作：

```bash
# 测试安装脚本
bash -c "$(curl -fsSL https://your-actual-url.com/install-git-ca.sh)"
```

## 推荐的部署方式

### 方式一：GitHub Raw（免费）

```bash
# 直接使用 GitHub Raw URL
bash -c "$(curl -fsSL https://raw.githubusercontent.com/zh30/git-commit-analyzer/main/install-git-ca.sh)"
```

**优点**：
- 免费
- 自动与仓库同步
- 无需额外配置

**缺点**：
- 在某些地区可能访问较慢
- 有速率限制

### 方式二：GitHub Pages（免费）

1. 创建 `gh-pages` 分支或使用 `docs/` 目录
2. 将 `install-git-ca.sh` 放入相应位置
3. 启用 GitHub Pages

```bash
# 访问 URL
https://username.github.io/git-commit-analyzer/install-git-ca.sh
```

### 方式三：CDN 加速（推荐）

使用 CDN 服务加速 GitHub Raw 内容：

```bash
# 使用 jsDelivr CDN
https://cdn.jsdelivr.net/gh/zh30/git-commit-analyzer@latest/install-git-ca.sh

# 使用 UNPKG
https://unpkg.com/browse/git-commit-analyzer@latest/install-git-ca.sh
```

## 安全考虑

### 1. 脚本签名（可选）

为了增加安全性，可以考虑对脚本进行签名：

```bash
# 生成签名
gpg --detach-sign --armor install-git-ca.sh

# 用户验证
curl -fsSL https://your-url.com/install-git-ca.sh | gpg --verify
```

### 2. 版本控制

建议在 URL 中包含版本信息：

```bash
# 包含版本号
https://sh.zhanghe.dev/install-git-ca-v1.0.3.sh

# 使用 latest 标签
https://sh.zhanghe.dev/install-git-ca-latest.sh
```

### 3. 访问统计

如果需要统计安装次数，可以使用重定向服务：

```bash
# 使用短链接服务
https://git-ca.install/install
https://bit.ly/git-ca-install
```

## 监控和维护

### 1. 访问日志监控

监控安装脚本的下载次数：

```bash
# nginx 访问日志
tail -f /var/log/nginx/access.log | grep install-git-ca.sh

# AWS CloudFront 监控
aws cloudwatch get-metric-statistics --namespace AWS/CloudFront --metric-name Requests --dimensions Name=DistributionId,Value=YOUR_DISTRIBUTION_ID --start-time 2024-01-01T00:00:00Z --end-time 2024-01-02T00:00:00Z --period 86400 --statistics Sum
```

### 2. 定期更新

定期检查和更新安装脚本：

- 依赖包版本更新
- 新的操作系统支持
- 安全漏洞修复
- 功能改进

### 3. 回滚策略

准备回滚方案：

```bash
# 保留多个版本的安装脚本
install-git-ca-v1.0.0.sh
install-git-ca-v1.0.1.sh
install-git-ca-latest.sh

# 使用符号链接切换版本
ln -sf install-git-ca-v1.0.1.sh install-git-ca-latest.sh
```

## 故障排除

### 常见问题

1. **CORS 错误**
   - 确保 CDN 服务器配置了正确的 CORS 头
   - 检查 `Access-Control-Allow-Origin` 设置

2. **SSL 证书问题**
   - 确保使用 HTTPS
   - 检查证书是否有效

3. **脚本执行权限**
   - 确保脚本有执行权限
   - 检查文件权限设置

4. **网络连接问题**
   - 提供备用下载链接
   - 考虑使用多个 CDN 源

### 调试方法

```bash
# 测试脚本下载
curl -I https://your-url.com/install-git-ca.sh

# 检查脚本内容
curl -fsSL https://your-url.com/install-git-ca.sh | head -10

# 验证脚本语法
bash -n <(curl -fsSL https://your-url.com/install-git-ca.sh)
```

## 总结

一键安装脚本大大提升了用户体验，将复杂的多步骤安装过程简化为单行命令。选择合适的部署方式并做好监控维护，能够确保用户获得最佳的安装体验。