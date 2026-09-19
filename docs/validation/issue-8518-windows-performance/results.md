# wgpu #8518 Windows 硬件性能 A/B：第二轮全部结果

本文件仅统计用户要求的第二轮完整重测，不包含历史样本。日期：2026-09-19。单位：每迭代 CPU API 墙钟毫秒。负百分比表示耗时下降。A/B 绝对值为四个组内版本均值的中位数；百分比为四个配对百分比的中位数，因此不能由表中两列中位数重新计算百分比。

每进程先将阶段平均耗时相加，再在组内分别平均两个 A 与两个 B，计算 `(B/A - 1) × 100%`。组合阶段来自相同进程；创建合计是假设每轮创建 1000 个 bind group 的使用频率估算，不是创建后立即使用的实测流程。128 次迭代不是独立样本，独立配对单位为四个进程组成的一组。

## VULKAN

### A/A：同一 A 二进制，中间两次 / 首尾两次 − 1

| 场景 | 阶段 | 第 1 组 | 第 2 组 |
|---|---|---:|---:|
| shared attachments | Encoding | -1.33% | +0.87% |
| shared attachments | Submit | -1.72% | -0.12% |
| shared attachments | Encoding + Submit | -1.35% | +0.82% |
| shared attachments | Create bind groups | +1.42% | -1.41% |
| shared attachments | Create + Encoding + Submit | -0.64% | +0.27% |
| overlapping mixed views | Encoding | +2.97% | +5.58% |
| overlapping mixed views | Submit | +4.45% | +9.89% |
| overlapping mixed views | Encoding + Submit | +3.02% | +5.73% |
| overlapping mixed views | Create bind groups | -3.06% | -0.62% |
| overlapping mixed views | Create + Encoding + Submit | +1.65% | +4.31% |
| discard then sample | Encoding | +1.31% | +0.65% |
| discard then sample | Submit | -0.85% | -4.08% |
| discard then sample | Encoding + Submit | +1.22% | +0.44% |
| discard then sample | Create bind groups | -2.42% | +3.40% |
| discard then sample | Create + Encoding + Submit | +0.37% | +1.11% |
| no sampled textures | Encoding | -6.86% | -2.05% |
| no sampled textures | Submit | -0.44% | -6.80% |
| no sampled textures | Encoding + Submit | -6.59% | -2.27% |
| no sampled textures | Create bind groups | -3.03% | +2.70% |
| no sampled textures | Create + Encoding + Submit | -6.03% | -1.48% |

### A/B：四组全部保留

G1/G3 为 ABBA，G2/G4 为 BAAB。

| 场景 | 阶段 | A ms | B ms | G1 | G2 | G3 | G4 | 配对中位数 |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| shared attachments | Encoding | 2.2376 | 1.8855 | -16.14% | -15.11% | -17.18% | -15.23% | -15.69% |
| shared attachments | Submit | 0.1176 | 0.1138 | -7.45% | -1.04% | -5.34% | +2.57% | -3.19% |
| shared attachments | Encoding + Submit | 2.3519 | 1.9993 | -15.69% | -14.42% | -16.58% | -14.38% | -15.05% |
| shared attachments | Create bind groups | 0.7984 | 0.7966 | -0.28% | -0.53% | -3.84% | +1.20% | -0.40% |
| shared attachments | Create + Encoding + Submit | 3.1481 | 2.7923 | -11.73% | -10.87% | -13.41% | -10.49% | -11.30% |
| overlapping mixed views | Encoding | 2.6611 | 2.3757 | -9.63% | -15.99% | -11.62% | -10.35% | -10.99% |
| overlapping mixed views | Submit | 0.0920 | 0.0974 | +10.66% | -9.03% | +4.79% | +5.26% | +5.02% |
| overlapping mixed views | Encoding + Submit | 2.7532 | 2.4732 | -8.96% | -15.73% | -11.07% | -9.83% | -10.45% |
| overlapping mixed views | Create bind groups | 0.7820 | 0.8029 | -0.71% | -0.53% | +2.66% | +8.54% | +1.07% |
| overlapping mixed views | Create + Encoding + Submit | 3.5267 | 3.2758 | -7.12% | -12.48% | -8.03% | -5.83% | -7.58% |
| discard then sample | Encoding | 2.6172 | 2.3062 | -12.00% | -11.94% | -11.97% | -11.61% | -11.95% |
| discard then sample | Submit | 0.1141 | 0.1207 | +6.73% | +2.61% | +7.82% | +3.86% | +5.30% |
| discard then sample | Encoding + Submit | 2.7319 | 2.4243 | -11.22% | -11.34% | -11.13% | -10.95% | -11.17% |
| discard then sample | Create bind groups | 0.7901 | 0.7936 | -0.04% | -2.23% | +16.78% | -1.77% | -0.90% |
| discard then sample | Create + Encoding + Submit | 3.5224 | 3.2281 | -8.71% | -9.30% | -4.99% | -8.85% | -8.78% |
| no sampled textures | Encoding | 1.3259 | 1.3294 | -1.06% | -2.44% | -0.75% | +6.97% | -0.91% |
| no sampled textures | Submit | 0.0629 | 0.0639 | -1.20% | -7.08% | +1.51% | +6.27% | +0.15% |
| no sampled textures | Encoding + Submit | 1.3888 | 1.3934 | -1.07% | -2.66% | -0.65% | +6.94% | -0.86% |
| no sampled textures | Create bind groups | 0.2674 | 0.2727 | -1.77% | +2.02% | +1.96% | +4.83% | +1.99% |
| no sampled textures | Create + Encoding + Submit | 1.6597 | 1.6661 | -1.18% | -1.93% | -0.23% | +6.60% | -0.71% |

## DX12

### A/A：同一 A 二进制，中间两次 / 首尾两次 − 1

| 场景 | 阶段 | 第 1 组 | 第 2 组 |
|---|---|---:|---:|
| shared attachments | Encoding | +4.90% | -3.13% |
| shared attachments | Submit | +6.75% | -1.60% |
| shared attachments | Encoding + Submit | +4.96% | -3.07% |
| shared attachments | Create bind groups | +2.71% | -5.30% |
| shared attachments | Create + Encoding + Submit | +4.70% | -3.35% |
| overlapping mixed views | Encoding | +3.16% | -0.66% |
| overlapping mixed views | Submit | +1.17% | -8.58% |
| overlapping mixed views | Encoding + Submit | +3.10% | -0.92% |
| overlapping mixed views | Create bind groups | -1.35% | +1.13% |
| overlapping mixed views | Create + Encoding + Submit | +2.54% | -0.67% |
| discard then sample | Encoding | -5.02% | -4.86% |
| discard then sample | Submit | -9.18% | -6.20% |
| discard then sample | Encoding + Submit | -5.16% | -4.91% |
| discard then sample | Create bind groups | +0.10% | -6.14% |
| discard then sample | Create + Encoding + Submit | -4.62% | -5.03% |
| no sampled textures | Encoding | -2.56% | -3.24% |
| no sampled textures | Submit | -0.56% | -4.09% |
| no sampled textures | Encoding + Submit | -2.48% | -3.28% |
| no sampled textures | Create bind groups | -1.32% | -5.49% |
| no sampled textures | Create + Encoding + Submit | -2.36% | -3.53% |

### A/B：四组全部保留

G1/G3 为 ABBA，G2/G4 为 BAAB。

| 场景 | 阶段 | A ms | B ms | G1 | G2 | G3 | G4 | 配对中位数 |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| shared attachments | Encoding | 3.2858 | 3.0502 | -15.40% | -8.69% | -5.65% | -4.61% | -7.17% |
| shared attachments | Submit | 0.1247 | 0.1263 | -8.61% | +2.85% | +0.21% | +3.07% | +1.53% |
| shared attachments | Encoding + Submit | 3.4102 | 3.1764 | -15.14% | -8.27% | -5.44% | -4.33% | -6.85% |
| shared attachments | Create bind groups | 0.4628 | 0.4626 | -1.74% | -1.53% | +2.68% | +1.10% | -0.21% |
| shared attachments | Create + Encoding + Submit | 3.8682 | 3.6370 | -13.56% | -7.47% | -4.48% | -3.68% | -5.98% |
| overlapping mixed views | Encoding | 3.2891 | 2.9456 | -10.47% | -10.42% | -11.47% | -5.91% | -10.44% |
| overlapping mixed views | Submit | 0.1039 | 0.1058 | +2.99% | +2.21% | -2.97% | +9.66% | +2.60% |
| overlapping mixed views | Encoding + Submit | 3.3922 | 3.0514 | -10.06% | -10.03% | -11.21% | -5.44% | -10.05% |
| overlapping mixed views | Create bind groups | 0.4617 | 0.4744 | +2.55% | +2.94% | +3.24% | +5.52% | +3.09% |
| overlapping mixed views | Create + Encoding + Submit | 3.8538 | 3.5257 | -8.55% | -8.48% | -9.49% | -4.12% | -8.51% |
| discard then sample | Encoding | 3.7412 | 3.4821 | -8.86% | -7.74% | -6.57% | -1.15% | -7.16% |
| discard then sample | Submit | 0.1304 | 0.1331 | +1.68% | +3.69% | +2.45% | +14.69% | +3.07% |
| discard then sample | Encoding + Submit | 3.8720 | 3.6152 | -8.52% | -7.36% | -6.26% | -0.62% | -6.81% |
| discard then sample | Create bind groups | 0.4649 | 0.4622 | +1.07% | -2.31% | -0.72% | +0.70% | -0.01% |
| discard then sample | Create + Encoding + Submit | 4.3375 | 4.0773 | -7.52% | -6.81% | -5.66% | -0.48% | -6.24% |
| no sampled textures | Encoding | 1.3851 | 1.4024 | -1.50% | +1.52% | -2.41% | +8.37% | +0.01% |
| no sampled textures | Submit | 0.0580 | 0.0568 | +5.87% | +1.34% | -5.38% | -2.02% | -0.34% |
| no sampled textures | Encoding + Submit | 1.4417 | 1.4593 | -1.23% | +1.51% | -2.52% | +7.93% | +0.14% |
| no sampled textures | Create bind groups | 0.1793 | 0.1863 | -1.06% | +3.00% | +4.80% | +6.92% | +3.90% |
| no sampled textures | Create + Encoding + Submit | 1.6211 | 1.6456 | -1.21% | +1.68% | -1.75% | +7.82% | +0.23% |

完整的进程级阶段值、组内均值和配对结果见 [evidence.json](evidence.json)。未删除任何正式组；预检单独记录，不参与统计。
