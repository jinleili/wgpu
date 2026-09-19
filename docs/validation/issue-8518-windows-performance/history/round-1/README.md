> 历史快照：当前主结论见 [第二轮完整报告](../../README.md)。本目录的数据不混入第二轮统计。

# wgpu #8518：Windows 硬件性能验收

后续复测：用户另行要求的一轮 DX12 A/A 已完成，**32 个进程未复现约 30% 尖峰**；原异常组漂移为 −1.43%，全部八组范围 −3.59%～+4.26%。详见 [DX12 A/A 第二轮报告](dx12-aa-repeat-1.md)及[完整数据](dx12-aa-repeat-1.json)。以下原始 A/B 报告及样本保留，本轮未重新测量 B。

2026-09-19，在 Intel Arc B390 上完成 **192 个正式进程**（每后端 32 个 A/A、64 个 A/B）及另计的 16 个预检。所有进程均正常结束，实际选中 Intel 硬件，每个正式进程执行指定场景的三项指标各 128 次，原像素检查没有失败。未跳过、未替换软件 adapter、未修改生产实现，未追加或删除正式样本。

**Vulkan 的三个采样纹理场景稳定呈现收益，计入创建成本后仍成立。DX12 也呈一致下降趋势，但本次 A/A 出现未解释的大幅尖峰，不能通过整个后端的稳定收益验收。两个后端的无采样对照均有小幅正向中位数及较大单组回退，尚不能排除公共路径成本。**

## 主要结果

百分比为四组 `(B/A - 1) × 100%` 的中位数，负数表示耗时下降。这里的“总成本”是假设每轮创建 1000 个 bind group，将同一进程的 Create、Encoding、Submit 耗时相加的估算，不是实际每帧创建并立即使用的完整流程。

| 后端 | 场景 | Encoding + Submit | 含创建的总成本 | 判断 |
|---|---|---:|---:|---|
| Vulkan | shared attachments | −14.36% | −10.96% | 四组均改善，超过观测漂移 |
| Vulkan | overlapping mixed views | −11.64% | −9.17% | 四组均改善，超过观测漂移 |
| Vulkan | discard then sample | −12.72% | −9.27% | 四组均改善，超过观测漂移 |
| Vulkan | no sampled textures | +2.08% | +2.51% | 未分辨出改善；存在回退风险，原因未定 |
| DX12 | shared attachments | −4.13% | −4.60% | 小幅下降趋势，被 A/A 噪声覆盖 |
| DX12 | overlapping mixed views | −9.63% | −8.34% | 超过同场景 A/A 漂移，但未超过后端异常幅度 |
| DX12 | discard then sample | −11.04% | −9.97% | 超过同场景 A/A 漂移，但未超过后端异常幅度 |
| DX12 | no sampled textures | +2.91% | +2.56% | 未分辨出改善；存在回退风险，原因未定 |

五项指标的绝对毫秒、每个 A/A 组、全部四组 A/B 变化及中位数均在 [results.md](results.md)。完整命令和公平构建检查在 [setup-and-commands.md](setup-and-commands.md)。[evidence.json](evidence.json) 包含全部 192 个正式进程的阶段值、48 个组的均值与变化、环境、版本和校验信息；[log-excerpts.txt](log-excerpts.txt) 包含必要运行证据。文件均可独立复制到其他设备阅读，不依赖 Windows `target` 中的原始日志。

## 版本与运行环境

| 项目 | 实际配置 |
|---|---|
| A | `f12c3e4508cb706c1e1faeb973bdb9c6970f5e15`，只覆盖为 B 的完整 `benches/` |
| B | `f47afd2b6aa8872eb283e808363e180f3930e9e3` |
| 主工作区 HEAD | `8c91d7a80a814d8641b2b69862e4ae37a3e11a1b`，仅增加此前正确性文档 |
| CPU | Intel Core Ultra X7 358H，16 核 / 16 逻辑处理器，x86-64 |
| GPU | Intel Arc B390，Vulkan 类型为 integrated GPU，PCI vendor/device `8086:b080` |
| 驱动 | Intel `32.0.101.8860`，日期 2026-06-25；Vulkan API `1.4.348`、driver `101.8860` |
| Windows | Windows 11 家庭版中文版 25H2，`26200.9168` |
| Rust / Cargo | `rustc 1.95.0 (59807616e)` / `cargo 1.95.0 (f2d3ce0bd)`，MSVC x64，LLVM 22.1.2 |
| MSVC | VS 18 Community，toolset `14.51.36231`，cl `19.51.36252.0`，link `14.51.36252.0` |
| 电源 | ACLineStatus=1，接电；BatteryFlag=128（无系统电池）；平衡方案，无有效 overlay 覆盖 |
| 并行度 | `RAYON_NUM_THREADS=8`；正式基准逐进程串行，正常优先级 |
| Vulkan runtime | 系统 `vulkan-1.dll 1.4.313.0`，Intel `igvk64.dll 32.0.101.8860` |
| DX12 runtime | `WGPU_DX12_COMPILER=Dxc`；动态 DXC `1.9.2602.24 (d355aa836)`；Agility `1.619.0.0.20260223.5`，SDK version 619，要求加载成功 |
| 调试设施 | WGPU DEBUG / VALIDATION / GPU_BASED_VALIDATION 均为 0；Vulkan layers 禁用；没有 profiler 或 allocator 探针 |

固定二进制 SHA-256：

```text
A 498d4c3574ff18d6ae434018cc6b8974a32308bc6966f8b8b3a243bb6a40fdc9
B a69e894391c0b6e56baaeeeeb9a5acdb4184b16ae8f0228889bca4d1ab107821
```

构建和正式运行分开。A/B features 树、Cargo.lock、benchmark 文件内容及编译设置一致，生产差异仅为指定的三个 wgpu-core 文件。测量后再次校验二进制和预先保存的 schedule 哈希，均未改变。

每个正式进程都打印 `Using adapter: Intel(R) Arc(TM) B390 GPU (Vulkan)` 或 `(Dx12)`。Vulkan 枚举产生两个条目，但设备 UUID、vendor/device 和驱动相同，均为该 Intel 硬件。系统还有 GameViewer 虚拟显示设备，未选用。没有 Lavapipe/llvmpipe，也没有使用 WARP 作为测量 adapter。

预检实际模块列表确认两版本加载相同动态 DXC、Agility 和 Intel 驱动。DX12 枚举过程同时加载系统 `d3d10warp.dll` 和系统 D3D12Core，这不代表选中了 WARP；实际选择以每进程 adapter 输出和 Intel 驱动模块为依据。指定 Agility 通过 Independent Devices API 加载，`REQUIRE=1` 禁止静默回退。未看到 SDKLayers、RenderDoc、PIX 或 Tracy 模块。系统虽注册了 RenderDoc Vulkan 隐式层，但本次已移除 capture 开关并禁用 layers；loader 预检只显示 Application → Loader → Driver，未插入附加层。Direct3D 控制配置为 app-controlled debug/GBV，force-warp=false，无应用覆盖项。

## 噪声、阶段成本和异常分析

### Vulkan

A/A 的 Encoding + Submit 漂移为 **−2.00% 至 +2.97%**，创建合计为 **−1.25% 至 +2.67%**。三个采样场景的 12 个 A/B 配对，Encoding + Submit 均下降 10.05%～16.87%，含创建成本均下降 7.18%～13.21%。下降方向一致且每组幅度均超过本后端观测漂移，因此在本次固定工作量和环境下判为稳定改善。

收益主要来自 Encoding（各场景中位数 −15.21%、−12.15%、−13.32%）。Submit 中位数反而为 +2.40%、+3.94%、+2.64%，绝对量约 0.09～0.11 ms；该阶段 A/A 漂移最大达 10.61%，不应宣称 Submit 加速。创建成本中位数为 −0.94%、−0.63%、+2.95%；discard 场景四组创建均小幅上升，但同场景 A/A 曾有 +8.36%，不能据此确认独立创建路径回退。

无采样对照不能宣布“无回退”：Encoding + Submit 四组为 **+0.74%、−1.06%、+3.41%、+10.81%**。第四组主要由 Encoding 增加 11.06% 导致，Submit 增加 5.04%，创建增加 6.44%。该组两个 B 的 Encoding 分别约 1.378 和 1.451 ms，两个 A 约 1.286 和 1.261 ms，异常不只是一个 B 进程。此组超过对照 A/A 的 +2.97%，必须保留；但其他组差异较小且有反向结果，中位数 +2.08% 落在观测漂移量级，不能将第四组外推为稳定 11% 回退。

### DX12

shared attachments 第二组 A/A 的 Encoding + Submit 为 **+30.19%**，创建合计 **+27.38%**。其中 `dx12-aa-g2-s1-p2-a` 的 Encoding 约 4.33 ms、Submit 约 0.81 ms；相邻同一 A 二进制进程约为 Encoding 3.00～3.34 ms、Submit 0.11～0.13 ms。组级 Encoding 漂移 +20.10%、Submit +307.44%，说明这是实际计时阶段的尖峰，不能归结为程序启动时间。

在 A/B 前已检查：本任务没有 cargo/rustc/link 或其他 benchmark 并行；电源方案保持平衡；阶段前 3 秒可读进程 CPU 总和约 1.94%（全机容量）。这些事后低负载观测不能解释或排除先前短时抢占、驱动或电源状态扰动。温度 WMI 查询拒绝访问，未提升权限；标称时钟读数不能作为实时每核 turbo/热状态证据。没有可靠定位并控制这个尖峰的办法，因此未删样、未换协议、未为“显著”反复重跑。

三个采样场景的四组 A/B 均下降。mixed 和 discard 的组合收益分别约 9.63% 和 11.04%，明显超过各自 A/A 的约 1.30% 和 1.99%；这是有一致性的局部证据。但它们仍小于同一后端实际出现的 30.19% 扰动，且不能证明这种扰动只影响 shared。**保守验收结论为：DX12 有改善趋势，shared 当前无法分辨可靠收益，后端整体稳定复现尚未确认。** 不是宣称 DX12 folding 无效。

DX12 的收益也集中在 Encoding；Submit 的三个中位数为 +4.65%、+2.47%、−0.36%，没有可靠的加速证据。shared 创建成本第三组 −15.84% 与其余三组 +0.72%～+1.97% 不一致：A 的创建计时存在较高值，创建合计中位数 −4.60% 因而不应解释为创建本身优化。原始值和全部配对均保留。

无采样对照 Encoding + Submit 为 **+3.56%、+2.25%、−0.99%、+6.63%**，中位数 +2.91%；含创建中位数 +2.56%。对照 A/A 最大绝对漂移约 4.07%，第四组超过它，但没有四组一致的回退。应记录回退风险，不能判定稳定回退，也不能判定零回退。

### 对照公共路径与限制

源码检查发现：B 对空 texture-init 列表的 bind group 提前返回，不会分配 bind-group BitVec；但对照仍有输出 color attachment，其初始化经过新增 first-use tracking。命令缓冲对象也新增状态。因此公共路径开销、编译后的代码布局和分配行为仍可能影响对照，不能因为“无采样纹理”就认定所有正向变化都是噪声。两个后端均有约 +2.5% 的含创建中位数，值得后续针对公共路径诊断；本次定位到阶段与相关代码路径，没有足够证据给出因果归属，也没有擅自修改候选或加入探针后继续冒充原候选结果。

这些数据是 CPU 侧 API 墙钟时间，不是 GPU 时间、CPU cycle 或应用帧率；GPU wait 和像素读回在计时外。只覆盖该固定微基准，不代表一般应用。后台用户应用保持运行，没有改永久电源设置或提升优先级；四个阶段前可读进程 CPU 总和约 1.81%～3.59%，不等于全过程系统空闲。没有连续热状态、每核频率或抢占轨迹。A/A 每场景仅两组、A/B 仅四组，观测漂移不是统计置信区间，也不是噪声上界。128 次迭代只产生一个进程均值，未按 128 个独立样本计算显著性。

## 与 Metal 的关系及最终判断

[固定版本的 Metal 报告](https://github.com/jinleili/wgpu-in-app/blob/68ad306a05e65ba5708fd494e1ef86fbbec22b1a/wgpu-in-app/examples/issue_8518_texture_render_result.md) 在 shared / mixed / discard 的 Encoding + Submit 中位数为 −3.65% / −5.96% / −4.72%。Windows Vulkan 的改善方向复现且本次相对幅度更大；DX12 方向也相同，但环境稳定性限制了结论。此处只比较趋势，不能跨机器比较绝对毫秒或据此宣称某后端本质上更快。Metal 报告中的“Windows 尚未验证”已过时；正确性状态以[后续 Windows 正确性报告](../../../issue-8518-windows/README.md)为准。

逐项回答：

1. **是否复现收益？** Vulkan 三个采样场景是；DX12 有一致下降趋势，但按本次后端 A/A 异常幅度，尚不能确认整体稳定复现，shared 尤其不确定。
2. **计入创建成本后？** Vulkan 仍有约 9.17%～10.96% 的中位数收益；DX12 观测值仍下降约 4.60%～9.97%，受同样稳定性限制。创建合计只是给定创建频率的估算。
3. **对照是否回退？** 两后端观测到正向中位数及明显单组回退，不能保证无回退；证据不足以判定稳定回退，已定位到主要为 Encoding，公共路径成本仍待因果确认。
4. **哪些超过噪声？** Vulkan 三个采样场景的 Encoding、Encoding+Submit、创建合计均有一致且超过观测漂移的改善；Submit、单独创建和对照的小幅变化不确定。DX12 mixed/discard 超过同场景漂移，但不能消除后端级异常带来的不确定性。
5. **是否改变实现方向？** 不改变“保留当前局部 folding、暂不扩大完整 tracker folding”。Vulkan 结果支持现有优化的价值，DX12 趋势提供补充证据，但本次不能作为“所有 Windows 场景均无回退”的无条件性能验收。对照风险和 DX12 稳定性应先得到更明确解释，再考虑扩大实现范围。

主工作区仅新增本报告目录，已有修改未被覆盖；候选生产源码、固定版本、二进制保持不变。临时工具、完整原始 JSON/日志和脚本保存在 `target/issue-8518-windows-performance/`，未进入交付源码。
