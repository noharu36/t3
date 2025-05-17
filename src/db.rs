use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Error;

#[derive(Deserialize, Serialize, Debug, sqlx::FromRow)]
pub struct ObjectMetadata {
    pub id: i64,
    pub bucket_name: String,
    pub object_id: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<i64>,
    pub created_at: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Bucket {
    pub id: String,
    pub bucket_name: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct MetadataStore {
    pool: sqlx::SqlitePool,
}

impl MetadataStore {
    pub async fn new(db_path: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(&db_path).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn insert_metadata(
        &self,
        bucket_name: &str,
        object_id: &str,
        file_name: Option<&str>,
        content_type: Option<&str>,
        content_length: i32,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query!(
            "
            INSERT INTO object_metadata (bucket_name, object_id, file_name, content_type, content_length, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ",
            bucket_name,
            object_id,
            file_name,
            content_type,
            content_length,
            now
        )
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_metadata(
        &self,
bucket_name: &str,
        object_id: &str,
    ) -> Result<Option<ObjectMetadata>> {
        let row = sqlx::query_as!(
            ObjectMetadata,
            "
            SELECT * FROM object_metadata
            WHERE bucket_name = ? AND object_id = ?
            ",
            bucket_name,
            object_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn delete_metadata(
        &self,
        bucket_name: &str,
        object_id: &str,
    ) -> Result<SqliteQueryResult> {
        let result = sqlx::query!(
            "
            DELETE FROM object_metadata WHERE bucket_name = ? AND object_id = ?
            ",
            bucket_name,
            object_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn create_bucket(
        &self,
        bucket_id: &str,
        bucket_name: &str,
        created_at: &str,
    ) -> Result<SqliteQueryResult, Error> {
        let result = sqlx::query!(
            "
            INSERT OR IGNORE INTO bucket_metadata (id, bucket_name, created_at)
            VALUES (?, ?, ?)
            ",
            bucket_id,
            bucket_name,
            created_at
        )
        .execute(&self.pool)
        .await;

        result
    }

    pub async fn get_buckets(&self) -> Result<Vec<Bucket>, Error> {
        let result = sqlx::query_as!(Bucket, "SELECT * FROM bucket_metadata")
            .fetch_all(&self.pool)
            .await;

        result
    }

    pub async fn delete_bucket(&self, bucket_name: &str) -> Result<SqliteQueryResult, Error> {
        let result = sqlx::query!(
            "
            DELETE FROM bucket_metadata WHERE bucket_name = ?
            ",
            bucket_name,
        )
        .execute(&self.pool)
        .await;

        result
    }

    pub async fn exist_buckets(&self, bucket_name: &str) -> Result<i64, Error> {
        let result = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM bucket_metadata WHERE bucket_name = ?)",
            bucket_name
        )
        .fetch_one(&self.pool)
        .await;

        result
    }
}
