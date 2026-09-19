# 第二轮完整重测：固定版本、构建与复现命令

## 来源与边界

- 仓库：`https://github.com/jinleili/wgpu`，原工作目录 `D:/rust/forks/wgpu`。
- 当前分支：`codex/issue-8518-texture-folding`；构建准备时 HEAD 为 `8c91d7a80a814d8641b2b69862e4ae37a3e11a1b`，工作区干净。第二轮开始时仅有此前生成的未跟踪性能报告目录。该提交仅添加此前正确性报告。
- A：`f12c3e4508cb706c1e1faeb973bdb9c6970f5e15`。
- B：`f47afd2b6aa8872eb283e808363e180f3930e9e3`。
- 本次未修改生产实现、未运行完整基线测试或 CTS，也未重跑 poison-memory 全量验证。未提交、推送或创建 PR。

以下 PowerShell 命令中的 `$r` 为 `D:/rust/forks/wgpu/target/issue-8518-windows-performance`。所有工具、脚本、二进制和逐进程原始日志均留在该忽略目录；本报告附带的 JSON 保留全部正式进程的计时值，阅读结果无需访问该 Windows 目录。

## 公平构建

```powershell
$r = 'D:/rust/forks/wgpu/target/issue-8518-windows-performance'
git worktree add --detach "$r/worktrees/a" f12c3e4508cb706c1e1faeb973bdb9c6970f5e15
git worktree add --detach "$r/worktrees/b" f47afd2b6aa8872eb283e808363e180f3930e9e3
git archive --format=tar -o "$r/candidate-benches.tar" f47afd2b6aa8872eb283e808363e180f3930e9e3 benches
tar -xf "$r/candidate-benches.tar" -C "$r/worktrees/a"
$env:RUSTUP_TOOLCHAIN = '1.95.0-x86_64-pc-windows-msvc'
$env:CARGO_TARGET_DIR = "$r/build"
# 本机网络下载使用任务级代理；正式运行二进制不需要网络。
$env:CARGO_HTTP_PROXY = 'http://127.0.0.1:7897'
$env:CURL_HOME = $r
# .curlrc: proxy=http://127.0.0.1:7897; retry=5;
# retry-all-errors; connect-timeout=30
foreach ($v in @('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_RUSTFLAGS')) {
    Remove-Item "Env:$v" -ErrorAction SilentlyContinue
}
foreach ($v in @('a','b')) {
    Push-Location "$r/worktrees/$v"
    cargo bench --locked -p wgpu-benchmark --bench wgpu-benchmark --no-run --message-format=json
    # 从 compiler-artifact.executable 复制到 binaries/<版本>/wgpu-benchmark.exe。
    # A 复制完成后才构建 B；正式测量只运行这两个固定文件。
    cargo tree --locked -p wgpu-benchmark -e features
    Pop-Location
}
```

整个 `benches/` 目录逐文件 SHA-256 相同，Cargo.lock 相同，features 树在规范化工作区路径后完全相同。构建使用默认 benchmark features，未传 `--all-features`；benchmark 自身 features 为空，`profiling` features 为空，无 Tracy/Superluminal crate。工作区默认依赖包含 `static-dxc`，但运行时明确选择动态 DXC。未添加 allocator 探针。

编译设置：bench 继承 release，`opt-level=3`、thin LTO、debug info 2、debug assertions/overflow checks 关闭；保留仓库 `.cargo/config.toml` 的 `--cfg wgpu_validate_locks_debug`。Cargo artifact 的 `profile.test=true` 表示 benchmark target 构建，运行模式由实际命令中的 `--bench` 确认。正常 wgpu-core API 验证和原像素断言仍在。

A/B 生产代码差异仅在 `wgpu-core/src/command/memory_init.rs`、`wgpu-core/src/command/pass.rs`、`wgpu-core/src/init_tracker/texture.rs`。提交间还存在测试源码差异，但本次未构建执行这些测试；基准源码已覆盖为完全一致的 B 版本。补丁、源文件哈希、编译参数和二进制哈希见 [evidence.json](evidence.json) 的 `build`。

## 运行环境与命令

```powershell
foreach ($v in @('VK_DRIVER_FILES','VK_ICD_FILENAMES','VK_ADD_DRIVER_FILES',
 'LVP_POISON_MEMORY','GALLIUM_DRIVER','VK_INSTANCE_LAYERS',
 'VK_LOADER_LAYERS_ENABLE','VK_LOADER_LAYERS_ALLOW','VK_LAYER_PATH',
 'VK_ADD_LAYER_PATH','VK_LOADER_DEBUG','WGPU_TRACE','WGPU_ADAPTER_INDEX',
 'ENABLE_VULKAN_RENDERDOC_CAPTURE','ENABLE_VULKAN_OBS_CAPTURE')) {
    Remove-Item "Env:$v" -ErrorAction SilentlyContinue
}
$env:VK_LOADER_LAYERS_DISABLE = '*'
$env:WGPU_ADAPTER_NAME = 'Intel(R) Arc(TM) B390 GPU'
$env:WGPU_DEBUG = '0'
$env:WGPU_VALIDATION = '0'
$env:WGPU_GPU_BASED_VALIDATION = '0'
$env:RAYON_NUM_THREADS = '8'
$env:WGPU_DX12_COMPILER = 'Dxc'
$env:WGPU_DX12_AGILITY_SDK_PATH = "$r/tools/agility/"
$env:WGPU_DX12_AGILITY_SDK_VERSION = '619'
$env:WGPU_DX12_AGILITY_SDK_REQUIRE = '1'
$env:PATH = "$r/tools/dxc;" + (($env:PATH -split ';' |
    Where-Object { $_ -notmatch 'issue-8518-windows|Mesa|lavapipe' }) -join ';')
$env:RUST_LOG = 'warn'
$env:WGPU_BACKEND = 'vulkan' # 第二个后端使用 dx12
& "$r/binaries/a/wgpu-benchmark.exe" --help
& "$r/binaries/a/wgpu-benchmark.exe" --bench --exact 'Texture Init Render: shared attachments' --iters 128 --color never --save-baseline r2-vulkan-aa-g1-s1-p1-a
```

完整场景名分别为 `Texture Init Render: shared attachments`、`Texture Init Render: overlapping mixed views`、`Texture Init Render: discard then sample`、`Texture Init Render: no sampled textures`。其余命令仅替换版本、后端、场景和唯一 run ID；全部顺序可从 JSON 的 `samples` 数组恢复。

每进程独立启动，正常优先级，前后间隔 1 秒。原始工作量保持 1000 个组、10 个消费 pass、10000 draws，前三场景 16 对共享纹理；无采样对照没有采样纹理。每进程 8 次预热并读回像素后计时，GPU 等待和读回不计入 Encoding/Submit。创建阶段为独立计时区间。

每次用唯一 `--save-baseline <run-id>`，随后立即复制该 JSON 到 `full-round-2/raw/<run-id>.json`，stdout、stderr、命令、开始结束时间和退出码分别保存。本轮所有 run ID 以 `r2-` 开头，独立于历史结果。框架仍更新 `previous.json`，但历史结果不依赖它；终端自动比较百分比不参与分析。检查每进程确实输出一个场景、三项指标、每项 128 次迭代及 10000 draws/1000 groups，退出码 0 且实际 adapter 匹配。

## 固定协议

随机种子 `20260919`，uint32 LCG `x=1664525*x+1013904223`，每组四场景用 Fisher–Yates 排序，遍历后端 Vulkan/DX12、阶段 A/A/A/B、组号，顺序在正式运行前写入 schedule.json 并记录哈希。A/A 每场景两组 AAAA，用中间两个均值相对首尾两个均值估计漂移；A/B 每场景四组，依次 ABBA、BAAB、ABBA、BAAB。每后端 32+64=96 个正式进程，两后端共 192 个。

预检另有每后端每场景每版本一个进程，使用 `--bench --iters 8`，共 16 个，不混入正式结果。预检读取模块列表核实 DLL；正式进程不进行模块轮询或 profiler 采样。各正式阶段前单独进行 3 秒后台进程 CPU 观测，不与正式计时并行。

第二轮沿用首次构建的两个固定二进制，未重新构建。原场景排列保持不变，保存到 `full-round-2/schedule.json` 并在测量前记录 SHA-256。实际编排命令为：

```powershell
# 从既有固定 schedule 派生，所有 run ID 加 r2- 前缀；不重选顺序。
& "$r/run-round2.ps1" -Backend vulkan -Phase preflight
& "$r/run-round2.ps1" -Backend dx12 -Phase preflight
& "$r/health.ps1" -Label round2-before-vulkan-aa
& "$r/run-round2.ps1" -Backend vulkan -Phase aa
node "$r/full-round-2/analyze.cjs"
& "$r/health.ps1" -Label round2-before-vulkan-ab
& "$r/run-round2.ps1" -Backend vulkan -Phase ab
node "$r/full-round-2/analyze.cjs"
& "$r/health.ps1" -Label round2-before-dx12-aa
& "$r/run-round2.ps1" -Backend dx12 -Phase aa
node "$r/full-round-2/analyze.cjs"
& "$r/health.ps1" -Label round2-before-dx12-ab
& "$r/run-round2.ps1" -Backend dx12 -Phase ab
node "$r/full-round-2/analyze.cjs"
node "$r/full-round-2/verify.cjs"
& "$r/full-round-2/check-statistics.ps1"
node "$r/full-round-2/report.cjs"
```

本轮是用户明确要求的完整重测，主结论仅使用本轮独立的 A/A 和 A/B；历史数据不混入本轮统计。本轮内部没有因显著性不足追加样本，也没有移除不利结果。A/A 观测漂移是本次环境下的有限经验参照，不是置信区间或未来噪声上界。
