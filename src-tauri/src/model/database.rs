use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Row, Sqlite,
};
use tauri::Manager;
use tracing::{error, info};

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// 初始化 SQLite 数据库连接池
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        // 获取应用数据目录
        let app_data_dir = app_handle.path().app_data_dir()?;

        // 确保目录存在
        std::fs::create_dir_all(&app_data_dir)?;

        // 构建数据库文件路径
        let db_path = app_data_dir.join("svroll.db");
        info!("数据库路径: {}", db_path.display());

        // 配置 SQLite 连接
        let connect_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

        // 创建连接池
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connect_options)
            .await?;

        // 初始化数据库表
        Self::init_database(&pool).await?;

        Ok(Self { pool })
    }

    async fn init_database(pool: &Pool<Sqlite>) -> Result<()> {
        // 创建记录表，使用 TEXT 类型存储 JSON 数据
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS json_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                record_type TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 创建索引加速查询
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_json_records_type 
            ON json_records(record_type)
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 保存 HistoryConfig 到数据库
    pub async fn save_config(&self, config: &HistoryConfig) -> Result<i64> {
        let json_data = serde_json::to_string(&config.data)?;

        let result = sqlx::query(
            r#"
            INSERT INTO json_records (record_type, data, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
        )
        .bind(&config.record_type)
        .bind(&json_data)
        .fetch_one(&self.pool)
        .await?;

        let id = result.get::<i64, _>("id");
        Ok(id)
    }

    /// 按类型获取所有记录为 HistoryConfig
    pub async fn get_configs(&self) -> Result<Vec<HistoryConfig>> {
        let rows = sqlx::query(
            r#"
            SELECT id, record_type, data, created_at as create_at, updated_at as update_at 
            FROM json_records
            ORDER BY create_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let id = row.get::<i64, _>("id");
            let record_type = row.get::<String, _>("record_type");
            let json_data: String = row.get("data");
            let created_at: DateTime<Local> = row.get("create_at");
            let updated_at: DateTime<Local> = row.get("update_at");

            match serde_json::from_str::<Value>(&json_data) {
                Ok(data) => {
                    let config = HistoryConfig {
                        id,
                        record_type,
                        data,
                        created_at,
                        updated_at,
                    };
                    results.push(config);
                }
                Err(e) => {
                    error!("解析 JSON 失败, ID: {}, 错误: {}", id, e);
                    continue;
                }
            }
        }

        Ok(results)
    }
    
    /// 获取单个配置记录
    pub async fn get_config(&self, id: i64) -> Result<Option<HistoryConfig>> {
        let row = sqlx::query(
            r#"
            SELECT id, record_type, data, 
                   created_at as create_at, updated_at as update_at 
            FROM json_records
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let id = row.get::<i64, _>("id");
            let record_type = row.get::<String, _>("record_type");
            let json_data: String = row.get("data");
            let created_at: DateTime<Local> = row.get("create_at");
            let updated_at: DateTime<Local> = row.get("update_at");
            
            let data = serde_json::from_str::<Value>(&json_data)
                .map_err(|e| anyhow::anyhow!("解析 JSON 失败: {}", e))?;
                
            Ok(Some(HistoryConfig {
                id,
                record_type,
                data,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn update_config(&self, config: &HistoryConfig) -> Result<bool> {
        let json_data = serde_json::to_string(&config.data)?;
        
        let result = sqlx::query(
            r#"
            UPDATE json_records 
            SET data = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(json_data)
        .bind(config.id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_all_configs(&self) -> Result<bool> {
        let result = sqlx::query("DELETE FROM json_records")
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub id: i64,
    #[serde(rename = "recordType")]
    pub record_type: String,
    pub data: Value,
    #[serde(with = "datetime_format", rename = "created_at")]
    pub created_at: DateTime<Local>,
    #[serde(with = "datetime_format", rename = "updated_at")]
    pub updated_at: DateTime<Local>,
}

impl HistoryConfig {
    // 创建新的配置记录，无需ID（数据库会自动分配）
    pub fn new<T: Serialize>(record_type: &str, data: &T) -> Result<Self> {
        let data_value = serde_json::to_value(data)?;
        let now = Local::now();
        
        Ok(Self {
            id: 0, // 将由数据库分配
            record_type: record_type.to_string(),
            data: data_value,
            created_at: now,
            updated_at: now,
        })
    }
    
    // 从特定类型数据获取内部数据
    pub fn get_data<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_value(self.data.clone())?)
    }
    
    // 更新数据
    pub fn set_data<T: Serialize>(&mut self, data: &T) -> Result<()> {
        self.data = serde_json::to_value(data)?;
        self.updated_at = Local::now();
        Ok(())
    }
}

mod datetime_format {
    use chrono::{DateTime, Local};
    use serde::{Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        chrono::NaiveDateTime::parse_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
            .and_then(|naive| {
                naive.and_local_timezone(Local)
                    .earliest()
                    .ok_or_else(|| serde::de::Error::custom("Failed to convert to local timezone"))
            })
    }
}
