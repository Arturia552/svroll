use chrono::Local;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttSendData {
    pub data: Value,
    #[serde(skip)]
    pub fields: Vec<MqttFieldStruct>,
}

impl MqttSendData {
    pub fn get_data(&self) -> &Value {
        &self.data
    }

    pub fn set_fields(&mut self, fields: Vec<MqttFieldStruct>) {
        self.fields = fields;
    }

    pub fn process_fields(&mut self, enable_random: bool) {
        let mut rng = rand::thread_rng();
        self.fields
            .iter()
            .for_each(|field| match field.field_type {
                FieldType::Timestamp => {
                    let now = Local::now().timestamp_millis();
                    self.data[&field.field_name] = Value::from(now);
                },
                FieldType::DateTime => {
                    let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
                    self.data[&field.field_name] = Value::from(now);
                },
                FieldType::Integer => {
                    if enable_random {
                        let random_integer = rng.gen_range(field.min_value as i64..=field.max_value as i64);
                        self.data[&field.field_name] = Value::from(random_integer);
                    }
                },
                FieldType::Boolean => {
                    if enable_random {
                        let random_boolean = rng.gen_bool(0.5);
                        self.data[&field.field_name] = Value::from(random_boolean);
                    }
                },
                FieldType::Float => {
                    if enable_random {
                        let random_float = rng.gen_range(field.min_value..=field.max_value);
                        self.data[&field.field_name] = Value::from(random_float);
                    }

                }
                _ => {}
            })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttFieldStruct {
    #[serde(rename = "fieldName")]
    pub field_name: String,
    #[serde(rename = "fieldType")]
    pub field_type: FieldType,
    #[serde(rename = "minValue")]
    pub min_value: f64,
    #[serde(rename = "maxValue")]
    pub max_value: f64,
    #[serde(rename = "possibleValues")]
    pub possible_values: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FieldType {
    Timestamp,
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Date,
    Time,
    Enum,
    Array,
    Object,
    Null,
    Unknown,
}
