# DX12 A/A 第二轮：30.19% 尖峰复现检查

2026-09-19，用户明确要求追加一轮，实际运行时间为本地 16:30:45～16:31:50（UTC+08:00）。

**本轮未复现约 30% 的尖峰，不支持“30.19% 会稳定出现”。** shared attachments 两组 Encoding + Submit 漂移为 **+0.88%、−1.43%**；四场景全部八组为 **−3.59%～+4.26%**。这与上轮 +30.19% 的幅度明显不同，更符合非持续性扰动的表现，但仍未确定上次尖峰的原因，不能证明它不会再次出现。

## 测量与验证

完整重跑四个场景的 A/A，每场景两组 AAAA，共 **32 个独立进程**，每进程 **128 次迭代**，保留全部结果。继续使用第一次构建的同一个 A 二进制，没有重新构建。A 为 `f12c3e4508cb706c1e1faeb973bdb9c6970f5e15`，benchmark 源码仍为 B 的 benches 覆盖；二进制 SHA-256：

`498d4c3574ff18d6ae434018cc6b8974a32308bc6966f8b8b3a243bb6a40fdc9`。

复用原 DX12 A/A 场景顺序（原随机种子 20260919），每进程间隔 1 秒、8 次预热、计时前像素读回、1000 个组/10 个消费 pass/10000 draws 的工作量均不变。驱动仍为 Intel 32.0.101.8860，每个进程实际打印 Intel Arc B390 (Dx12)；DXC、Agility、调试开关、RAYON_NUM_THREADS=8 均沿用原配置。32 个进程退出码均为 0，全部输出一个指定场景、三项指标各 128 次，没有像素断言或设备错误。

开始前没有其他构建或 benchmark 进程，平衡电源方案未变；3 秒可读进程 CPU 总和为约 2.76% 的全机容量（包括观测脚本自身约 1.04%）。没有关闭用户应用、改变优先级或电源设置，也没有在正式进程中附加 profiler。未记录连续温度或调度轨迹，故不能归因到某个后台进程或热状态。

本轮仅回答尖峰复现问题，不是第二轮 A/B 性能验收。原 192 个正式进程的所有原始文件哈希保持不变，原始 A/A、A/B 和结论保留；新增结果单独存储，没有拿本轮 A/A 替换上轮噪声数据。

## 全部组合耗时配对

仍计算 `(中间两次均值 / 首尾两次均值 − 1) × 100%`。这是同一 A 的测量漂移，**不是 B 相对 A 的性能变化**。

| 场景 | 组 | 上轮漂移 | 本轮漂移 | 本轮首尾均值 ms | 本轮中间均值 ms |
|---|---:|---:|---:|---:|---:|
| shared attachments | 1 | +3.06% | +0.88% | 3.4483 | 3.4786 |
| shared attachments | 2 | +30.19% | -1.43% | 3.5428 | 3.4922 |
| overlapping mixed views | 1 | -0.26% | -1.95% | 3.4378 | 3.3708 |
| overlapping mixed views | 2 | -1.30% | -0.38% | 3.4444 | 3.4313 |
| discard then sample | 1 | -1.99% | -0.52% | 3.9806 | 3.9600 |
| discard then sample | 2 | -0.86% | -3.59% | 4.0314 | 3.8866 |
| no sampled textures | 1 | -1.15% | +4.26% | 1.4442 | 1.5056 |
| no sampled textures | 2 | -4.07% | -0.56% | 1.4608 | 1.4526 |

## 单进程尖峰核对

仅观察组均值可能抵消个别进程异常，因此也检查每场景全部 8 个进程的 Encoding + Submit。

| 场景 | 上轮中位数 / 最大值 ms | 本轮中位数 / 最大值 ms | 本轮最大值高于本轮中位数 |
|---|---:|---:|---:|
| shared attachments | 3.3016 / 5.1446 | 3.4900 / 3.6229 | +3.81% |
| overlapping mixed views | 3.1315 / 3.1582 | 3.4232 / 3.4668 | +1.27% |
| discard then sample | 3.5897 / 3.7240 | 3.9451 / 4.1316 | +4.73% |
| no sampled textures | 1.4003 / 1.5724 | 1.4467 / 1.6115 | +11.39% |

原异常位置 shared G2/P2：上轮 Encoding **4.3301 ms**、Submit **0.8145 ms**；本轮同位置分别为 **3.3403 ms**、**0.1250 ms**。上轮 shared 单进程最大组合耗时为 5.1446 ms，比该轮该场景中位数高 55.82%；本轮最大 3.6229 ms，仅高 3.81%。本轮所有场景的单进程组合耗时最大相对场景中位数偏离为 +11.39%（无采样对照），没有同量级的单进程组合尖峰。

两轮整体水平也并非完全相同：shared / mixed / discard / control 的 A 进程组合耗时中位数相对上轮分别变化 +5.71% / +9.31% / +9.90% / +3.31%。这说明局部四进程组较平稳，不等于跨时段绝对耗时固定；不能直接跨两轮拼接 A 与 B。

## 全部阶段漂移

| 场景 | 阶段 | 上轮 G1 | 上轮 G2 | 本轮 G1 | 本轮 G2 |
|---|---|---:|---:|---:|---:|
| shared attachments | Encoding | +3.09% | +20.10% | +0.79% | -1.53% |
| shared attachments | Submit | +2.13% | +307.44% | +3.36% | +1.26% |
| shared attachments | Encoding + Submit | +3.06% | +30.19% | +0.88% | -1.43% |
| shared attachments | Create bind groups | +6.64% | +6.69% | -0.38% | -0.86% |
| shared attachments | Create + Encoding + Submit | +3.49% | +27.38% | +0.73% | -1.36% |
| overlapping mixed views | Encoding | -0.15% | -1.39% | -1.88% | -0.38% |
| overlapping mixed views | Submit | -3.96% | +1.78% | -4.09% | -0.41% |
| overlapping mixed views | Encoding + Submit | -0.26% | -1.30% | -1.95% | -0.38% |
| overlapping mixed views | Create bind groups | -0.58% | -1.11% | +2.94% | -2.29% |
| overlapping mixed views | Create + Encoding + Submit | -0.30% | -1.28% | -1.37% | -0.61% |
| discard then sample | Encoding | -2.04% | -0.81% | -0.25% | -3.57% |
| discard then sample | Submit | -0.59% | -2.38% | -7.68% | -4.33% |
| discard then sample | Encoding + Submit | -1.99% | -0.86% | -0.52% | -3.59% |
| discard then sample | Create bind groups | -3.93% | -3.52% | +0.14% | -1.46% |
| discard then sample | Create + Encoding + Submit | -2.20% | -1.16% | -0.45% | -3.37% |
| no sampled textures | Encoding | -1.98% | -4.30% | +4.36% | -0.64% |
| no sampled textures | Submit | +21.20% | +2.58% | +1.61% | +1.30% |
| no sampled textures | Encoding + Submit | -1.15% | -4.07% | +4.26% | -0.56% |
| no sampled textures | Create bind groups | +0.39% | +1.08% | -0.10% | -3.91% |
| no sampled textures | Create + Encoding + Submit | -0.98% | -3.51% | +3.76% | -0.94% |

## 对原结论的影响

本轮降低了“DX12 在相同协议下会持续产生约 30% 尖峰”的疑虑，但只增加了一轮 A/A，不能确定异常概率或根因。原 shared A/B 中位数下降约 4.13%，与本轮后端最大组合漂移 4.26% 仍在同一量级；mixed/discard 的原下降约 9.63%/11.04% 大于本轮局部漂移，改善趋势更可信，但没有新增 B 样本，原性能数据及保守限制保持。两后端对照回退风险也不因本次 A/A 而消失。

## 命令与数据

```powershell
$r = "D:/rust/forks/wgpu/target/issue-8518-windows-performance"
& "$r/health.ps1" -Label before-dx12-aa-repeat1
& "$r/run-dx12-aa-repeat1.ps1" -Backend dx12 -Phase aa
node "$r/analyze-dx12-aa-repeat1.cjs"
node "$r/report-dx12-aa-repeat1.cjs"
# 每个进程实际命令形如：
& "$r/binaries/a/wgpu-benchmark.exe" --bench --exact "Texture Init Render: shared attachments" --iters 128 --color never --save-baseline dx12-aa-repeat1-g2-s1-p2-a
```

环境设置仍见 [setup-and-commands.md](setup-and-commands.md)。完整 32 条实际命令、顺序、开始结束时间、每进程三个原始阶段值、五项组均值和配对变化、哈希与环境见 [dx12-aa-repeat-1.json](dx12-aa-repeat-1.json)。逐进程原始 JSON/stdout/stderr/meta 均独立保存在忽略目录 `target/issue-8518-windows-performance/dx12-aa-repeat-1/raw/`；唯一 baseline 名避免覆盖上轮结果。源码未改动，未提交或推送。
