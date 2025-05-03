use std::fs::File;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tokio::fs;

/// 从JSON文件加载发送数据
///
/// # 参数
/// * `file_path` - JSON文件路径
///
/// # 返回
/// 成功返回解析后的数据对象，失败返回错误
pub async fn load_from_json_file<T>(file_path: &str) -> Result<T>
where
    T: DeserializeOwned + Debug,
{
    let contents = fs::read_to_string(file_path)
        .await
        .with_context(|| format!("Failed to read the file: {}", file_path))?;

    let msg: T = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?;

    Ok(msg)
}

/// 保存数据到JSON文件
///
/// # 参数
/// * `file_path` - 目标文件路径
/// * `data` - 要保存的数据对象
///
/// # 返回
/// 成功返回Ok，失败返回错误
pub async fn save_to_json_file<T>(file_path: &str, data: &T) -> Result<()>
where
    T: Serialize + Debug,
{
    let json = serde_json::to_string_pretty(data)
        .with_context(|| format!("Failed to convert data to JSON: {:?}", data))?;

    fs::write(file_path, json)
        .await
        .with_context(|| format!("Failed to write to file: {}", file_path))?;

    Ok(())
}

/// 从CSV文件读取数据并转换为结构体列表
///
/// # 参数
/// * `file_path` - CSV文件路径
///
/// # 返回
/// 成功返回结构体列表，失败返回错误
pub async fn read_from_csv_into_struct<C>(file_path: &str) -> Result<Vec<C>>
where
    C: DeserializeOwned + Debug,
{
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b',').from_reader(file);
    let mut csv_content_vec: Vec<C> = vec![];
    for result in rdr.deserialize::<C>() {
        let record = result?;
        csv_content_vec.push(record);
    }
    Ok(csv_content_vec)
}
