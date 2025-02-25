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


export const mqttConfigTypeDef: MqttConfig = {
  sendData: "",
  protocol: "",
  clients: [],
  threadSize: 0,
  enableRegister: false,
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
    register: {
      publish: {
        topic: "",
        qos: 0,
        keyIndex: 0,
        extraKey: "",
      },
      subscribe: {
        topic: "",
        qos: 0,
        keyIndex: 0,
        extraKey: "",
      },
    },
  },
};

export function convert2Type(obj: any, typeDef: any): void {
  if (typeof obj !== typeof typeDef) {
    return;
  }
  if (typeof obj === "object" && obj !== null) {
    for (const key in typeDef) {
      if (typeDef.hasOwnProperty(key)) {
        if (typeof obj[key] !== typeof typeDef[key]) {
          // 强制转换类型
          if (typeof typeDef[key] === "number") {
            if(obj[key] === null) {
              obj[key] = undefined
            }else {
              obj[key] = Number(obj[key]);
            }
          } else if (typeof typeDef[key] === "string") {
            obj[key] = String(obj[key]);
          } else if (typeof typeDef[key] === "boolean") {
            obj[key] = Boolean(obj[key]);
          } else if (Array.isArray(typeDef[key])) {
            obj[key] = Array.isArray(obj[key]) ? obj[key] : [];
          } else if (typeof typeDef[key] === "object") {
            obj[key] = typeof obj[key] === "object" ? obj[key] : {};
          }
        }
        convert2Type(obj[key], typeDef[key]);
      }
    }
  }
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

