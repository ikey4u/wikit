Wikit GUI 基于 Native Shell (NSH) 重写的测试版本，业务与文件存储由 Rust 实现。

- Android：Android 9+，ARM64，下载 `.apk` 安装。
- macOS：macOS 14+，Apple Silicon，下载 `.zip` 解压后打开 Wikit.app。
- 支持本地/远程词典查询、词典导入、收藏、查询历史、学习笔记、外观切换与 HTML 页面预览。
- 附带 SHA256SUMS.txt，可核对下载文件的 SHA-256。

安装说明：Android 使用测试签名，无法覆盖安装签名不同的旧版；macOS 使用临时签名且未经 Apple 公证，首次启动可能需要在“系统设置 → 隐私与安全性”允许打开。

已知限制：移动端暂不提供系统文件选择器，可使用下载 URL 或可访问的沙箱路径导入 .wikit 文件；导入不会自动复制文件旁的外部媒体目录。iOS 不包含在本次发布中。

这是预发布测试版，请通过 GitHub Issues 反馈问题。每个发布标签都对应 Actions 构建使用的源码提交。
