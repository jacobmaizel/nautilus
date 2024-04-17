pub mod models;
pub mod users;
use anyhow::{Context, Ok, Result};
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    Connection, PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::Mutex;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

lazy_static::lazy_static! {
    static ref MIGRATION_LOCK: Mutex<()> = Mutex::new(());
}

#[derive(Clone)]
pub struct Db {
    pub pool: DbPool,
}

impl Db {
    pub fn new(url: String) -> Self {
        let pool = init_db(url).unwrap();
        Db { pool }
    }

    pub fn get_conn(&self) -> DbConnection {
        self.pool.get().unwrap()
    }

    pub fn test_conn(&self) -> DbConnection {
        let mut conn = self.pool.get().unwrap();
        conn.begin_test_transaction().unwrap();
        conn
    }

    pub fn run_migrations(&self) -> Result<()> {
        let mut conn = self.get_conn();

        let _lock = MIGRATION_LOCK.lock();

        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        Ok(())
    }
}

fn init_db(url: String) -> Result<DbPool> {
    let pool = create_connection_pool(url)?;

    Ok(pool)
}

fn create_connection_pool(url: String) -> Result<DbPool> {
    let manager = ConnectionManager::<PgConnection>::new(url);

    let pool = diesel::r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .context("Failed to create Connection Pool")?;
    Ok(pool)
}
