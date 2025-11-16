/**
 * 事件管理模块
 * 负责处理 Tauri 事件监听和回调管理
 */
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import type { rs2JsEntity } from "@/types/mqttConfig"

export interface EventCallbacks {
  onCounter?: (count: number) => void
  onClientInfo?: (clientInfo: any) => void
  onTerminal?: (entity: rs2JsEntity) => void
}

export class EventManager {
  private unlisten: UnlistenFn | null = null

  /**
   * 开始监听事件
   * @param callbacks 事件回调函数
   */
  async startListening(callbacks: EventCallbacks): Promise<void> {
    // 如果已经在监听，先停止
    if (this.unlisten) {
      await this.stopListening()
    }

    this.unlisten = await listen("rs2js", async (event) => {
      try {
        const entity: rs2JsEntity = JSON.parse(event.payload as string)

        switch (entity.msgType) {
          case "counter":
            if (callbacks.onCounter) {
              callbacks.onCounter(parseInt(entity.msg))
            }
            break

          case "clientInfo":
            if (callbacks.onClientInfo) {
              try {
                const clientInfo = JSON.parse(entity.msg)
                callbacks.onClientInfo(clientInfo)
              } catch (e) {
                console.error("解析客户端信息失败:", e)
              }
            }
            break

          case "terminal":
            if (callbacks.onTerminal) {
              callbacks.onTerminal(entity)
            }
            break

          default:
            console.warn("未知的消息类型:", entity.msgType)
        }
      } catch (error) {
        console.error("解析事件数据失败:", error)
      }
    })
  }

  /**
   * 停止监听事件
   */
  async stopListening(): Promise<void> {
    if (this.unlisten) {
      this.unlisten()
      this.unlisten = null
    }
  }

  /**
   * 检查是否正在监听
   */
  isListening(): boolean {
    return this.unlisten !== null
  }
}
