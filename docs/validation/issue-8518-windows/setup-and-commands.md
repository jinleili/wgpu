# 环境准备与执行命令

以下记录原验证机的命令，路径相对 `D:/rust/forks/wgpu`。版本见 [报告](README.md)，下载 hash、各次完整参数和环境已汇总于 [evidence.json](evidence.json)。独立日志文件仍保存在原机 target 目录。换机使用时调整盘符、路径、GPU 名称和本机代理；不要覆盖已存在的 target/debug 或 target/agility-sdk。

## 初始核对

```powershell
git status --short
git rev-parse HEAD
git branch --show-current
Get-Content AGENTS.md
Get-Content docs/testing.md
Get-Content .github/actions/install-mesa/action.yml
Get-Content .github/actions/install-dxc/action.yml
Get-Content .github/actions/install-vulkan-sdk/action.yml
Get-Content .github/workflows/ci.yml
Get-CimInstance Win32_OperatingSystem
Get-CimInstance Win32_VideoController
```

交接文档原文保存为 handoff.md；固定 raw URL：
`https://raw.githubusercontent.com/jinleili/wgpu-in-app/68ad306a05e65ba5708fd494e1ef86fbbec22b1a/wgpu-in-app/examples/issue_8518_windows_validation_handoff.md`。

## 下载与准备

```powershell
$out='D:/rust/forks/wgpu/target/issue-8518-windows'
Invoke-WebRequest 'https://github.com/pal1000/mesa-dist-win/releases/download/26.1.3/mesa3d-26.1.3-release-msvc.7z' -OutFile "$out/mesa-iwr.7z"
Invoke-WebRequest 'https://github.com/microsoft/DirectXShaderCompiler/releases/download/v1.9.2602.24/dxc_2026_05_27.zip' -OutFile "$out/dxc.zip"
Invoke-WebRequest 'https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-0.9.145/cargo-nextest-0.9.145-x86_64-pc-windows-msvc.zip' -OutFile "$out/nextest.zip"
Invoke-WebRequest 'https://www.7-zip.org/a/7zr.exe' -OutFile "$out/7zr.exe"
Invoke-WebRequest 'https://www.7-zip.org/a/7z2603-extra.7z' -OutFile "$out/7z-extra.7z"
Invoke-WebRequest 'https://sdk.lunarg.com/sdk/download/1.4.357.0/windows/vulkansdk-windows-X64-1.4.357.0.exe' -OutFile "$out/vulkan-sdk.exe"
& "$out/7zr.exe" x "$out/7z-extra.7z" "-o$out/7zip" -y
& "$out/7zip/x64/7za.exe" x "$out/mesa-iwr.7z" "-o$out/mesa" 'x64/*' -y
Expand-Archive "$out/dxc.zip" "$out/dxc"
Expand-Archive "$out/nextest.zip" "$out/nextest"
$p=Start-Process "$out/vulkan-sdk.exe" -ArgumentList @('--root',"$out/vulkan",'--accept-licenses','--default-answer','--confirm-command','install','copy_only=1') -WindowStyle Hidden -PassThru
$p.WaitForExit()
```

实际初始下载也使用过 `curl.exe -fL --retry 3 URL -o FILE`；慢速 Mesa 直连/分片尝试停止后改用系统代理的 Invoke-WebRequest，最终有效包为 `mesa-iwr.7z`。下载诊断日志保留为 `*.download.log`、`*.retry.log`。7zr 直接解 SDK 安装器的尝试失败，因此改用官方 copy_only 安装。

```powershell
$env:RUSTUP_TOOLCHAIN='1.95.0-x86_64-pc-windows-msvc'
$env:CARGO_TARGET_DIR="$out/build"
$env:CARGO_PROFILE_DEV_DEBUG='line-tables-only'
cargo build --offline --locked -p wgpu-xtask
New-Item -ItemType Directory -Force "$out/agility-sdk"
New-Item -ItemType Junction -Path target/agility-sdk -Target "$out/agility-sdk"
New-Item -ItemType Junction -Path target/debug -Target "$out/build/debug"
& "$out/build/debug/wgpu-xtask.exe" install-agility-sdk
& "$out/build/debug/wgpu-xtask.exe" install-warp --target-dir "$out/build/debug"
. "$out/env.ps1"
cargo test --locked --benches --tests --all-features --no-run
& "$out/vulkan/Bin/vulkaninfoSDK.exe" --summary
```

上面的无运行构建失败/重试日志分别是 build-tests.log、build-tests-retry.log、build-tests-proxy.log。静态 DXC 由依赖脚本下载；使用任务级 CURL_HOME/.curlrc 及 CARGO_HTTP_PROXY 处理本机代理，未修改依赖源码。

## 主验证

```powershell
& "$out/run.ps1" -Label lavapipe-targeted-ready -Filter 'test(first_uses_preserve_order_and_gaps) | test(zero_init::texture_binding) | test(Texture Init Render)'
& "$out/run.ps1" -Label lavapipe-zero-init -Filter 'binary(wgpu-gpu) & test(zero_init)'
```

统一执行器实质执行 `cargo xtask test --test-threads 4 ... -E FILTER`。初期使用 status/final-status=all，后续改为 pass/fail 以减少范围外跳过输出；各次精确参数已记录。

## 隔离故障与恢复

```powershell
git worktree add --detach "$out/fault" f47afd2b6aa8872eb283e808363e180f3930e9e3
Copy-Item .gpuconfig "$out/fault/.gpuconfig"
```

只在 fault worktree 的 `wgpu-core/src/command/memory_init.rs` 应用 fault-injection.patch。之后在该 worktree 中，继承 env.ps1 的 Lavapipe 环境，执行：

```powershell
cargo nextest run --locked --benches --tests --all-features -E 'binary(wgpu-gpu) & test(zero_init::texture_binding::dropped_command_buffer)' --success-output immediate --failure-output immediate
git restore --source=f47afd2b6aa8872eb283e808363e180f3930e9e3 -- wgpu-core/src/command/memory_init.rs
cargo nextest run --locked --benches --tests --all-features -E 'binary(wgpu-gpu) & test(zero_init::texture_binding::dropped_command_buffer)' --success-output immediate --failure-output immediate
git diff --exit-code
```

分别对应 fault-injected、fault-restored 的 log/command/result 文件。恢复后回主仓库运行全量，再进行硬件阶段：

```powershell
& "$out/run.ps1" -Label lavapipe-full -Full
& "$out/run.ps1" -Label hardware-vulkan -Mode vulkan -Filter 'test(first_uses_preserve_order_and_gaps) | test(Texture Init Render) | (binary(wgpu-gpu) & test(zero_init) & test(Intel))'
& "$out/run.ps1" -Label hardware-dx12 -Mode dx12 -Filter 'test(first_uses_preserve_order_and_gaps) | test(Texture Init Render) | (binary(wgpu-gpu) & test(zero_init) & test(Intel))'
& "$out/run.ps1" -Label hardware-vulkan-single -Mode vulkan -Filter 'test(first_uses_preserve_order_and_gaps) | test(Texture Init Render) | (binary(wgpu-gpu) & test(zero_init) & test(Intel) & test(/GPU\/0\]/))'
```

Tracy 单独诊断仅在其子进程设置：

```powershell
$env:TRACY_SYMBOL_OFFLINE_RESOLVE='1'
& "$out/run.ps1" -Label dx12-tracy-diagnostic -Mode dx12 -Filter 'test(Texture Init Render)'
```

原始主验证没有这个 Tracy 变量。最后保存测试统计及所有跳过名称，确认 snapshot 生成只有换行状态变化后恢复 `naga/tests/out`，核对主仓库与故障 worktree 均干净。没有运行 cargo bench 性能模式、CTS、基线测试或发布操作。

## 验证结束时的辅助脚本文本

这些是原机脚本的文档副本；如需复现，调整配置后分别保存到新机器的任务产物目录。脚本不含故障注入。原机 .curlrc 使用 retry=5、retry-all-errors、connect-timeout=30，以及原机本地代理；新机器按实际网络配置准备，不能假设 127.0.0.1:7897 可用。

### env.ps1

```powershell
param([ValidateSet('lavapipe','vulkan','dx12')][string]$Mode='lavapipe')
$ErrorActionPreference='Stop'
$ValidationRoot='D:/rust/forks/wgpu/target/issue-8518-windows'
$env:RUSTUP_TOOLCHAIN='1.95.0-x86_64-pc-windows-msvc'
$env:CARGO_TARGET_DIR="$ValidationRoot/build"
$env:CARGO_PROFILE_DEV_DEBUG='line-tables-only'
$env:CARGO_TERM_COLOR='never'
$env:CURL_HOME=$ValidationRoot
$env:CARGO_HTTP_PROXY='http://127.0.0.1:7897'
$env:RUST_BACKTRACE='1'
$env:RUST_LOG='info'
$env:PATH="$ValidationRoot/nextest;$ValidationRoot/dxc/bin/x64;$ValidationRoot/vulkan/Bin;$env:PATH"
$env:WGPU_DX12_COMPILER="$ValidationRoot/dxc/bin/x64/dxcompiler.dll"
$env:VK_LAYER_PATH="$ValidationRoot/vulkan/Bin"
if ($Mode -eq 'lavapipe') {
    $env:PATH="$ValidationRoot/mesa/x64;$env:PATH"
    $env:VK_DRIVER_FILES="$ValidationRoot/mesa/x64/lvp_icd.x86_64.json"
    $env:LVP_POISON_MEMORY='true'
    $env:GALLIUM_DRIVER='llvmpipe'
    $env:WGPU_BACKEND='vulkan'
    $env:WGPU_ADAPTER_NAME='llvmpipe'
} else {
    foreach ($name in @('VK_DRIVER_FILES','VK_ICD_FILENAMES','LVP_POISON_MEMORY','GALLIUM_DRIVER')) {
        Remove-Item "Env:$name" -ErrorAction SilentlyContinue
    }
    $env:WGPU_BACKEND=$Mode
    $env:WGPU_ADAPTER_NAME='Intel(R) Arc(TM) B390 GPU'
}

```

### run.ps1

```powershell
param(
    [string]$Label,
    [ValidateSet('lavapipe','vulkan','dx12')][string]$Mode='lavapipe',
    [string]$Filter='',
    [string]$Workdir='D:/rust/forks/wgpu',
    [switch]$Full,
    [switch]$List
)
. "$PSScriptRoot/env.ps1" -Mode $Mode
Set-Location $Workdir
$arguments=@('xtask','test','--test-threads','4','--status-level','pass','--final-status-level','fail','--success-output','immediate','--failure-output','immediate')
if ($List) { $arguments=@('xtask','test','--list') }
if (!$Full) { $arguments+=@('-E',$Filter) }
$record=[ordered]@{label=$Label;time=(Get-Date -Format o);cwd=$Workdir;head=(& git rev-parse HEAD);command='cargo';args=$arguments;environment=@{}}
foreach ($name in @('RUSTUP_TOOLCHAIN','CARGO_TARGET_DIR','CARGO_PROFILE_DEV_DEBUG','WGPU_BACKEND','WGPU_ADAPTER_NAME','WGPU_DX12_COMPILER','VK_DRIVER_FILES','VK_LAYER_PATH','LVP_POISON_MEMORY','GALLIUM_DRIVER')) { $record.environment[$name]=[Environment]::GetEnvironmentVariable($name,'Process') }
$record | ConvertTo-Json -Depth 5 | Set-Content "$ValidationRoot/$Label.command.json"
& cargo @arguments > "$ValidationRoot/$Label.log" 2>&1
$result=$LASTEXITCODE
if (Test-Path .gpuconfig) { Copy-Item .gpuconfig "$ValidationRoot/$Label.gpuconfig.json" }
@{exit=$result;finished=(Get-Date -Format o)} | ConvertTo-Json | Set-Content "$ValidationRoot/$Label.result.json"
Get-Content "$ValidationRoot/$Label.log" | Select-String 'Starting |Summary |FAIL \[|error:|Error:' | Select-Object -Last 20
exit $result

```
