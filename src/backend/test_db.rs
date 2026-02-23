/// Shared in-memory database fixtures for backend integration tests.
///
/// Both `history_query` and `history_details_query` import from here so the
/// schema and dataset stay in one place.
///
/// Schema is applied from `migration.sql` via `include_str!` so the test DB
/// always matches the production schema automatically.
use chrono::{Duration, Utc};
use libsql::{Builder, Connection};

const MIGRATION_SQL: &str = include_str!("../../migration.sql");

/// Applies every statement from `migration.sql` to `conn`.
/// Comment-only lines and blank segments are skipped.
async fn apply_migration(conn: &Connection) {
    for raw in MIGRATION_SQL.split(';') {
        let stmt: String = raw
            .lines()
            .filter(|l| !l.trim().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");
        let stmt = stmt.trim();
        if !stmt.is_empty() {
            conn.execute(stmt, ())
                .await
                .unwrap_or_else(|e| panic!("Migration statement failed: {e}\nSQL: {stmt}"));
        }
    }
}

/// Full dataset: Alice (kid_id=1) with 3 monthly periods (relative to now),
/// Bob (kid_id=2) with notes spread across multiple months.
/// Use this fixture when you need realistic pagination data.
pub async fn setup_test_db() -> Connection {
    let db = Builder::new_local(":memory:")
        .build()
        .await
        .expect("Failed to create in-memory database");

    let conn = db
        .connect()
        .expect("Failed to connect to in-memory database");

    apply_migration(&conn).await;

    let now = Utc::now().naive_utc();

    // Kid 1: "Alice" – 3 positive notes in the most recent month,
    //                   2 positive + 1 negative 30-38 days ago,
    //                   4 positive ~60 days ago.
    conn.execute(
        "INSERT INTO kids (name, created_at) VALUES (?1, ?2)",
        libsql::params!["Alice", now.format("%Y-%m-%d %H:%M:%S").to_string()],
    )
    .await
    .expect("Failed to insert Alice");

    for i in 0..3i64 {
        let t = (now - Duration::days(10 - i))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        conn.execute(
            "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, 1, ?1)",
            libsql::params![t],
        )
        .await
        .expect("Failed to insert Alice note");
    }
    for i in 0..2i64 {
        let t = (now - Duration::days(30 + i * 3))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        conn.execute(
            "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, 1, ?1)",
            libsql::params![t],
        )
        .await
        .expect("Failed to insert Alice note");
    }
    let t = (now - Duration::days(38))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    conn.execute(
        "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, -1, ?1)",
        libsql::params![t],
    )
    .await
    .expect("Failed to insert Alice note");

    for i in 0..4i64 {
        let t = (now - Duration::days(60 + i * 2))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        conn.execute(
            "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, 1, ?1)",
            libsql::params![t],
        )
        .await
        .expect("Failed to insert Alice note");
    }

    // Kid 2: "Bob"
    conn.execute(
        "INSERT INTO kids (name, created_at) VALUES (?1, ?2)",
        libsql::params!["Bob", now.format("%Y-%m-%d %H:%M:%S").to_string()],
    )
    .await
    .expect("Failed to insert Bob");

    for i in 0..5i64 {
        let t = (now - Duration::days(20 - i))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        conn.execute(
            "INSERT INTO notes (kid_id, quantity, created_at) VALUES (2, 1, ?1)",
            libsql::params![t],
        )
        .await
        .expect("Failed to insert Bob note");
    }
    for i in 0..2i64 {
        let t = (now - Duration::days(50 + i * 3))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        conn.execute(
            "INSERT INTO notes (kid_id, quantity, created_at) VALUES (2, 1, ?1)",
            libsql::params![t],
        )
        .await
        .expect("Failed to insert Bob note");
    }
    for i in 0..2i64 {
        let t = (now - Duration::days(55 + i * 3))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        conn.execute(
            "INSERT INTO notes (kid_id, quantity, created_at) VALUES (2, -1, ?1)",
            libsql::params![t],
        )
        .await
        .expect("Failed to insert Bob note");
    }

    conn
}

/// Minimal dataset: Sam (kid_id=1) with notes in exactly two hard-coded
/// calendar months.  Use this fixture when you need deterministic dates.
///
/// 2026-02: Feb 05, Feb 10, Feb 15  — quantity=+1 each (3 notes)
/// 2026-01: Jan 10, Jan 20          — quantity=-1 each (2 notes)
pub async fn setup_two_period_db() -> Connection {
    let db = Builder::new_local(":memory:")
        .build()
        .await
        .expect("Failed to create in-memory database");
    let conn = db.connect().expect("Failed to connect");

    apply_migration(&conn).await;

    conn.execute(
        "INSERT INTO kids (name, created_at) VALUES ('Sam', '2026-01-01 00:00:00')",
        (),
    )
    .await
    .unwrap();

    // 2026-02
    for day in [5u8, 10, 15] {
        conn.execute(
            &format!(
                "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, 1, '2026-02-{:02} 00:00:00')",
                day
            ),
            (),
        )
        .await
        .unwrap();
    }

    // 2026-01
    for day in [10u8, 20] {
        conn.execute(
            &format!(
                "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, -1, '2026-01-{:02} 00:00:00')",
                day
            ),
            (),
        )
        .await
        .unwrap();
    }

    conn
}

/// Dataset with 55 notes in a single month for kid_id=1 ("Max"),
/// designed to trigger the MAX_HISTORY_ITEMS (50) cap.
///
/// 2026-03: 55 notes with quantity=+1, spread across the month.
pub async fn setup_many_notes_db() -> Connection {
    let db = Builder::new_local(":memory:")
        .build()
        .await
        .expect("Failed to create in-memory database");
    let conn = db.connect().expect("Failed to connect");

    apply_migration(&conn).await;

    conn.execute(
        "INSERT INTO kids (name, created_at) VALUES ('Max', '2026-01-01 00:00:00')",
        (),
    )
    .await
    .unwrap();

    // Insert 55 notes in 2026-03 (exceeds the 50-note cap)
    for i in 0..55u32 {
        let day = (i % 28) + 1; // days 1..=28
        let hour = i / 28; // overflow into different hours
        conn.execute(
            &format!(
                "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, 1, '2026-03-{:02} {:02}:00:00')",
                day, hour
            ),
            (),
        )
        .await
        .unwrap();
    }

    conn
}
