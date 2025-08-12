export interface ConnectConfig {
  sendData?: string;
  clientFilePath?: string;
  protocol: string;
  clients?: ClientInfo[];
  threadSize?: number;
  enableRandom?: boolean;
  broker?: string;
  maxConnectPerSecond?: number;
  sendInterval?: number;
  fieldStruct?: JsonStruct[];
  topicConfig?: TopicConfig;
}

export const connectConfigTypeDef: ConnectConfig = {
  sendData: "",
  protocol: "",
  clients: [],
  threadSize: 0,
  enableRandom: false,
  broker: "",
  maxConnectPerSecond: 0,
  sendInterval: 0,
  fieldStruct: [],
  topicConfig: {
    data: {
      publish: {
        topic: "",
        qos: 0,
        keyIndex: 0,
      },
    },
  },
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
  id?: number;
  fieldName?: string;
  fieldType?: FieldTypeEnum;
  minValue?: number;
  maxValue?: number;
  possibleValues?: PossibleValue[];
  // For object type, nested children definitions
  children?: JsonStruct[];
}

export interface PossibleValue {
  value: any;  // 支持各种类型的值：number, string, boolean等
  probability: number;
}

export interface rs2JsEntity {
  msgType: 'counter' | 'clientInfo' | 'error' | 'terminal';
  msg: string;
  time: string;
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

export interface ClientInfo {
  clientId: string;
  username: string;
  password: string;
  connectionState?: ConnectionState
  identifyKey?: string;
}

export enum ConnectionState {
  Connected = "Connected",
  Connecting = "Connecting",
  Failed = "Failed",
}