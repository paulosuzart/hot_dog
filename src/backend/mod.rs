pub mod history_details_query;
pub mod history_query;
pub mod kids;
pub mod turso;

#[cfg(feature = "server")]
pub mod db {

    #[cfg(feature = "server")]
    pub trait QueryExecutor {
        type Output;
        type Error;
        async fn execute_query(
            &self,
            conn: &libsql::Connection,
        ) -> Result<Self::Output, Self::Error>;
    }
}
