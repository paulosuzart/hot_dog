#[cfg(feature = "server")]
use libsql::{Builder, Connection, Database};
#[cfg(feature = "server")]
use std::ops::{Deref, DerefMut};
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use tokio::sync::{Mutex, OnceCell};

#[cfg(feature = "server")]
#[derive(Debug)]
struct SimplePool {
    db: Arc<Database>,
    available: Mutex<Vec<Connection>>,
}

#[cfg(feature = "server")]
impl SimplePool {
    fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            available: Mutex::new(Vec::new()),
        }
    }
}

#[cfg(feature = "server")]
async fn acquire_from_pool(pool: &Arc<SimplePool>) -> PooledConn {
    let mut available = pool.available.lock().await;
    let conn = available.pop().unwrap_or_else(|| {
        pool.db.connect().expect("Failed to connect to database")
    });

    PooledConn {
        conn: Some(conn),
        pool: Arc::clone(pool),
    }
}

#[cfg(feature = "server")]
async fn release_to_pool(pool: &Arc<SimplePool>, conn: Connection) {
    let mut available = pool.available.lock().await;
    if available.len() < 10 {
        available.push(conn);
    }
}

#[cfg(feature = "server")]
#[derive(Debug)]
pub struct PooledConn {
    conn: Option<Connection>,
    pool: Arc<SimplePool>,
}

#[cfg(feature = "server")]
impl Deref for PooledConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().unwrap()
    }
}

#[cfg(feature = "server")]
impl DerefMut for PooledConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().unwrap()
    }
}

#[cfg(feature = "server")]
impl Drop for PooledConn {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let pool = Arc::clone(&self.pool);
            tokio::spawn(async move {
                release_to_pool(&pool, conn).await;
            });
        }
    }
}

#[cfg(feature = "server")]
static POOL: OnceCell<Arc<SimplePool>> = OnceCell::const_new();

#[cfg(feature = "server")]
async fn init_db_pool() -> Arc<SimplePool> {
    let url = std::env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL must be set");
    let token = std::env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");

    let db = Builder::new_remote(url, token)
        .build()
        .await
        .expect("Failed to build database");

    Arc::new(SimplePool::new(Arc::new(db)))
}

#[cfg(feature = "server")]
pub async fn get_db() -> PooledConn {
    let pool = POOL.get_or_init(|| init_db_pool()).await;
    acquire_from_pool(pool).await
}
