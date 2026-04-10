# Tech Debt Tracker

本表仅记录与“高重构成本边界”相关的技术债。

| ID       | Debt                        | Impacted Boundary     | Current Risk | Suggested Direction                               | Status |
| -------- | --------------------------- | --------------------- | ------------ | ------------------------------------------------- | ------ |
| DEBT-001 | `specta-zod` 自动链路不可用 | IPC 类型一致性        | 中           | 继续维持 Rust + 手写 Zod 的双轨校验，等待生态稳定 | Open   |
| DEBT-002 | Sync 仅有保留接口无执行路径 | Sync-Ready 边界清晰度 | 低           | 在未来需求确认后补独立设计文档，不提前实现        | Open   |
