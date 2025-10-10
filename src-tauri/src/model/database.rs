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

use crate::param::Protocol;

/// 数据库管理器
///
/// 管理SQLite数据库连接和提供CRUD操作
#[derive(Debug)]
pub struct Database {
    /// SQLite连接池
    pool: Pool<Sqlite>,
}

impl Database {
    /// 初始化SQLite数据库
    ///
    /// 创建连接池并初始化数据库表
    ///
    /// # 参数
    /// * `app_handle` - Tauri应用句柄，用于获取应用数据目录
    ///
    /// # 返回
    /// 成功返回数据库实例，失败返回错误
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
            .max_connections(1)
            .connect_with(connect_options)
            .await?;

        // 初始化数据库表
        Self::init_database(&pool).await?;

        Ok(Self { pool })
    }

    /// 初始化数据库表结构
    ///
    /// 创建必要的表和索引
    ///
    /// # 参数
    /// * `pool` - 数据库连接池
    ///
    /// # 返回
    /// 成功返回Ok(()), 失败返回错误
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

    /// 保存配置到数据库
    ///
    /// # 参数
    /// * `config` - 要保存的历史配置
    ///
    /// # 返回
    /// 成功返回插入记录的ID，失败返回错误
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

    /// 获取所有配置记录
    ///
    /// 按创建时间降序返回所有历史配置
    ///
    /// # 返回
    /// 成功返回历史配置列表，失败返回错误
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
    ///
    /// 根据ID获取特定的历史配置
    ///
    /// # 参数
    /// * `id` - 配置记录ID
    ///
    /// # 返回
    /// 成功返回配置记录，不存在返回None，失败返回错误
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

    /// 更新配置记录
    ///
    /// # 参数
    /// * `config` - 要更新的配置记录
    ///
    /// # 返回
    /// 成功返回是否更新了记录，失败返回错误
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

    /// 删除所有配置记录
    ///
    /// # 返回
    /// 成功返回是否删除了记录，失败返回错误
    pub async fn delete_all_configs(&self) -> Result<bool> {
        let result = sqlx::query("DELETE FROM json_records")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// 历史配置记录
///
/// 存储在数据库中的配置历史记录
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// 记录ID
    pub id: i64,
    /// 记录类型
    #[serde(rename = "recordType")]
    pub record_type: String,
    /// 配置数据，以JSON格式存储
    pub data: Value,
    /// 创建时间
    #[serde(with = "datetime_format", rename = "created_at")]
    pub created_at: DateTime<Local>,
    /// 更新时间
    #[serde(with = "datetime_format", rename = "updated_at")]
    pub updated_at: DateTime<Local>,
}

impl HistoryConfig {
    /// 创建新的配置记录
    ///
    /// # 参数
    /// * `record_type` - 记录类型
    /// * `data` - 要存储的数据对象
    ///
    /// # 返回
    /// 成功返回历史配置实例，失败返回错误
    pub fn new<T: Serialize>(record_type: Protocol, data: &T) -> Result<Self> {
        let data_value = serde_json::to_value(data)?;
        let now = Local::now();
        let protocol = match record_type {
            Protocol::Mqtt => "mqtt",
            Protocol::Tcp => "tcp",
        };

        Ok(Self {
            id: 0,
            record_type: protocol.to_string(),
            data: data_value,
            created_at: now,
            updated_at: now,
        })
    }

    /// 获取配置数据
    ///
    /// 将JSON数据反序列化为指定类型
    ///
    /// # 返回
    /// 成功返回反序列化后的对象，失败返回错误
    pub fn get_data<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_value(self.data.clone())?)
    }

    /// 更新配置数据
    ///
    /// # 参数
    /// * `data` - 新的数据对象
    ///
    /// # 返回
    /// 成功返回Ok(()), 失败返回错误
    pub fn set_data<T: Serialize>(&mut self, data: &T) -> Result<()> {
        self.data = serde_json::to_value(data)?;
        self.updated_at = Local::now();
        Ok(())
    }
}

/// 日期时间序列化和反序列化模块
mod datetime_format {
    use chrono::{DateTime, Local};
    use serde::{Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    /// 序列化日期时间为字符串
    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    /// 从字符串反序列化为日期时间
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        chrono::NaiveDateTime::parse_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
            .and_then(|naive| {
                naive
                    .and_local_timezone(Local)
                    .earliest()
                    .ok_or_else(|| serde::de::Error::custom("Failed to convert to local timezone"))
            })
    }
}
