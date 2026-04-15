use std::path::Path;

#[cfg(test)]
use surrealdb::engine::local::Mem;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::Surreal;

/// 对 `Surreal<Db>` 连接做的一层轻量封装。
///
/// `connect_file`（生产）和 `connect_memory`（测试）返回的底层类型一致，
/// 因此 infra 层其余代码可以只面对这一种具体句柄。
pub struct Database {
    inner: Surreal<Db>,
}

impl Database {
    /// 在 `path` 打开一个基于 SurrealKV 文件的持久化数据库。
    pub async fn connect_file(path: &Path) -> anyhow::Result<Self> {
        let db = Surreal::new::<SurrealKv>(path).await?;
        db.use_ns("mirage").use_db("main").await?;
        Ok(Self { inner: db })
    }

    /// 打开内存数据库，只用于单元测试。
    #[cfg(test)]
    pub async fn connect_memory() -> anyhow::Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("mirage").use_db("main").await?;
        Ok(Self { inner: db })
    }

    /// 暴露内部的 `Surreal<Db>` 句柄，方便仓储实现直接调用完整的 SurrealDB 接口。
    pub fn inner(&self) -> &Surreal<Db> {
        &self.inner
    }
}
