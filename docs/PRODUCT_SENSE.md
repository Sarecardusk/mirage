# Mirage Product Sense (Implementation Boundary Focus)

> Last Updated: 2026-04-05

本文件只记录会影响底层实现边界的产品取舍，不记录视觉与文案层决策。
本文件不是边界约束单点定义；硬约束以 `DESIGN.md` / `RELIABILITY.md` / `SECURITY.md` 为准。

## 产品取舍（原因）

- 选择 `Theme Card` 作为核心实体，是为了将“共享设定”与“会话态”解耦。
- 不做 SillyTavern 角色卡兼容，是为了避免被兼容负担绑架演进速度。
- 采用本地优先与无云账号中心路线，是为了降低系统复杂度与安全暴露面。
- 当前仅保留 Sync-Ready 抽象，是为了预留扩展点但不提前引入分布式复杂性。
- 会话隔离以 `Session` 为单位，是为了避免把上下文隔离问题误建模为 Profile 管理。

## 对实现的影响（引用权威文档）

- 架构职责与契约：`DESIGN.md`
- 数据一致性与迁移：`RELIABILITY.md`
- 安全拒绝策略：`SECURITY.md`

## Profile 定位

- `Profile` 在仅作为可选工作区容器保留，不作为对外主模型。
- 默认单一 Profile 且 UI 不暴露，避免把会话隔离问题错误建模为 Profile 隔离。

## 本期不讨论

- UI 样式与布局。
- 可替换库的偏好选择。
- 运营策略与商业化策略。
