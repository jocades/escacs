use anyhow::Context;
use sqlx::{
    migrate::MigrateDatabase, sqlite::SqliteQueryResult, Executor, Pool, Sqlite, SqlitePool,
};
use std::path::Path;
use tauri::State;
use tracing::{debug, trace};

use crate::AppState;

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn connect(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let url = path
            .as_ref()
            .to_str()
            .context("invalid path to db url conversion")?;

        if !Sqlite::database_exists(url).await? {
            Sqlite::create_database(url).await?;
            trace!(url, "new database created");
        }

        let pool = SqlitePool::connect(url).await?;
        Ok(Self { pool })
    }

    pub async fn connect_and_migrate(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = Self::connect(path).await?;
        for file in std::fs::read_dir("migrations")?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_digit()
            })
        {
            sqlx::raw_sql(std::fs::read_to_string(&file)?.as_str())
                .execute(&db.pool)
                .await?;
            trace!(?file, "migrated");
        }
        Ok(db)
    }

    pub async fn execute_raw(&self, raw_sql: &str) -> anyhow::Result<SqliteQueryResult> {
        Ok(sqlx::raw_sql(raw_sql).execute(&self.pool).await?)
    }
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Study {
    id: i32,
    name: String,
    tree_json: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewStudy {
    name: String,
    tree_json: String,
}

pub type Json = serde_json::Map<String, serde_json::Value>;

#[tauri::command]
pub async fn insert_study(study: NewStudy, state: State<'_, AppState>) -> Result<i64, String> {
    let id = sqlx::query(
        r#"
        insert into study (name, tree_json)
        values ($1, $2)
        "#,
    )
    .bind(study.name)
    .bind(study.tree_json)
    .execute(&state.db.pool)
    .await
    .unwrap()
    .last_insert_rowid();

    Ok(id)
}

#[tauri::command]
pub async fn get_studies(state: State<'_, AppState>) -> Result<Vec<Study>, String> {
    let studies = sqlx::query_as::<_, Study>("select * from study")
        .fetch_all(&state.db.pool)
        .await
        .unwrap();
    Ok(studies)
}

#[tauri::command]
pub async fn update_study(state: State<'_, AppState>) -> Result<(), String> {
    state.db.execute_raw("insert").await.unwrap();
    Ok(())
}
