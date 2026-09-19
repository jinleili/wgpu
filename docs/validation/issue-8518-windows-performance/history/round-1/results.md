# wgpu #8518 Windows 硬件性能 A/B：全部结果

日期：2026-09-19。单位：每迭代 CPU API 墙钟毫秒。负百分比表示耗时下降。A/B 绝对值为四个组内版本均值的中位数；百分比为四个配对百分比的中位数，因此不能由表中两列中位数重新计算百分比。

每进程先将阶段平均耗时相加，再在组内分别平均两个 A 与两个 B，计算 `(B/A - 1) × 100%`。组合阶段来自相同进程；创建合计是假设每轮创建 1000 个 bind group 的使用频率估算，不是创建后立即使用的实测流程。128 次迭代不是独立样本，独立配对单位为四个进程组成的一组。

## VULKAN

### A/A：同一 A 二进制，中间两次 / 首尾两次 − 1

| 场景 | 阶段 | 第 1 组 | 第 2 组 |
|---|---|---:|---:|
| shared attachments | Encoding | -0.46% | +1.78% |
| shared attachments | Submit | +0.89% | +6.57% |
| shared attachments | Encoding + Submit | -0.40% | +2.00% |
| shared attachments | Create bind groups | +1.30% | +4.70% |
| shared attachments | Create + Encoding + Submit | +0.03% | +2.67% |
| overlapping mixed views | Encoding | -0.94% | -0.23% |
| overlapping mixed views | Submit | -10.61% | +0.21% |
| overlapping mixed views | Encoding + Submit | -1.29% | -0.21% |
| overlapping mixed views | Create bind groups | -0.11% | -0.87% |
| overlapping mixed views | Create + Encoding + Submit | -1.03% | -0.36% |
| discard then sample | Encoding | -2.09% | +0.55% |
| discard then sample | Submit | +0.13% | +1.07% |
| discard then sample | Encoding + Submit | -2.00% | +0.57% |
| discard then sample | Create bind groups | +1.50% | +8.36% |
| discard then sample | Create + Encoding + Submit | -1.25% | +2.29% |
| no sampled textures | Encoding | +2.97% | +2.10% |
| no sampled textures | Submit | +3.16% | +4.23% |
| no sampled textures | Encoding + Submit | +2.97% | +2.19% |
| no sampled textures | Create bind groups | -3.46% | -0.41% |
| no sampled textures | Create + Encoding + Submit | +1.92% | +1.77% |

### A/B：四组全部保留

G1/G3 为 ABBA，G2/G4 为 BAAB。

| 场景 | 阶段 | A ms | B ms | G1 | G2 | G3 | G4 | 配对中位数 |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| shared attachments | Encoding | 2.1016 | 1.7841 | -17.20% | -17.00% | -13.41% | -13.24% | -15.21% |
| shared attachments | Submit | 0.1042 | 0.1056 | -10.52% | +2.86% | +2.71% | +2.10% | +2.40% |
| shared attachments | Encoding + Submit | 2.2104 | 1.8897 | -16.87% | -16.08% | -12.64% | -12.51% | -14.36% |
| shared attachments | Create bind groups | 0.7504 | 0.7395 | -2.31% | -0.79% | -1.09% | +2.43% | -0.94% |
| shared attachments | Create + Encoding + Submit | 2.9608 | 2.6343 | -13.21% | -12.24% | -9.69% | -8.78% | -10.96% |
| overlapping mixed views | Encoding | 2.5608 | 2.2282 | -10.90% | -12.49% | -14.76% | -11.81% | -12.15% |
| overlapping mixed views | Submit | 0.0860 | 0.0890 | +4.10% | +5.43% | +3.77% | +1.42% | +3.94% |
| overlapping mixed views | Encoding + Submit | 2.6471 | 2.3166 | -10.42% | -11.91% | -14.15% | -11.38% | -11.64% |
| overlapping mixed views | Create bind groups | 0.7546 | 0.7507 | +0.70% | -0.01% | -2.99% | -1.24% | -0.63% |
| overlapping mixed views | Create + Encoding + Submit | 3.4044 | 3.0713 | -7.96% | -9.26% | -11.67% | -9.08% | -9.17% |
| discard then sample | Encoding | 2.4611 | 2.1341 | -13.21% | -13.42% | -14.08% | -10.74% | -13.32% |
| discard then sample | Submit | 0.1014 | 0.1044 | +1.89% | +1.21% | +3.39% | +7.06% | +2.64% |
| discard then sample | Encoding + Submit | 2.5632 | 2.2385 | -12.61% | -12.84% | -13.40% | -10.05% | -12.72% |
| discard then sample | Create bind groups | 0.7284 | 0.7494 | +3.22% | +2.53% | +3.48% | +2.69% | +2.95% |
| discard then sample | Create + Encoding + Submit | 3.2916 | 2.9864 | -9.11% | -9.43% | -9.72% | -7.18% | -9.27% |
| no sampled textures | Encoding | 1.2727 | 1.2959 | +0.91% | -1.03% | +3.38% | +11.06% | +2.15% |
| no sampled textures | Submit | 0.0564 | 0.0568 | -3.17% | -1.90% | +4.07% | +5.04% | +1.09% |
| no sampled textures | Encoding + Submit | 1.3287 | 1.3528 | +0.74% | -1.06% | +3.41% | +10.81% | +2.08% |
| no sampled textures | Create bind groups | 0.2546 | 0.2658 | +4.73% | -2.84% | +4.78% | +6.44% | +4.75% |
| no sampled textures | Create + Encoding + Submit | 1.5882 | 1.6186 | +1.38% | -1.36% | +3.63% | +10.11% | +2.51% |

## DX12

### A/A：同一 A 二进制，中间两次 / 首尾两次 − 1

| 场景 | 阶段 | 第 1 组 | 第 2 组 |
|---|---|---:|---:|
| shared attachments | Encoding | +3.09% | +20.10% |
| shared attachments | Submit | +2.13% | +307.44% |
| shared attachments | Encoding + Submit | +3.06% | +30.19% |
| shared attachments | Create bind groups | +6.64% | +6.69% |
| shared attachments | Create + Encoding + Submit | +3.49% | +27.38% |
| overlapping mixed views | Encoding | -0.15% | -1.39% |
| overlapping mixed views | Submit | -3.96% | +1.78% |
| overlapping mixed views | Encoding + Submit | -0.26% | -1.30% |
| overlapping mixed views | Create bind groups | -0.58% | -1.11% |
| overlapping mixed views | Create + Encoding + Submit | -0.30% | -1.28% |
| discard then sample | Encoding | -2.04% | -0.81% |
| discard then sample | Submit | -0.59% | -2.38% |
| discard then sample | Encoding + Submit | -1.99% | -0.86% |
| discard then sample | Create bind groups | -3.93% | -3.52% |
| discard then sample | Create + Encoding + Submit | -2.20% | -1.16% |
| no sampled textures | Encoding | -1.98% | -4.30% |
| no sampled textures | Submit | +21.20% | +2.58% |
| no sampled textures | Encoding + Submit | -1.15% | -4.07% |
| no sampled textures | Create bind groups | +0.39% | +1.08% |
| no sampled textures | Create + Encoding + Submit | -0.98% | -3.51% |

### A/B：四组全部保留

G1/G3 为 ABBA，G2/G4 为 BAAB。

| 场景 | 阶段 | A ms | B ms | G1 | G2 | G3 | G4 | 配对中位数 |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| shared attachments | Encoding | 3.3481 | 3.1982 | -6.09% | -6.54% | -2.80% | -2.94% | -4.51% |
| shared attachments | Submit | 0.1251 | 0.1299 | +3.29% | +1.66% | +6.01% | +8.54% | +4.65% |
| shared attachments | Encoding + Submit | 3.4764 | 3.3282 | -5.75% | -6.25% | -2.48% | -2.51% | -4.13% |
| shared attachments | Create bind groups | 0.4623 | 0.4680 | +1.97% | +1.71% | -15.84% | +0.72% | +1.21% |
| shared attachments | Create + Encoding + Submit | 3.9853 | 3.8000 | -4.83% | -5.33% | -4.36% | -2.14% | -4.60% |
| overlapping mixed views | Encoding | 3.3084 | 2.9679 | -10.11% | -10.69% | -9.85% | -9.13% | -9.98% |
| overlapping mixed views | Submit | 0.1043 | 0.1069 | +2.31% | +5.26% | +0.65% | +2.63% | +2.47% |
| overlapping mixed views | Encoding + Submit | 3.4129 | 3.0758 | -9.73% | -10.20% | -9.52% | -8.77% | -9.63% |
| overlapping mixed views | Create bind groups | 0.4745 | 0.4785 | +1.90% | -1.56% | -0.05% | -2.52% | -0.81% |
| overlapping mixed views | Create + Encoding + Submit | 3.8893 | 3.5617 | -8.31% | -9.11% | -8.36% | -8.01% | -8.34% |
| discard then sample | Encoding | 3.9122 | 3.5076 | -11.25% | -12.60% | -11.44% | -7.97% | -11.34% |
| discard then sample | Submit | 0.1346 | 0.1340 | +2.23% | -2.95% | -6.81% | +4.36% | -0.36% |
| discard then sample | Encoding + Submit | 4.0464 | 3.6431 | -10.81% | -12.28% | -11.28% | -7.55% | -11.04% |
| discard then sample | Create bind groups | 0.4696 | 0.4653 | +2.02% | +0.04% | -3.32% | +3.95% | +1.03% |
| discard then sample | Create + Encoding + Submit | 4.5160 | 4.1176 | -9.49% | -11.03% | -10.45% | -6.33% | -9.97% |
| no sampled textures | Encoding | 1.3898 | 1.4293 | +3.58% | +2.28% | -1.20% | +6.73% | +2.93% |
| no sampled textures | Submit | 0.0573 | 0.0593 | +2.92% | +1.56% | +4.05% | +4.30% | +3.48% |
| no sampled textures | Encoding + Submit | 1.4473 | 1.4886 | +3.56% | +2.25% | -0.99% | +6.63% | +2.91% |
| no sampled textures | Create bind groups | 0.1808 | 0.1832 | -2.06% | +1.77% | +2.75% | +3.32% | +2.26% |
| no sampled textures | Create + Encoding + Submit | 1.6282 | 1.6709 | +2.93% | +2.20% | -0.59% | +6.27% | +2.56% |

完整的进程级阶段值、组内均值和配对结果见 [evidence.json](evidence.json)。未删除任何正式组；预检单独记录，不参与统计。
