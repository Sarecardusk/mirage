# Mirage Docs

`docs/` 是 Mirage 的实现边界文档入口。
本目录只记录本期需要锁定的“高重构成本约束”以及这些约束如何被执行、验证、演进。

## 核心规范（权威）

- `DESIGN.md`：系统边界、模块职责、IPC 分层与稳定接口约束。
- `RELIABILITY.md`：数据一致性、迁移、失败路径与降级策略。
- `SECURITY.md`：安全模型、密钥策略、硬拒绝规则。
- `HARNESS.md`：Harness 工程执行语义、命令/事件/流式/诊断约束。
- `COMPATIBILITY.md`：公开契约兼容性策略与 breaking change 定义。
- `TEST_STRATEGY.md`：约束到测试证据的映射原则与最小回归矩阵。
- `OPERATIONS.md`：可观测性、关联 ID、脱敏与诊断要求。

## 主题约束（索引）

- `CONSTRAINT_INDEX.md`：全约束导航索引（按执行级别分组，链回权威来源）。
- `FRONTEND.md`：UI Host 可做/不可做边界与当前期 UI 技术基线约定。
- `GARDENING_POLICY.md`：文档养护规则。
- `PRODUCT_SENSE.md`：影响底层实现边界的产品取舍。
- `QUALITY_SCORE.md`：边界向验收标准与评审清单。

## 子目录

- `design-docs/`：设计文档索引与模板要求。
- `exec-plans/`：执行计划模板、进行中计划与已归档计划。
- `product-specs/`：产品规格索引与版本规格草案。
- `references/`：外部参考资料索引。

## 范围声明

- 本目录覆盖：实现边界、稳定契约、不可漂移职责、失败与安全下界。
- 本目录覆盖：约束执行方式、兼容性策略、测试证据与诊断要求。
- 本目录不覆盖：页面样式、视觉细节、可替换库技巧、低成本重构点。
- 例外说明：`FRONTEND.md` 可定义“当前期 UI 技术基线”，该约束用于统一实现栈，不等同视觉规范。

## 状态说明

当前仓库仍处于模板期，文档中的约束分两类：

- 当前已生效：协作流程、文档门禁、基础工程链、capability/CSP 开发态约束。
- 落地即生效：`Theme Card`、`Session`、迁移、检索、网关、流式生成等业务能力一旦开始实现，就必须同时满足的约束。

## 本期作用域模型

- 默认采用 `单用户单工作区 + Theme Card 下多 Session`。
- `Theme Card` 承载共享设定，`Session` 承载独立上下文与记忆。
- `Profile` 仅保留为可选扩展容器，默认不作为用户可见主模型。
