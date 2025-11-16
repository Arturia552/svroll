/**
 * 配置管理模块
 * 负责配置的创建、验证和转换
 */
import type { ConnectConfig } from "@/types/mqttConfig"
import { convertType, isValidHexString } from "@/utils/typeConverter"

export class ConfigManager {
  /**
   * 默认配置类型定义
   */
  static readonly DEFAULT_TYPE_DEF: ConnectConfig = {
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
  }

  /**
   * 创建默认配置
   * @returns 默认配置对象
   */
  static createDefaultConfig(): ConnectConfig {
    return {
      sendData: "",
      protocol: "Mqtt",
      clients: [],
      threadSize: 100,
      enableRandom: false,
      broker: "",
      maxConnectPerSecond: 100,
      sendInterval: 1,
      fieldStruct: [],
      topicConfig: {
        data: {
          publish: {
            topic: null,
            qos: null,
            keyIndex: null,
          },
        },
        register: {
          publish: {
            topic: null,
            qos: null,
            keyIndex: null,
            extraKey: null,
          },
          subscribe: {
            topic: null,
            qos: null,
            keyIndex: null,
            extraKey: null,
          },
        },
      },
    }
  }

  /**
   * 验证并转换配置类型
   * @param config 配置对象
   */
  static validateAndConvertConfig(config: ConnectConfig): void {
    convertType(config, this.DEFAULT_TYPE_DEF)
  }

  /**
   * 检测并设置编辑器模式
   * @param config 配置对象
   * @returns 编辑器模式 ('json' | 'hex')
   */
  static detectEditorMode(config: ConnectConfig): "json" | "hex" {
    if (config.sendData && isValidHexString(config.sendData)) {
      return "hex"
    }
    return "json"
  }

  /**
   * 为运行时准备配置
   * @param config 配置对象
   * @param editorMode 编辑器模式
   */
  static prepareConfigForRuntime(config: ConnectConfig, editorMode: "json" | "hex"): void {
    this.validateAndConvertConfig(config)

    if (editorMode === "hex") {
      config.sendData = config.sendData.replace(/\s+/g, "")
    }
  }
}
