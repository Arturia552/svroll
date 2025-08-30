<template>
  <div class="home">
    <!-- <theme-switch /> -->
    <div class="main-container">
      <div class="header">
        <div class="config-tabs-group">
          <el-button :type="activeConfigTab === 'basic' ? 'primary' : 'default'
                     "
                     size="default"
                     :disabled="isRunning"
                     class="config-tab-button"
                     @click="setActiveConfigTab('basic')">
            <template #icon>
              <el-icon :size="16">
                <setting />
              </el-icon>
            </template>基础配置
            <el-tag v-if="tabsConfigRef?.formErrors?.basic"
                    type="danger"
                    size="small"
                    effect="dark"
                    class="error-badge">
              !
            </el-tag>
          </el-button>
          <el-button :type="activeConfigTab === 'client' ? 'primary' : 'default'
                     "
                     size="default"
                     :disabled="isRunning"
                     class="config-tab-button"
                     @click="setActiveConfigTab('client')">
            <template #icon>
              <el-icon :size="16">
                <cellphone />
              </el-icon>
            </template>客户端配置
            <el-tag v-if="tabsConfigRef?.formErrors?.client"
                    type="danger"
                    size="small"
                    effect="dark"
                    class="error-badge">
              !
            </el-tag>
          </el-button>
          <el-button v-if="config.protocol === 'Mqtt'"
                     :type="activeConfigTab === 'data' ? 'primary' : 'default'
                     "
                     size="default"
                     :disabled="isRunning"
                     class="config-tab-button"
                     @click="setActiveConfigTab('data')">
            <template #icon>
              <el-icon :size="16">
                <document />
              </el-icon>
            </template>数据配置
            <el-tag v-if="tabsConfigRef?.formErrors?.data"
                    type="danger"
                    size="small"
                    effect="dark"
                    class="error-badge">
              !
            </el-tag>
          </el-button>
        </div>
        <div class="func-button-group">
          <el-button type="primary"
                     size="default"
                     :disabled="isRunning"
                     @click="exportConfig">
            <template #icon>
              <el-icon :size="16">
                <upload />
              </el-icon>
            </template>导出配置
          </el-button>
          <el-button type="primary" :disabled="isRunning" @click="loadConfig">
            <template #icon>
              <el-icon :size="16">
                <download />
              </el-icon>
            </template>导入配置
          </el-button>
          <el-button type="info" :disabled="isRunning" @click="showHistory">
            <template #icon>
              <el-icon :size="16">
                <clock />
              </el-icon>
            </template>历史配置
          </el-button>
        </div>
      </div>

      <div class="editor-section">
        <div v-show="!isRunning && !showDashboard" class="content-wrapper">
          <div class="config-wrapper">
            <tabs-config ref="tabsConfigRef" v-model:config-form="config" v-model:active-tab="activeConfigTab" />
          </div>
          <div class="editor-wrapper">
            <div class="editor-controls">
              <el-button type="danger"
                         :disabled="isRunning"
                         size="small"
                         @click="resetConfig">
                <template #icon>
                  <el-icon :size="16">
                    <delete />
                  </el-icon>
                </template>重置表单
              </el-button>
              <div class="editor-mode-controls">
                <el-button v-if="editorMode === 'hex'"
                           type="primary"
                           size="small"
                           @click="formatHexText">
                  格式化Hex
                </el-button>
                <el-radio-group v-model="editorMode" size="small">
                  <el-radio-button value="json">
                    JSON
                  </el-radio-button>
                  <el-radio-button value="hex">
                    HEX
                  </el-radio-button>
                </el-radio-group>
              </div>
            </div>
            <code-editor ref="codeEditorRef"
                         v-model:jsonEdit="config.sendData"
                         class="json-edit-container"
                         :language="editorMode" />
          </div>
        </div>
        <dashboard-panel v-if="showDashboard"
                         :counter="counter"
                         :terminal-log="terminalLog"
                         :client-info="clientInfo"
                         class="dashboard-container"
                         @return-to-editor="returnToEditor" />
      </div>

      <div class="control-section">
        <div class="controls">
          <el-button v-if="!isRunning || !showDashboard"
                     type="primary"
                     size="default"
                     :disabled="isRunning"
                     @click="start">
            <template #icon>
              <el-icon :size="20">
                <video-play />
              </el-icon>
            </template>
            开始
          </el-button>
          <el-button v-if="isRunning && showDashboard"
                     type="danger"
                     size="default"
                     :loading="stopping"
                     @click="stop">
            <template #icon>
              <el-icon :size="20">
                <video-pause />
              </el-icon>
            </template>
            停止
          </el-button>
          <el-button v-if="showDashboard"
                     type="warning"
                     size="default"
                     :disabled="isRunning"
                     @click="returnToEditor">
            <template #icon>
              <el-icon>
                <back />
              </el-icon>
            </template>返回编辑器
          </el-button>
        </div>
      </div>
    </div>
  </div>
  <el-drawer v-model="historyDrawerVisible"
             title="历史配置"
             direction="rtl"
             size="400px"
             :with-header="false"
             destroy-on-close>
    <history-component @load-config="handleHistoryConfigLoad" />
  </el-drawer>
</template>
<script setup lang="ts" name="Home">
import CodeEditor from "@/components/CodeEditor/index.vue"
import TabsConfig from "@/pages/config/TabsConfig.vue"
import DashboardPanel from "@/components/Dashboard/DashboardPanel.vue"
import HistoryComponent from "@/components/History/index.vue"
import { ConnectConfig, rs2JsEntity, ClientInfo } from "@/types/mqttConfig"
import {
  Clock,
  Back,
  Delete,
  Upload,
  Download,
  VideoPlay,
  VideoPause,
  Setting,
  Cellphone,
  Document,
} from "@element-plus/icons-vue"
import { syncFieldStructFromEditorData } from "@/hooks/processJsonStruct"
import { TauriService } from "@/services/tauriService"
import { EventManager, type EventCallbacks } from "@/services/eventManager"
import { TimerManager, type ClientInfoSummary } from "@/services/timerManager"
import { ConfigManager } from "@/services/configManager"

// 响应式状态
const isRunning = ref<boolean>(false)
const activeConfigTab = ref<"basic" | "client" | "data">("basic")
const clientInfo = ref<ClientInfoSummary>({
  connected: 0,
  disconnected: 0,
  failed: 0,
  connecting: 0,
})

const terminalLog = ref<rs2JsEntity[]>([])
const clientConnectionInfo = ref<ClientInfo[]>([])
const editorMode = ref<"json" | "hex">("json")
const codeEditorRef = ref<InstanceType<typeof CodeEditor> | null>(null)
const tabsConfigRef = ref<InstanceType<typeof TabsConfig> | null>(null)
const stopping = ref<boolean>(false)
const showDashboard = ref<boolean>(false)
const config = ref<ConnectConfig>(ConfigManager.createDefaultConfig())

// 抽屉状态
const historyDrawerVisible = ref<boolean>(false)
const counter = ref<number>(0)

// 服务实例
const eventManager = new EventManager()
const timerManager = new TimerManager()

// 依赖注入
provide("config", config)
provide("clientConnectionInfo", clientConnectionInfo)

const setActiveConfigTab = (tab: "basic" | "client" | "data") => {
  // 切换tab前先同步fieldStruct
  syncFieldStructFromEditor()
  activeConfigTab.value = tab

  // 在下一个tick中刷新DataModel，确保组件已经渲染
  nextTick(() => {
    if (tabsConfigRef.value) {
      tabsConfigRef.value.refreshDataModel()
    }
  })
}

const syncFieldStructFromEditor = () => {
  try {
    // 只有在MQTT协议下才需要同步fieldStruct
    if (config.value.protocol !== "Mqtt") {
      return
    }
    config.value.fieldStruct = syncFieldStructFromEditorData(
      config.value.sendData || "{}",
      config.value.fieldStruct || [],
    )
  } catch (error) {
    console.warn("同步编辑器数据到fieldStruct失败:", error)
    // 不显示错误信息给用户，因为这是后台同步操作
  }
}

const showHistory = () => {
  historyDrawerVisible.value = true
}

const handleHistoryConfigLoad = (historyConfig: ConnectConfig) => {
  config.value = historyConfig
  ConfigManager.validateAndConvertConfig(config.value)

  editorMode.value = ConfigManager.detectEditorMode(config.value)

  historyDrawerVisible.value = false
}

const exportConfig = async () => {
  try {
    // 在导出前先进行配置校验
    const isValid = await tabsConfigRef.value?.validateForm()

    if (!isValid) {
      ElMessage.warning("请检查配置项并修正错误后再导出")
      return
    }

    ConfigManager.validateAndConvertConfig(config.value)
    await TauriService.exportConfig(config.value)
    ElMessage.success("导出成功")
  } catch (e) {
    ElMessage.error(String(e))
  }
}

const loadConfig = async () => {
  try {
    const filePath = await TauriService.selectConfigFile()
    if (!filePath || filePath === "") return

    config.value = await TauriService.loadConfig(filePath)
    ConfigManager.validateAndConvertConfig(config.value)

    editorMode.value = ConfigManager.detectEditorMode(config.value)
    ElMessage.success("导入成功")
  } catch (e) {
    ElMessage.error(String(e))
  }
}

const start = async () => {
  try {
    // 在开始前先进行配置校验
    const isValid = await tabsConfigRef.value?.validateForm()

    if (!isValid) {
      ElMessage.warning("请检查配置项并修正错误后再开始")
      return
    }

    terminalLog.value = []
    counter.value = 0
    ConfigManager.prepareConfigForRuntime(config.value, editorMode.value)
    console.log("开始任务，配置：", config.value)
    isRunning.value = true
    showDashboard.value = true

    // 开始监听事件
    startEventListening()

    // 开始定时获取客户端信息
    startClientInfoTimer()

    const msg = await TauriService.startTask(config.value)
    ElMessage.success(msg)
  } catch (e) {
    ElMessage.error(String(e))
    isRunning.value = false
  }
}

const stop = async () => {
  stopping.value = true
  try {
    const msg = await TauriService.stopTask(config.value.protocol)
    ElMessage.success(msg)
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    isRunning.value = false
    stopping.value = false
    stopAllServices()
  }
}

// 新增返回编辑器视图的方法
const returnToEditor = () => {
  showDashboard.value = false
}

// 格式化Hex文本
const formatHexText = () => {
  if (codeEditorRef.value && editorMode.value === "hex") {
    codeEditorRef.value.formatHex()
  }
}

const resetConfig = () => {
  ElMessageBox.confirm("确定要清空当前配置吗？", "提示", {
    confirmButtonText: "确定",
    cancelButtonText: "取消",
    type: "warning",
  })
    .then(() => {
      config.value = ConfigManager.createDefaultConfig()
      editorMode.value = "json"
      ElMessage.success("配置已清空")
    })
    .catch(() => {
      // 用户取消操作
    })
}

// 事件监听相关函数
const startEventListening = () => {
  const callbacks: EventCallbacks = {
    onCounter: (count: number) => {
      counter.value = count
      if (!isRunning.value) {
        isRunning.value = true
        stopping.value = false
        showDashboard.value = true
      }
    },
    onClientInfo: (info: any) => {
      clientInfo.value = info
    },
    onTerminal: (entity: rs2JsEntity) => {
      terminalLog.value.push(entity)
      if (terminalLog.value.length > 100) {
        terminalLog.value.shift()
      }
    },
  }

  eventManager.startListening(callbacks)
}

// 定时器相关函数
const startClientInfoTimer = () => {
  timerManager.startClientInfoTimer(
    config.value.protocol,
    (summary: ClientInfoSummary, clients: ClientInfo[]) => {
      clientInfo.value = summary
      clientConnectionInfo.value = clients
    },
  )
}

const stopAllServices = () => {
  timerManager.stop()
  eventManager.stopListening()
}

onMounted(() => {
  // 组件挂载时开始监听事件
  startEventListening()
})

onUnmounted(() => {
  // 组件卸载时清理资源
  stopAllServices()
})
</script>
<style lang="scss" scoped>
.home {
  width: 100vw;
  height: 100vh;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--el-bg-color-page);
}

.main-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  /* 允许flex子元素收缩 */
  background-color: var(--el-bg-color);
  border-radius: var(--el-border-radius-base);
  border: 1px solid var(--el-border-color-light);
  padding: 16px;
  box-sizing: border-box;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--el-border-color-light);
  flex-shrink: 0;
  min-width: 0;
  /* 允许flex子元素收缩 */
  overflow: hidden;
  /* 防止内容溢出 */

  .title {
    margin: 0;
    color: var(--el-text-color-primary);
    font-weight: 600;
  }
}

.config-tabs-group {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  /* 允许换行 */
  min-width: 0;
  /* 允许收缩 */

  .config-tab-button {
    position: relative;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: all 0.3s;
    font-size: 13px;
    padding: 6px 12px;
    min-width: 0;
    /* 允许收缩 */
    flex-shrink: 1;
    /* 允许收缩 */

    .error-badge {
      position: absolute;
      top: -6px;
      right: -6px;
      min-width: 16px;
      height: 16px;
      line-height: 16px;
      padding: 0 4px;
      font-size: 10px;
      border-radius: 8px;
    }
  }
}

.func-button-group {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  /* 允许换行 */
  min-width: 0;
  /* 允许收缩 */

  .el-button {
    display: flex;
    align-items: center;
    gap: 4px;
    transition: all 0.3s;
    font-size: 13px;
    padding: 6px 12px;
    min-width: 0;
    /* 允许收缩 */
    flex-shrink: 1;
    /* 允许收缩 */

    &:hover {
      transform: translateY(-2px);
    }
  }
}

.editor-section {
  margin-bottom: 16px;
  flex: 1;
  min-height: 0;
  /* 关键：允许flex子元素收缩 */
  display: flex;
  flex-direction: column;

  .section-title {
    font-size: 16px;
    font-weight: 500;
    margin-bottom: 12px;
    color: var(--el-text-color-regular);
  }
}

.content-wrapper {
  flex: 1;
  display: flex;
  gap: 16px;
  min-height: 0;
  /* 关键：允许flex子元素收缩 */
  overflow: hidden;
  /* 防止内容溢出 */
}

.config-wrapper {
  width: 50%;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  border-radius: var(--el-border-radius-base);
  overflow: auto;
  padding: 8px;
  background: var(--el-bg-color-page);
  border: 1px solid var(--el-border-color-light);
  min-width: 0;
  /* 允许收缩 */
}

.editor-wrapper {
  flex: 1;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  /* 允许收缩 */
}

.editor-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 0 8px 0;
  flex-wrap: wrap;
  /* 允许换行 */
  gap: 8px;
  min-width: 0;
  /* 允许收缩 */
}

.editor-mode-controls {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
  /* 允许换行 */
  min-width: 0;
  /* 允许收缩 */
}

.json-edit-container {
  flex: 1;
  min-height: 200px;
  border-radius: var(--el-border-radius-base);
  overflow: hidden;
  box-shadow: var(--el-box-shadow-light);
}

.dashboard-container {
  flex: 1;
  min-height: 200px;
  border-radius: var(--el-border-radius-base);
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.control-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
  flex-shrink: 0;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color-light);
  flex-wrap: wrap;
  /* 允许换行 */
  gap: 12px;
  min-width: 0;
  /* 允许收缩 */

  .controls {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
    /* 允许换行 */
    min-width: 0;
    /* 允许收缩 */

    .el-button {
      padding: 10px 20px;
      font-weight: 500;
      display: flex;
      align-items: center;
      gap: 6px;
      transition: all 0.3s;
      min-width: 0;
      /* 允许收缩 */
      flex-shrink: 1;
      /* 允许收缩 */

      &:hover {
        transform: translateY(-2px);
      }

      .el-icon {
        margin-right: 4px;
      }
    }
  }

  .status-panel {
    .counter-card {
      background-color: var(--el-fill-color-light);
      border-radius: var(--el-border-radius-base);
      padding: 12px 16px;
      text-align: center;
      min-width: 120px;
      box-shadow: var(--el-box-shadow-light);

      .counter-label {
        font-size: 12px;
        color: var(--el-color-success);
        margin-bottom: 6px;
      }

      .counter-value {
        font-size: 24px;
        font-weight: 600;
        color: var(--el-color-primary);
      }
    }
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .home {
    padding: 4px;
  }

  .main-container {
    padding: 8px;
  }

  .header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .config-tabs-group,
  .func-button-group {
    justify-content: center;
  }

  .content-wrapper {
    flex-direction: column;
    gap: 12px;
  }

  .config-wrapper,
  .editor-wrapper {
    width: 100%;
  }

  .control-section {
    justify-content: center;
  }
}
</style>
