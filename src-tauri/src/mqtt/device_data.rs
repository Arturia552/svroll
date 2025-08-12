use chrono::Local;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// MQTT发送数据结构
/// 
/// 包含要发送的JSON数据和字段定义，用于构建MQTT消息内容
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttSendData {
    /// 要发送的实际数据，以JSON格式存储
    pub data: Value,
    /// 数据字段定义列表，在序列化时忽略
    #[serde(skip)]
    pub fields: Vec<MqttFieldStruct>,
}

impl MqttSendData {
    /// 获取数据内容
    /// 
    /// 返回数据的引用
    pub fn get_data(&self) -> &Value {
        &self.data
    }

    /// 设置数据字段定义
    /// 
    /// # 参数
    /// * `fields` - 字段定义列表
    pub fn set_fields(&mut self, fields: Vec<MqttFieldStruct>) {
        self.fields = fields;
    }
}

/// 处理字段值，根据字段类型和配置设置数据
/// 
/// 支持多种数据类型的处理，包括时间戳、日期时间、整数、浮点数、布尔值和枚举
/// 
/// # 参数
/// * `data` - 要处理的JSON数据
/// * `fields` - 字段定义列表
/// * `enable_random` - 是否启用随机值生成
pub fn process_fields(data: &mut Value, fields: &Vec<MqttFieldStruct>, enable_random: bool) {
    let mut rng = rand::thread_rng();
    fields.iter().for_each(|field| match field.field_type {
        FieldType::Timestamp => {
            let now = Local::now().timestamp_millis();
            data[&field.field_name] = Value::from(now);
        }
        FieldType::DateTime => {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            data[&field.field_name] = Value::from(now);
        }
        FieldType::Integer => {
            if enable_random {
                if field.max_value.is_none() && field.min_value.is_none() {
                    return;
                }
                let random_integer = rng.gen_range(field.min_value.unwrap() as i64..=field.max_value.unwrap() as i64);
                data[&field.field_name] = Value::from(random_integer);
            }
        }
        FieldType::Boolean => {
            if enable_random {
                let random_boolean = rng.gen_bool(0.5);
                data[&field.field_name] = Value::from(random_boolean);
            }
        }
        FieldType::Float => {
            if enable_random {
                if field.max_value.is_none() && field.min_value.is_none() {
                    return;
                }
                let random_float = rng.gen_range(field.min_value.unwrap()..=field.max_value.unwrap());
                data[&field.field_name] = Value::from(random_float);
            }
        }
        FieldType::Object => {
            let mut object = Value::Object(Default::default());
            process_fields(&mut object, &field.child.as_ref().unwrap(), enable_random);
            data[&field.field_name] = object;
        }
        FieldType::Enum => {
            if enable_random {
                if let Some(possible_values) = &field.possible_values {
                    if !possible_values.is_empty() {
                        let total_probability: f64 = possible_values.iter()
                            .map(|pv| pv.probability)
                            .sum();
                        
                        let random_value = rng.gen_range(0.0..total_probability);
                        
                        let mut cumulative_prob = 0.0;
                        for pv in possible_values {
                            cumulative_prob += pv.probability;
                            if random_value <= cumulative_prob {
                                data[&field.field_name] = Value::from(pv.value);
                                break;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    })
}

/// MQTT字段结构体
/// 
/// 描述字段的属性，包括名称、类型、取值范围等
/// 用于定义MQTT消息的数据结构
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttFieldStruct {
    /// 字段名称
    #[serde(rename = "fieldName")]
    pub field_name: String,
    /// 字段数据类型
    #[serde(rename = "fieldType")]
    pub field_type: FieldType,
    /// 最小值（对数值类型有效）
    #[serde(rename = "minValue")]
    pub min_value: Option<f64>,
    /// 最大值（对数值类型有效）
    #[serde(rename = "maxValue")]
    pub max_value: Option<f64>,
    /// 可能的取值列表（对枚举类型有效）
    #[serde(rename = "possibleValues")]
    pub possible_values: Option<Vec<PossibleValue>>,
    /// 子字段列表（对对象类型有效）
    #[serde(rename = "children", default)]
    pub child: Option<Vec<MqttFieldStruct>>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PossibleValue {
    pub value: u32,
    pub probability: f64,
}


/// 字段类型枚举
/// 
/// 定义支持的各种数据类型，用于指定字段的数据格式
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FieldType {
    /// 时间戳格式（毫秒级Unix时间戳）
    Timestamp,
    /// 字符串类型
    String,
    /// 整数类型
    Integer,
    /// 浮点数类型
    Float,
    /// 布尔类型
    Boolean,
    /// 日期时间格式（如：2023-01-01 12:34:56.789）
    DateTime,
    /// 仅日期格式
    Date,
    /// 仅时间格式
    Time,
    /// 枚举类型，从预定义的选项中选择
    Enum,
    /// 数组类型
    Array,
    /// 对象类型，包含子字段
    Object,
    /// 空值
    Null,
    /// 未定义类型
    Unknown,
}
