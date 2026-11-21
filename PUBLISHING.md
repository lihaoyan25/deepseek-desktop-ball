# 发布 Release

普通用户只需要在 GitHub Releases 下载安装包，不需要命令，也不需要编译。

## 1. 推送源码

先在 GitHub 创建一个仓库，然后在项目根目录推送：

```powershell
git init
git add .
git status --short
git commit -m "Initial release"
git branch -M main
git remote add origin https://github.com/<你的账号>/<仓库名>.git
git push -u origin main
```

提交前确认 `dist-installer/`、`reference/`、根 `assets/`、`tools/` 和内部
`docs/*.md` 没有被加入暂存区；这些都在 `.gitignore` 中。

## 2. 创建 Release（网页操作即可）

1. 打开仓库的 **Releases** → **Draft a new release**；
2. Tag 填 `v0.1.0`，标题填 `v0.1.0`；
3. 说明粘贴 [RELEASE_NOTES_v0.1.0.md](RELEASE_NOTES_v0.1.0.md) 的内容；
4. 上传以下两个文件：
   - `dist-installer/DeepSeekBall_0.1.0_x64-setup.exe`
   - `dist-installer/SHA256SUMS.txt`
5. 点击 **Publish release**。

发布后，普通用户进入仓库的 **Releases** 页面，直接点击安装包下载即可。

## 3. 固定直链

把下面链接中的账号和仓库名替换掉，就是始终指向最新 Release 安装包的直链：

```text
https://github.com/<你的账号>/<仓库名>/releases/latest/download/DeepSeekBall_0.1.0_x64-setup.exe
```

可以把这条链接放进 README、论坛帖子或下载页。

## 可选：命令行发布

如果你已经安装并登录 GitHub CLI，可以用一条命令发布：

```powershell
gh release create v0.1.0 `
  dist-installer/DeepSeekBall_0.1.0_x64-setup.exe `
  dist-installer/SHA256SUMS.txt `
  --title "v0.1.0" `
  --notes-file RELEASE_NOTES_v0.1.0.md
```

这只是维护者发布用的可选命令；下载用户不需要执行。
