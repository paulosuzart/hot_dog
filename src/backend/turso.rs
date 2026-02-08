#[cfg(feature = "server")]
use libsql::{Builder, Connection};
#[cfg(feature = "server")]
use tokio::sync::OnceCell;

#[cfg(feature = "server")]
static CONN: OnceCell<Connection> = OnceCell::const_new();

#[cfg(feature = "server")]
async fn init_db() -> Connection {
    let url = std::env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL must be set");
    let token = std::env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");

    let db = Builder::new_remote(url, token)
        .build()
        .await
        .expect("Failed to build database");
    db.connect().expect("Failed to connect to database")
}

#[cfg(feature = "server")]
pub async fn get_db() -> &'static Connection {
    CONN.get_or_init(|| init_db()).await
}
