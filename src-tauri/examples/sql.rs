use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{SqlitePool, SqliteRow},
    Connection, Executor, FromRow, Row, Sqlite, SqliteConnection,
};

const DB_URL: &str = ":memory:";

#[derive(Debug, FromRow)]
struct User {
    id: i64,
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // if !Sqlite::database_exists(DB_URL).await? {
    //     Sqlite::create_database(DB_URL).await?;
    //     println!("DB created");
    // }
    //
    let pool = SqlitePool::connect(DB_URL).await?;
    println!("DB connected");

    // sqlx::migrate::

    sqlx::raw_sql(include_str!("../migrations/test.sql"))
        .execute(&pool)
        .await?;

    let users = sqlx::query_as::<_, User>("select * from user")
        .fetch_all(&pool)
        .await?;

    let result = sqlx::query("select * from user")
        .map(|row: SqliteRow| User {
            id: row.get("id"),
            name: row.get("name"),
        })
        .fetch_all(&pool)
        .await?;

    dbg!(users);
    // let result = sqlx::query("select * from user").fetch_one(&pool).await?;

    // conn.execute("").await?;

    // sqlx::query("create table if not exists user (id integer primary key, name text not null)")
    //     .execute(&mut conn)
    //     .await?;

    Ok(())
}
