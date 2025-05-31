/**
 * Tauri 服务模块
 * 负责处理所有与 Tauri 后端的通信
 */
import { invoke } from "@tauri-apps/api/core"
import { open, save } from "@tauri-apps/plugin-dialog"
import { writeTextFile } from "@tauri-apps/plugin-fs"
import type { ConnectConfig, ClientInfo } from "@/types/mqttConfig"

export class TauriService {
  /**
   * 开始任务
   * @param config 连接配置
   * @returns Promise<string>
   */
  static async startTask(config: ConnectConfig): Promise<string> {
    return await invoke("start_task", { param: config })
  }

  /**
   * 停止任务
   * @param protocol 协议类型
   * @returns Promise<string>
   */
  static async stopTask(protocol: string): Promise<string> {
    return await invoke("stop_task", { protocol })
  }

  /**
   * 获取客户端信息
   * @param protocol 协议类型
   * @returns Promise<ClientInfo[]>
   */
  static async getClients(protocol: string): Promise<ClientInfo[]> {
    return await invoke("get_clients", { protocol })
  }

  /**
   * 加载配置文件
   * @param filePath 文件路径
   * @returns Promise<ConnectConfig>
   */
  static async loadConfig(filePath: string): Promise<ConnectConfig> {
    return await invoke<ConnectConfig>("load_config", { filePath })
  }

  /**
   * 导出配置到文件
   * @param config 配置对象
   * @returns Promise<void>
   */
  static async exportConfig(config: ConnectConfig): Promise<void> {
    const filePath = await save({
      filters: [{ name: "JSON 文件", extensions: ["json"] }],
      title: "导出配置",
    })

    if (!filePath || filePath?.trim() === "") {
      throw new Error("未选择保存路径")
    }

    const content = JSON.stringify(config)
    await writeTextFile(filePath, content)
  }

  /**
   * 选择并导入配置文件
   * @returns Promise<string | null> 返回选择的文件路径
   */
  static async selectConfigFile(): Promise<string | null> {
    const filePath = await open({
      filters: [{ name: "JSON 文件", extensions: ["json"] }],
      title: "导入配置",
    })

    return filePath as string | null
  }
}
