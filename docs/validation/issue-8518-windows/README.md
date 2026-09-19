# wgpu #8518 Windows 正确性验证报告

日期：2026-09-19（Asia/Shanghai）。结论：本次实测未发现候选版本的 texture initialization folding 正确性错误；三条验证路径均完成。验证阶段没有修改交付源码或创建 PR；本目录是在验证结束后，按用户要求整理入库的报告与证据。

## 跨设备交接

本目录随 Git 提交迁移，不依赖另一台设备存在 Windows 原机的 `target` 目录。

- [环境准备与完整命令](setup-and-commands.md)：包含环境脚本和执行器的文本副本。先按新机器实际路径、GPU、代理配置调整，不直接套用原机设置。
- [结构化证据](evidence.json)：包含各阶段命令、退出码、实际执行计数、adapter 身份、下载校验和与源码恢复核对。
- [关键原始日志摘录](log-excerpts.txt)：保留测试汇总、定向用例、故障断言和诊断消息；摘录附对应完整日志的 SHA-256。
- [全量跳过清单](lavapipe-full-skips.txt)：逐项列出全部 55 项跳过。

下文提到的独立 `.log`、`.gpuconfig.json`、`.command.json` 等文件名，均指原验证机 `target/issue-8518-windows/` 的完整原始材料；本提交保存上述可迁移的关键证据，不包含 SDK、驱动 DLL、构建缓存、故障补丁或临时 worktree。验证结果只针对下面的候选提交；报告提交本身没有改变实现。跨设备后可继续分析实现和验证范围，不能将本机结果表述为其他设备或平台已实测。

## 代码与范围

- 仓库：`D:/rust/forks/wgpu`，分支 `codex/issue-8518-texture-folding`。
- 候选 HEAD：`f47afd2b6aa8872eb283e808363e180f3930e9e3`。
- 指定基线：`f12c3e4508cb706c1e1faeb973bdb9c6970f5e15`。没有更换基线，没有运行基线/trunk 测试或 CTS。
- 初始工作区干净；最终 `git diff --exit-code` 成功，`git status --porcelain=v1` 为空。隔离副本也干净。
- 已读取根 `AGENTS.md`、`docs/testing.md`、Windows CI 和安装 action，以及[指定提交的完整交接文档](https://github.com/jinleili/wgpu-in-app/blob/68ad306a05e65ba5708fd494e1ef86fbbec22b1a/wgpu-in-app/examples/issue_8518_windows_validation_handoff.md)。读取副本保存在原验证机的 `target/issue-8518-windows/`。
- 仅验证正确性。所有 render 基准均通过 nextest 的测试模式执行，没有性能 A/B，日志内软件驱动耗时不作为性能证据。

## 工具、系统与驱动

| 项目                          | 实测版本或身份                                                                                       |
| ----------------------------- | ---------------------------------------------------------------------------------------------------- |
| Windows                       | Windows 11 家庭版中文版，x64，10.0.26200                                                             |
| Rust / Cargo                  | rustc 1.95.0 (59807616e)，cargo 1.95.0 (f2d3ce0bd)，x86_64-pc-windows-msvc                           |
| MSVC                          | VS 18 Community，C/C++ 编译器 19.51.36252，工具目录 14.51.36231                                      |
| cargo-nextest                 | 0.9.145 (00af4550e，2026-09-16)                                                                      |
| Vulkan SDK / validation layer | 1.4.357.0 / VK_LAYER_KHRONOS_validation 1.4.357                                                      |
| 实际 Vulkan loader            | 1.4.313，沿用本机 loader                                                                             |
| Mesa                          | 26.1.3，MSVC x64 发布包，git-bfce4dd45a，LLVM 22.1.8                                                 |
| DXC                           | 1.9.2602.24，d355aa836，CI 指定发布包                                                                |
| 静态 DXC 构建依赖             | mach-dxcompiler-rs 0.1.4+2024.11.22-df583a3.1，对应 2024.11.22+284d956.1 Windows MSVC Dynamic_lib 包 |
| Agility SDK                   | Microsoft.Direct3D.D3D12 1.619.0，feature version 619，含 D3D12Core 和 SDKLayers                     |
| WARP                          | Microsoft.Direct3D.WARP 1.0.20，运行时报告 1.0.20.0；仅枚举，不计入硬件测试通过数                    |
| Intel 驱动                    | 32.0.101.8860；Vulkan driver_info 为 101.8860                                                        |
| 解压工具                      | 7-Zip 26.03                                                                                          |

Vulkan SDK 使用官方支持的 `copy_only=1` 安装到原验证机的 `target/issue-8518-windows/`，未执行系统 layer 注册或修改全局 PATH，也未替换系统 DLL。参见 [LunarG 安装说明](https://vulkan.lunarg.com/doc/view/1.4.357.0/windows/getting_started.html)。本次不需要管理员操作。

工具和 build 产物实际位于原验证机的 `target/issue-8518-windows/`。为兼容 xtask 硬编码路径，`target/debug` 和 `target/agility-sdk` 是指向该 target 目录内 build/debug、agility-sdk 的目录连接。Rust/Cargo 常规用户缓存仍使用机器已有的用户级缓存。

下载 SHA-256 见 `download-sha256.json`。Mesa SHA-256 为 `6dd431f4620cea73970b13e3ffa94f721f2a3924306b8a4283c97648cdb6eb9c`，与 GitHub 发布 API 的 digest 一致。

## Adapter 核实

| 路径                   | 实际 adapter                                  | 身份证据                                                                                                       |
| ---------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Lavapipe poison-memory | llvmpipe (LLVM 22.1.8, 256 bits)，Vulkan，Cpu | vendor 0x10005，driver llvmpipe，Mesa 26.1.3；vulkaninfo、`.gpuconfig`、GPU 测试日志和四个 render 测试输出一致 |
| 硬件 Vulkan            | Intel(R) Arc(TM) B390 GPU，IntegratedGpu      | vendor 0x8086，device 0xb080，Intel Corporation，101.8860，API 1.4.348                                         |
| 硬件 DX12              | Intel(R) Arc(TM) B390 GPU，IntegratedGpu      | vendor 0x8086，device 0xb080，PCI 0000:00:02.0，驱动 32.0.101.8860                                             |
| DX12 软件 adapter      | Microsoft Basic Render Driver，Cpu            | vendor 0x1414，device 0x008c，WARP 1.0.20.0；由筛选明确排除                                                    |

Lavapipe 进程设置 `VK_DRIVER_FILES=.../mesa/x64/lvp_icd.x86_64.json`、`LVP_POISON_MEMORY=true`、`WGPU_BACKEND=vulkan`。成功依据包括实际读回的 poison 值，绝非仅凭环境变量。

硬件阶段删除 `VK_DRIVER_FILES`、`VK_ICD_FILENAMES`、`LVP_POISON_MEMORY`、`GALLIUM_DRIVER`，重新运行 xtask 生成 `.gpuconfig`。GPU harness 按 Intel 名称筛选；基准通过 `WGPU_ADAPTER_NAME` 选择 Intel，并打印实际 adapter。

本机 Vulkan loader 把相同 Intel GPU 枚举了两次，vulkaninfo 中 deviceUUID、driverUUID 相同。首次硬件测试保留原始记录（111 个选中条目，107 实际通过，4 个能力跳过）。随后增加 adapter `/0` 筛选重跑，以下以去重后的一次执行为准。没有把重复枚举当作两张显卡。

关键 adapter 文件：`lavapipe-vulkaninfo.txt`、`lavapipe-full.gpuconfig.json`、`vulkan-hardware-sdk-restored.txt`、`hardware-vulkan-single.gpuconfig.json`、`hardware-dx12.gpuconfig.json`。

## 实际执行结果

这里的“通过”已扣除 harness 返回成功但标记为 `[Unsupported: ...]` 或 `[Skipped Failure: ...]` 的条目。每次 xtask 前置的 `generate_gpuconfig_report` 单独通过 1 项，不混入下表。

| 阶段                                                 | 范围内条目 | 实际执行 | 通过 | 失败 | 范围内跳过 |
| ---------------------------------------------------- | ---------: | -------: | ---: | ---: | ---------: |
| Lavapipe CPU 独立确认                                |          1 |        1 |    1 |    0 |          0 |
| Lavapipe：CPU + 7 回归 + 4 render                    |         12 |       12 |   12 |    0 |          0 |
| Lavapipe：zero_init 扩展筛选                         |         53 |       51 |   51 |    0 |          2 |
| 上两组定向范围的去重并集                             |         58 |       56 |   56 |    0 |          2 |
| Lavapipe poison-memory 全量原生                      |       1127 |     1072 | 1072 |    0 |         55 |
| Intel Vulkan：CPU + zero_init + 4 render，单一枚举项 |         58 |       56 |   56 |    0 |          2 |
| Intel DX12：CPU + zero_init + 4 render               |         58 |       58 |   58 |    0 |          0 |
| DX12 Tracy 消息诊断复测，4 render                    |          4 |        4 |    4 |    0 |          0 |

Lavapipe 全量的原始 nextest summary 是 **1121 tests run: 1121 passed, 6 skipped**。其中 49 个“passed”条目实际由 harness 跳过，扣除后为 1072 实际执行；再加默认过滤排除的 6 个 OOM 用例，总跳过为 55。全部 369 个标记为 `[Executed...]` 的 GPU/示例条目均能对应到 `TEST FINISHED`，没有失配或无 adapter 提前返回被误算为通过。

定向筛选中其余 nextest `skipped` 数字（例如 1493）是范围外测试/未选 adapter，不计为本次定向覆盖内跳过。原始数量完整保存在每次 `.log` 和 `.summary.json`。

七项新增回归在 Lavapipe、Intel Vulkan、Intel DX12 均实际执行且通过：

- `partial_view`
- `dropped_command_buffer`
- `discard_same_encoder`
- `discard_separate_submissions`
- `destroyed_texture`
- `overlapping_views`
- `reordered_submissions`

四个 render 测试模式在三个路径均实际执行且通过，并校验 GPU 输出：shared attachments、overlapping mixed views、discard then sample、no sampled textures。

CPU `first_uses_preserve_order_and_gaps` 通过。该测试枚举 36 种矩形范围的两两组合和 4 种首次使用类型组合，共 5184 组内部场景，比较折叠前后初始化需求。

Lavapipe 与 Intel Vulkan 扩展筛选中各有 2 项缺 feature 跳过：

- `external_texture_from_view_zero_init_after_discard`
- `external_texture_zero_init_after_discard`

这两项在硬件 DX12 实际执行并通过。Lavapipe 全量的全部 55 项跳过名称及原因见 `lavapipe-full-skips.txt`。49 个 harness 跳过按原因分为：19 个 Features、2 个 Limits、13 个 BACKEND、2 个 BACKEND|ADAPTER、13 个 VENDOR；另 6 个 OOM 测试由仓库默认配置排除。

## 临时漏清零故障注入

隔离 worktree：`target/issue-8518-windows/fault`，HEAD 同为候选提交。未在主源码树注入。

故障仅位于 `BakedCommands::initialize_texture_memory`：对 label 为 `texture binding init` 的纹理跳过 `clear_texture`，仍保留原先初始化 tracker 的处理。确切补丁见 `fault-injection.patch`。

| 状态               | 用例                   | 实际执行 / 通过 / 失败 | 证据                                 |
| ------------------ | ---------------------- | ---------------------- | ------------------------------------ |
| 注入前候选         | dropped_command_buffer | 1 / 1 / 0              | lavapipe-targeted-ready.log          |
| 注入后隔离副本     | dropped_command_buffer | 1 / 0 / 1              | fault-injected.log，nextest exit 100 |
| 恢复隔离副本源码后 | dropped_command_buffer | 1 / 1 / 0              | fault-restored.log，exit 0           |
| 主候选后续全量     | 含全部七项回归         | 全部通过               | lavapipe-full.log                    |

注入后的断言在 `texture_binding.rs:421` 失败：首次 mip=0、layer=0、x=0、y=0，期望 `0`，实际 `2155905152 = 0x80808080`。失败源是纹理读回内容错误；该日志没有 VUID，不能将其解释为仅捕获了 Vulkan 布局验证错误。这也实证了本机 Lavapipe 的 poison-memory 生效。

随后用 `git restore --source=f47afd2b6aa8872eb283e808363e180f3930e9e3 -- wgpu-core/src/command/memory_init.rs` 恢复隔离副本；主树与恢复副本的文件 SHA-256 一致，二者 git status 均为空。故障补丁作为诊断材料保存在 target 中，交付源码不含故障或探针。

## Windows 诊断与限制

本次没有候选测试失败。准备阶段失败及运行日志异常逐项如下，均依据本机证据处理：

1. **工具链别名下载冲突/停滞。** 初次隐式解析仓库 `1.95` 别名触发 rustup 下载，出现 clippy 下载文件重命名失败。改为进程级 `RUSTUP_TOOLCHAIN=1.95.0-x86_64-pc-windows-msvc` 使用本机已经安装的同版本；最终全部测试使用该版本。已停止初次遗留的版本查询进程。
2. **静态 DXC 网络失败。** `build-tests.log` 中 mach-dxcompiler 构建脚本 curl exit 56，后续重试仍受直连问题影响。核实系统代理为本机 127.0.0.1:7897 后，仅在任务目录 `.curlrc` 和测试进程 Cargo 配置中使用代理，构建成功（`build-tests-proxy.log`）。没有更换依赖版本。
3. **离线 CPU 启动失败。** `lavapipe-cpu.log` 为 nextest metadata 缺 `az 1.3.0`，并未执行测试，不计通过。联网准备依赖后，直接 CPU 测试以及正式 xtask CPU 测试均实际通过。第一次 xtask metadata 下载过慢被主动终止，记录为 `lavapipe-targeted.log` 的准备失败；成功重跑为 `lavapipe-targeted-ready.log`。
4. **空驱动变量导致无 Vulkan driver。** 准备恢复硬件环境时，PowerShell/.NET 把空变量保留在环境中，造成 `vkCreateInstance: Found no drivers`。调试日志证实驱动覆盖变量为空；改为 `Remove-Item Env:...` 真正删除后，vulkaninfo 立即恢复枚举 Intel。无 adapter 的尝试不计测试成功。
5. **Tracy `SymInitialize FAILED ... 87`。** 源头已核实为 `tracy-client-sys 0.30.0` 的 `TracyCallstack.cpp:685`，由 `--all-features` 启用的基准 Tracy 客户端调用 DbgHelp。四个 DX12 render 测试正常路径均通过，但各打印一次该诊断；设置其已支持的 `TRACY_SYMBOL_OFFLINE_RESOLVE=1` 后，四项再次通过且消息数由 4 降为 0。它影响 profiling 符号解析，不是 texture 输出断言失败；未进一步断言 Windows API 返回 87 的底层原因。该环境选项仅用于单独诊断，主验证未靠禁用校验来通过。
6. **Vulkan validation 警告。** 全量出现 49 次 `WARNING-VkImageSubresourceRange-layerCount-compatibility`；初次双枚举硬件 Vulkan 扩展运行出现 86 次。消息指出当前未启用 maintenance9，3D image 的 layerCount=1 表示全部 depth slices，提醒启用 maintenance9 后语义会改变。它是前向兼容 warning；本次输出断言通过。候选相对指定基线的 `wgpu-hal/src/vulkan` diff 为空（仅源码比较，未运行基线）。全量另有 15 次 `VALIDATION-SETTINGS` 和 30 次 `WARNING-Setting-Limit-Adjusted`，来自测试主动启用 GPU-assisted validation 时的组合检查及 feature 调整提示，未导致用例失败。
7. **Windows 换行造成 snapshot 工作区状态变化。** 全量 snapshot 生成后有 905 个 `naga/tests/out` 文件显示修改；保存的普通 diff、numstat、忽略行末空白的 diff 均为空，仅有 LF/CRLF 警告。确认全部来自本次生成且初始干净后，恢复了这一区域。最终工作区干净，未把生成内容作为代码修复交付。

没有沿用 Mac 的失败结论：Windows Lavapipe 的 `subgroup_operations` 实际执行并通过；DXIL/HLSL passthrough 在 Vulkan 路径按 backend 规则跳过，已计入跳过，不宣称这些测试在 Vulkan 通过，也未把它们误判为缺 DXC。

硬件路径没有 poison-memory 机制；敏感性证据来自 Lavapipe 故障注入。全量范围是仓库标准 `cargo xtask test` 默认原生集合（包括 benches/tests/all-features），保留默认排除 OOM，不包含 CTS、trunk、Linux 实测或额外 doctest/完整 CI 矩阵。

## 日志与复现

- `run.ps1` / `env.ps1`：进程级环境和统一执行器。
- `*.command.json`：每次工作目录、HEAD、完整命令参数、主要环境。
- `*.result.json`：退出状态；`*.summary.json`：扣除语义跳过的计数和完整用例清单。
- `lavapipe-targeted-ready.log`、`lavapipe-zero-init.log`、`lavapipe-full.log`：软件驱动主结果。
- `hardware-vulkan-single.log`、`hardware-dx12.log`：硬件主结果。
- `fault-injected.log`、`fault-restored.log`、`fault-injection.patch`：敏感性验证。
- `dx12-tracy-diagnostic.log`：Tracy 诊断。
- `setup-and-commands.md`：环境准备与执行命令。
- `source-verification.json`、`final-git-status.txt`、`fault-final-status.txt`、`environment-restored.txt`：收尾证据。

所有 driver-selection 变量只设置于子测试进程；父执行环境最终没有 VK/WGPU/LVP/GALLIUM 相关选择变量。机器默认驱动未改变，最终根 `.gpuconfig` 是真实 Intel Vulkan 配置，各阶段历史配置已分别归档。
