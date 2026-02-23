pub mod history_details_query;
pub mod history_query;
pub mod kids;
pub mod turso;

#[cfg(all(test, feature = "server"))]
pub mod test_db;

#[cfg(all(feature = "server"))]
pub mod constants {
    use std::{env, sync::LazyLock};

    pub const MAX_HISTORY_PAGE_SIZE: LazyLock<u8> = LazyLock::new(|| {
        env::var("HD_MAX_HISTORY_PAGE_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15)
    });

    pub const MAX_HISTORY_DETAIL_PAGE_SIZE: LazyLock<usize> = LazyLock::new(|| {
        env::var("HD_MAX_HISTORY_DETAIL_PAGE_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50)
    });
}
