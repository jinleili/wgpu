# wgpu #8518：Windows 硬件性能验收（第二轮）

**本报告以用户要求的第二轮完整 A/A、A/B 重测为结论依据。Vulkan 三个采样场景均复现收益；DX12 三个场景的四组配对也全部呈下降方向，其中 mixed views 的收益最明确，shared 和 discard 的收益幅度仍有波动。两后端无采样对照的配对中位数均接近零，未观察到稳定回退。**

2026-09-19，在真实 Intel Arc B390 上完成本轮 **192 个正式进程＋16 个独立预检**。正式测量时间为 16:42:42～16:50:20（UTC+08:00）。每进程 128 次迭代，全部正常完成，实际 adapter、工作量、像素检查及二进制哈希均通过核验。没有跳过、删样或追加样本，没有修改实现或重新构建。

首轮及单独 A/A 复测已放入[历史目录](history/round-1/README.md)。其原始数据保留，不混入本轮统计，也不用于本报告的噪声比较。

## 主要结果

下表为四组配对变化的中位数，使用 `(B/A − 1) × 100%`，负数表示耗时下降。创建合计是假设每轮创建 1000 个 bind group，在同一进程内相加 Create、Encoding、Submit 后计算的成本估算。

| 后端 | 场景 | Encoding＋Submit | 含创建成本 | 本轮判断 |
|---|---|---:|---:|---|
| Vulkan | shared attachments | −15.05% | −11.30% | 四组一致改善，超过本轮漂移 |
| Vulkan | overlapping mixed views | −10.45% | −7.58% | 四组一致改善，超过同场景漂移 |
| Vulkan | discard then sample | −11.17% | −8.78% | 四组一致改善，超过同场景漂移 |
| Vulkan | no sampled textures | −0.86% | −0.71% | 接近零，未分辨出稳定变化 |
| DX12 | shared attachments | −6.85% | −5.98% | 四组下降，部分组与噪声接近，幅度不稳定 |
| DX12 | overlapping mixed views | −10.05% | −8.51% | 四组一致改善，超过同场景漂移 |
| DX12 | discard then sample | −6.81% | −6.24% | 四组下降，末组收益未超过噪声 |
| DX12 | no sampled textures | +0.14% | +0.23% | 接近零，未分辨出稳定变化 |

五项指标的绝对毫秒、全部 A/A 组、全部四组 A/B 及中位数见 [results.md](results.md)。[evidence.json](evidence.json) 包含本轮所有进程的阶段值、48 个组的均值与配对结果、完整执行命令、环境及校验信息。[setup-and-commands.md](setup-and-commands.md) 记录构建和运行命令；[log-excerpts.txt](log-excerpts.txt) 保留必要日志。上述文件可独立跨设备阅读。

## 本轮噪声与收益证据

| 后端 | A/A Encoding＋Submit 漂移范围 | A/A 含创建成本漂移范围 |
|---|---:|---:|
| Vulkan | −6.59%～+5.73% | −6.03%～+4.31% |
| DX12 | −5.16%～+4.96% | −5.03%～+4.70% |

A/A 比较中间两次与首尾两次的均值。这些是本轮经验漂移，不是置信区间或噪声上界。判断同时考虑同场景 A/A、后端整体漂移和四组方向的一致性，不把中位数当作每组均能达到的收益。

### Vulkan

三个采样场景的 12 个 Encoding＋Submit 配对全部下降，最小下降 8.96%，超过本轮后端最大绝对组合漂移 6.59%。含创建的 12 个配对也全部下降，每场景最小收益均大于各自 A/A 的合计漂移。含创建中位数下降 7.58%～11.30%，本轮支持保留当前优化。

收益主要来自 Encoding，shared / mixed / discard 中位数分别为 −15.69% / −10.99% / −11.95%。Submit 分别为 −3.19% / +5.02% / +5.30%；discard 四组 Submit 均上升 2.61%～7.82%，是观测到的阶段成本，不应称所有阶段都加速。其绝对量约 0.11～0.12 ms，组合成本仍下降。

创建成本中位数分别为 −0.40% / +1.07% / −0.90%，没有稳定创建收益。discard 第三组创建上升 16.78%，其中一个 B 为 0.9782 ms，另一个为 0.8118 ms，两个 A 为 0.7493 / 0.7835 ms。该组完整保留，含创建收益因此缩小到 4.99%，但组内仍下降。

### DX12

三个采样场景的 12 个配对在 Encoding＋Submit 和含创建成本上都下降。mixed 的四组组合变化为 −10.06%、−10.03%、−11.21%、−5.44%，均大于同场景 A/A 最大漂移 3.10%；含创建四组下降 4.12%～9.49%，也超过同场景合计漂移 2.54%。该场景的收益证据最明确。

shared 四组组合变化为 −15.14%、−8.27%、−5.44%、−4.33%。中位数下降 6.85%，但最后一组与本场景 A/A 最大漂移 4.96% 接近。discard 四组为 −8.52%、−7.36%、−6.26%、−0.62%，最后一组落在本场景约 5.16% 的漂移内。**本轮支持这两个场景的改善方向，但不能认定收益幅度稳定；mixed 的结论更强。**

shared 和 discard 的收益随组号减小，可能涉及时间、运行顺序或状态变化；没有持续频率、温度或调度轨迹，不能确认根因。discard 最后一组中，一个 B 的组合耗时为 3.7501 ms，另一个为 4.0069 ms，两个 A 为 3.8683 / 3.9370 ms；后一 B 的 Encoding 和 Submit 均偏高，组内收益因此接近零。该组数据未被剔除。

DX12 的 Encoding 中位数分别为 −7.17% / −10.44% / −7.16%，Submit 为 +1.53% / +2.60% / +3.07%，收益同样主要来自 Encoding。mixed 创建成本四组均上升 2.55%～5.52%，中位数 +3.09%，高于该场景 A/A 创建漂移的绝对幅度 1.35%；这是需要记录的创建成本上升信号。没有修改创建实现或加入探针归因；计入该成本后，mixed 四组总成本仍改善。

### 无采样对照

Vulkan 四组组合变化为 −1.07%、−2.66%、−0.65%、+6.94%，DX12 为 −1.23%、+1.51%、−2.52%、+7.93%。中位数分别为 −0.86% 和 +0.14%，含创建中位数为 −0.71% 和 +0.23%。**本轮对照整体接近零，没有一致的回退方向，也没有明确改善。**

第四组的正向变化主要来自 Encoding：Vulkan +6.97%、DX12 +8.37%。两者该组均为 BAAB，B 位于首尾；同场景 A/A 两组均表现为中间快于首尾，因此存在运行位置效应的可能。Vulkan A/A 第一组 −6.59% 换算首尾相对中间约为 +7.06%，与第四组 +6.94% 接近。DX12 第四组仍超过同场景已观察到的位置差异，不能完全由此解释。这仅说明数据与位置或状态波动相容，并非证明根因。

空 texture-init 列表的 bind group 会提前返回，但对照仍有输出附件，经过公共 first-use tracking 路径；公共路径成本也不能排除。由于四组没有一致回退，本轮不判定稳定回退；第四组变化仍完整记录，接近零的中位数不代表每次运行都不变。

## 执行和统计完整性

| 后端 | A/A 进程 | A/B 进程 | 正常完成 | 失败 | 跳过 | 另计预检 |
|---|---:|---:|---:|---:|---:|---:|
| Vulkan | 32 | 64 | 96 | 0 | 0 | 8 |
| DX12 | 32 | 64 | 96 | 0 | 0 | 8 |

每场景每版本单独启动进程，使用 `--bench --exact <完整名称> --iters 128 --color never`，以唯一 `--save-baseline r2-...` 保存结果。保留 8 次预热、计时前像素读回、1000 个组、10 个消费 pass、10000 draws，采样场景共享 16 对纹理。A/A 每场景两组 AAAA；A/B 每场景四组，交替 ABBA、BAAB。沿用固定种子 20260919 的场景顺序，测量前保存本轮 schedule 及哈希。

正式进程串行执行，每进程间隔 1 秒，计时期间没有并行构建、其他基准或本任务 profiler 采样。阶段前可读进程 CPU 总和约占全机容量的 1.69%～2.57%；用户应用保持运行，未调整永久电源或优先级。抽查不代表全过程完全空闲，持续热状态和抢占轨迹未记录。

独立 PowerShell 计算从原始 JSON 复核了全部 240 个组指标。确认 192 个唯一 run ID、完整迭代数与工作量、退出码和 adapter；二进制及 schedule 未变，历史原始文件哈希也未变。完整原始数据和脚本位于忽略目录 `target/issue-8518-windows-performance/full-round-2/`。

组合指标先在同一进程内相加，再计算版本组均值和配对百分比，没有相加阶段百分比。绝对值表采用四个版本组均值的中位数，变化列采用四个配对百分比的中位数，两者不必代数一致。128 次迭代形成一个进程均值，不是 128 个独立样本。CPU API 墙钟时间不等于 GPU 时间、CPU cycle 或应用帧率，GPU wait 和像素读回不在计时内。创建合计只是给定创建频率的估算，不是创建后立即使用的真实流程测量。

## 与 Metal 的比较及判断

[固定 Metal 报告](https://github.com/jinleili/wgpu-in-app/blob/68ad306a05e65ba5708fd494e1ef86fbbec22b1a/wgpu-in-app/examples/issue_8518_texture_render_result.md) 的 shared / mixed / discard 组合中位数为 −3.65% / −5.96% / −4.72%。本轮 Windows 两后端呈相同改善方向。这里只比较趋势，不跨机器比较绝对毫秒；Windows 正确性状态以[后续正确性报告](../issue-8518-windows/README.md)为准。

1. **是否复现收益？** Vulkan 三个采样场景明确复现；DX12 mixed 明确，shared/discard 四组方向一致但幅度不够稳定。
2. **计入创建后？** 三个采样场景的每个配对仍下降。Vulkan 中位数下降 7.58%～11.30%，DX12 为 5.98%～8.51%；DX12 部分组仍落在本轮噪声范围内。
3. **对照是否回退？** 中位数接近零，没有稳定回退证据；第四组 Encoding 上升已单独分析。
4. **哪些超过噪声？** Vulkan 三个场景的组合收益以及 DX12 mixed 最明确。DX12 shared/discard 部分组、单独 Submit/创建的小幅变化和对照仍有不确定性；mixed 创建成本上升已计入总成本。
5. **实现方向是否改变？** 继续保留当前局部 folding，暂不扩大完整 tracker folding。本轮支持现有优化的价值，扩大实现仍需新的热点与收益证据。

本次仅更新报告和忽略目录内的工具、数据，没有修改生产实现，没有运行完整基线测试或 CTS，没有提交、推送或创建 PR。


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

预检实际模块列表确认两版本加载相同动态 DXC、Agility 和 Intel 驱动。DX12 枚举过程同时加载系统 `d3d10warp.dll` 和系统 D3D12Core，这不代表选中了 WARP；实际选择以每进程 adapter 输出和 Intel 驱动模块为依据。指定 Agility 通过 Independent Devices API 加载，`REQUIRE=1` 禁止静默回退。未看到 SDKLayers、RenderDoc、PIX 或 Tracy 模块。系统虽注册了 RenderDoc Vulkan 隐式层，但本次已移除 capture 开关并禁用 layers；测量后的独立 loader 诊断只显示 Application → Loader → Driver，未插入附加层。Direct3D 控制配置为 app-controlled debug/GBV，force-warp=false，无应用覆盖项。
