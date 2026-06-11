# LiteBench · 轻量电脑评测

一个仿 Novabench 的轻量化电脑评测工具。**最终形态是一个单文件 `.exe`**：用户下载 → 双击 →
自动跑分、读取精准硬件信息、并自动打开浏览器展示综合评分与硬件详情。无需安装、无需上传文件。

## 用户体验（零繁琐）

1. 下载 `bench-agent.exe`（约 2 MB，绿色免安装）。
2. 双击运行。它会：
   - 用 CIM/WMI 读取**精准**硬件（整机型号、CPU、内存容量与批次、磁盘、GPU、推断生产年份）；
   - 跑分测试 CPU / 内存 / 存储；
   - 在本机 `127.0.0.1:38291` 提供**内嵌网页界面**并**自动打开浏览器**显示结果。
3. 浏览器里实时看到进度，跑完自动展示评分、雷达图、网络对比和硬件详情。

## 架构

```
bench-agent.exe  =  [CIM/WMI 硬件检测 + 跑分]  +  [内嵌 Web UI + 本地 HTTP 服务]
```

- 网页直接由 exe 同源提供（`127.0.0.1:38291`），因此 `fetch('/status')`、`fetch('/result')`
  无需 CORS、无混合内容问题，也无需用户上传 `result.json`。
- 精准硬件检测必须由原生程序完成（浏览器沙箱读不到型号/批次/序列号），这正是 exe 存在的原因。

## 目录结构

```
bench/
├─ agent/                 # Rust：硬件检测 + 跑分 + 内嵌 UI + 本地服务
│  └─ src/
│     ├─ main.rs          # 后台跑分 + 启动服务 + 自动开浏览器
│     ├─ dmi.rs           # CIM/WMI 精准检测（型号/内存批次/磁盘/GPU/年份推断）
│     ├─ bench_cpu/mem/disk.rs
│     ├─ score.rs         # 归一化加权综合评分
│     ├─ server.rs        # tiny_http：/status /result + 内嵌静态界面(include_dir)
│     ├─ state.rs / types.rs
└─ web/                   # React + Vite + ECharts 前端（构建产物内嵌进 exe）
   └─ src/...
```

## 准确性说明（已对齐 Windows“系统信息”页）

数据源与“此电脑→属性”一致（CIM/WMI）：

- 整机型号：`Win32_ComputerSystem`（如 `Dell Inc. G3 3590`）
- CPU：`Win32_Processor`（名称 / 主频 / 核心 / 线程）
- 内存：装机容量取各内存条容量之和；并展示每根的厂商/料号/序列号（真实批次）
- 磁盘 / 显卡：`Win32_DiskDrive` / `Win32_VideoController`（显卡自动跳过虚拟显示器，优先独显）
- 生产年份：依据 BIOS 发布日期**估算**（精确 SPD/SMART 周年用户态读不到，标注为估算）

## 从源码构建单文件 exe

> 最终用户**不需要** Rust。只有构建这一步需要。

环境：Node.js（构建前端）+ Rust（编译 exe）。

```bash
# 1) 构建前端（产物在 web/dist，会被 exe 内嵌）
cd web && npm install && npm run build

# 2) 编译单文件 exe
cd ../agent && cargo build --release
# 产物：agent/target/release/bench-agent.exe（自包含，无外部 DLL 依赖）
```

Windows 上若用 GNU 工具链，需要 `rust-mingw` 组件提供链接器（`rustup component add rust-mingw`）。
本项目刻意只依赖纯 Rust + 链接 Windows 系统库的 crate，无需安装 MSVC Build Tools 或 mingw `as`。

## 前端开发模式

```bash
cd web && npm run dev    # http://localhost:5173
```
开发模式下前端会连接 `http://127.0.0.1:38291` 的本地助手（需另行运行 exe）。
也可点“加载示例数据”用 `web/src/data/baseline.ts` 的示例预览界面。

## 评分方法

各项原始指标除以基准常数再加权，归一到“主流参考机 ≈ 1000 分”：

- `cpu = (单核/BASE_SINGLE*0.4 + 多核/BASE_MULTI*0.6) * 1000`
- `memory = 带宽/BASE_MEM * 1000`
- `disk = (顺序/BASE_SEQ*0.5 + 随机IOPS/BASE_RAND*0.5) * 1000`
- `total = cpu*0.5 + memory*0.2 + disk*0.3`

基准常数见 `agent/src/score.rs`，后续接入云端后用真实分布重新校准。

## 后续可选增强

- 真实网络排行榜（Node/Go + PostgreSQL，按型号聚合百分位）替换内置参考机型样本。
- 代码签名，避免 Windows SmartScreen 提示。
- GitHub Actions 自动产出三平台 exe。
```
