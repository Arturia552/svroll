/**
 * 定时器管理模块
 * 负责管理客户端信息的定时获取
 */
import { TauriService } from "./tauriService"
import type { ClientInfo } from "@/types/mqttConfig"

export interface ClientInfoSummary {
  connected: number
  disconnected: number
  failed: number
  connecting: number
}

export class TimerManager {
  private timerId: number | null = null
  private protocol: string = ""
  private onClientInfoUpdate?: (summary: ClientInfoSummary, clients: ClientInfo[]) => void

  /**
   * 开始定时获取客户端信息
   * @param protocol 协议类型
   * @param callback 回调函数
   * @param interval 间隔时间（毫秒），默认500ms
   */
  startClientInfoTimer(
    protocol: string,
    callback: (summary: ClientInfoSummary, clients: ClientInfo[]) => void,
    interval: number = 500,
  ): void {
    this.stop() // 确保先停止已有的定时器
    this.protocol = protocol
    this.onClientInfoUpdate = callback

    this.timerId = window.setInterval(() => {
      this.fetchClientInfo()
    }, interval)
  }

  /**
   * 停止定时器
   */
  stop(): void {
    if (this.timerId !== null) {
      clearInterval(this.timerId)
      this.timerId = null
    }
  }

  /**
   * 检查定时器是否正在运行
   */
  isRunning(): boolean {
    return this.timerId !== null
  }

  /**
   * 获取客户端信息
   */
  private async fetchClientInfo(): Promise<void> {
    try {
      const clients = await TauriService.getClients(this.protocol)
      const summary = this.calculateClientSummary(clients)

      if (this.onClientInfoUpdate) {
        this.onClientInfoUpdate(summary, clients)
      }
    } catch (error) {
      console.error("获取客户端信息失败:", error)
    }
  }
  /**
   * 计算客户端状态摘要
   * @param clients 客户端列表
   * @returns 客户端状态摘要
   */
  private calculateClientSummary(clients: ClientInfo[]): ClientInfoSummary {
    const summary: ClientInfoSummary = {
      connected: 0,
      disconnected: 0,
      failed: 0,
      connecting: 0,
    }

    clients.forEach((client: ClientInfo) => {
      switch (client.connectionState) {
        case "Connected":
          summary.connected += 1
          break
        case "Failed":
          summary.failed += 1
          break
        case "Connecting":
          summary.connecting += 1
          break
        default:
          summary.disconnected += 1
      }
    })

    return summary
  }
}
