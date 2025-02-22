export interface MqttConfig {
  sendData?: string;
  clientFilePath?: string;
  protocol: string;
  clients?: MqttClient[];
  threadSize?: number;
  enableRegister?: boolean;
  enableRandom?: boolean;
  broker?: string;
  maxConnectPerSecond?: number;
  sendInterval?: number;
  fieldStruct?: JsonStruct[];
  topicConfig?: TopicConfig;
}

export interface TopicConfig {
  register?: topicWrap;
  data?: topicWrap;
}

export interface topicWrap {
  publish: TopicInfo,
  subscribe?: TopicInfo
}

export interface TopicInfo {
  keyIndex?: number;
  topic: string;
  qos: number;
  extraKey?: string;
}



export interface JsonStruct {
  fieldName?: string;
  fieldType?: FieldTypeEnum;
  minValue?: number;
  maxValue?: number;
  possibleValues?: any;
  children?: JsonStruct[];
}

export interface rs2JsEntity {
  msgType: string;
  msg: string;
}

export enum FieldTypeEnum {
  Timestamp = "Timestamp",
  String = "String",
  Integer = "Integer",
  Float = "Float",
  Boolean = "Boolean",
  DateTime = "DateTime",
  Date = "Date",
  Time = "Time",
  Enum = "Enum",
  Array = "Array",
  Object = "Object",
  Null = "Null",
  Unknown = "Unknown",
}

export interface MqttClient {
  clientId: string;
  username: string;
  password: string;
}