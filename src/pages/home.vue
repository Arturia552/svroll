<template>
  <div class="home">
    <!-- <theme-switch /> -->
    <el-card class="main-card">
      <div class="header">
        <div class="func-button-group">
          <el-button type="success"
                     size="default"
                     :disabled="isRunning"
                     @click="showNewConfig">
            <template #icon>
              <el-icon :size="16">
                <Edit />
              </el-icon>
            </template>编辑配置
          </el-button>
          <el-button type="primary" :disabled="!valid || isRunning" @click="exportConfig">
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
        <div v-if="!isRunning && !showDashboard" class="editor-wrapper">
          <div class="editor-controls">
            <el-button type="danger" :disabled="isRunning" @click="resetConfig">
              <template #icon>
                <el-icon :size="16">
                  <delete />
                </el-icon>
              </template>重置表单
            </el-button>
            <div class="editor-mode-controls">
              <el-button v-if="editorMode === 'hex'" type="primary" @click="formatHexText">
                格式化Hex
              </el-button>
              <el-radio-group v-model="editorMode">
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
                     size="large"
                     :disabled="!valid || isRunning"
                     @click="start">
            <template #icon>
              <el-icon :size="20">
                <VideoPlay />
              </el-icon>
            </template>
            开始
          </el-button>
          <el-button v-if="isRunning && showDashboard"
                     type="danger"
                     size="large"
                     :loading="stopping"
                     @click="stop">
            <template #icon>
              <el-icon :size="20">
                <VideoPause />
              </el-icon>
            </template>
            停止
          </el-button>
          <el-button v-if="showDashboard"
                     type="warning"
                     size="large"
                     :disabled="isRunning"
                     @click="returnToEditor">
            <el-icon>
              <back />
            </el-icon>返回编辑器
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
  <el-drawer v-model="configDrawerVisible"
             class="basic-drawer"
             direction="rtl"
             size="100%"
             :destroy-on-close="false"
             :wrapper-closable="true"
             :show-close="false"
             :with-header="false"
             :close-on-click-modal="false">
    <tabs-config v-if="configDrawerVisible"
                 v-model:config-form="config"
                 v-model:valid="valid"
                 @close="closeConfigDrawer"
                 @submit="handleConfigSubmit" />
  </el-drawer>

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
import { Clock, Back, Delete } from '@element-plus/icons-vue'

import { TauriService } from "@/services/tauriService"
import { EventManager, type EventCallbacks } from "@/services/eventManager"
import { TimerManager, type ClientInfoSummary } from "@/services/timerManager"
import { ConfigManager } from "@/services/configManager"

// 响应式状态
const valid = ref<boolean>(false)
const isRunning = ref<boolean>(false)
const clientInfo = ref<ClientInfoSummary>({
  connected: 0,
  disconnected: 0,
  failed: 0,
  connecting: 0
})

const terminalLog = ref<rs2JsEntity[]>([])
const clientConnectionInfo = ref<ClientInfo[]>([])
const editorMode = ref<'json' | 'hex'>("json")
const codeEditorRef = ref<InstanceType<typeof CodeEditor> | null>(null)
const stopping = ref<boolean>(false)
const showDashboard = ref<boolean>(false)
const config = ref<ConnectConfig>(ConfigManager.createDefaultConfig())

// 抽屉状态
const configDrawerVisible = ref<boolean>(false)
const historyDrawerVisible = ref<boolean>(false)
const counter = ref<number>(0)

// 服务实例
const eventManager = new EventManager()
const timerManager = new TimerManager()

// 依赖注入
provide("config", config)
provide("clientConnectionInfo", clientConnectionInfo)

const showNewConfig = () => {
  configDrawerVisible.value = true
}

const showHistory = () => {
  historyDrawerVisible.value = true
}

const handleHistoryConfigLoad = (historyConfig: ConnectConfig) => {
  config.value = historyConfig
  console.log(config.value)
  ConfigManager.validateAndConvertConfig(config.value)

  editorMode.value = ConfigManager.detectEditorMode(config.value)

  valid.value = true
  historyDrawerVisible.value = false
}

const exportConfig = async () => {
  try {
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
    valid.value = true
    ElMessage.success("导入成功")
  } catch (e) {
    ElMessage.error(String(e))
  }
}

const start = async () => {
  try {
    terminalLog.value = []
    counter.value = 0
    ConfigManager.prepareConfigForRuntime(config.value, editorMode.value)

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

const closeConfigDrawer = () => {
  configDrawerVisible.value = false
}

const handleConfigSubmit = () => {
  nextTick(() => {
    console.log("配置已提交，表单验证状态:", valid.value)
  })
}

// 格式化Hex文本
const formatHexText = () => {
  if (codeEditorRef.value && editorMode.value === 'hex') {
    codeEditorRef.value.formatHex()
  }
}

const resetConfig = () => {
  ElMessageBox.confirm('确定要清空当前配置吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  }).then(() => {
    config.value = ConfigManager.createDefaultConfig()
    valid.value = false
    editorMode.value = "json"
    ElMessage.success('配置已清空')
  }).catch(() => {
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
        valid.value = true
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
    }
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
    }
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
  padding: 10px;
  height: 100vh;
  /* 使用视口高度 */
  box-sizing: border-box;
  /* 确保padding不增加元素总高度 */
  display: flex;
  flex-direction: column;
  overflow: hidden;
  /* 防止出现滚动条导致布局抖动 */
  background-color: var(--el-bg-color-page);
}

.main-card {
  border-radius: var(--el-border-radius-base);
  box-shadow: var(--el-box-shadow);
  flex: 1;
  /* 占用所有可用空间 */
  display: flex;
  flex-direction: column;
  overflow: hidden;
  /* 防止内容溢出 */

  :deep(.el-card__body) {
    padding: 24px;
    display: flex;
    flex-direction: column;
    height: 100%;
    /* 确保卡片体占满卡片容器 */
    overflow: auto;
    /* 内容过多时允许滚动 */
  }
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  border-bottom: 1px solid var(--el-border-color-light);
  padding-bottom: 16px;
  flex-shrink: 0;
  /* 防止头部被压缩 */

  .title {
    margin: 0;
    color: var(--el-text-color-primary);
    font-weight: 600;
  }
}

.func-button-group {
  display: flex;
  gap: 12px;

  .el-button {
    display: flex;
    align-items: center;
    gap: 6px;
    transition: all 0.3s;

    &:hover {
      transform: translateY(-2px);
    }
  }
}

.editor-section {
  margin-bottom: 24px;
  flex: 1;
  /* 编辑区域占用剩余空间 */
  min-height: 200px;
  /* 设置最小高度 */
  display: flex;
  flex-direction: column;

  .section-title {
    font-size: 16px;
    font-weight: 500;
    margin-bottom: 12px;
    color: var(--el-text-color-regular);
  }
}

.editor-wrapper {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.editor-controls {
  display: flex;
  justify-content: space-between;
  padding: 0 0 10px 0;
}

.editor-mode-controls {
  display: flex;
  gap: 10px;
  align-items: center;
}

.json-edit-container {
  height: 100%;
  /* 使用100%高度填充父容器 */
  min-height: 260px;
  /* 保持最小高度 */
  border-radius: var(--el-border-radius-base);
  overflow: hidden;
  box-shadow: var(--el-box-shadow-light);
  flex: 1;
  /* 占用编辑区域的所有剩余空间 */
}

.dashboard-container {
  height: 100%;
  min-height: 260px;
  border-radius: var(--el-border-radius-base);
  overflow: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.control-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: auto;
  /* 将控制区域推到底部 */
  flex-shrink: 0;
  /* 防止控制区域被压缩 */

  .controls {
    display: flex;
    gap: 16px;

    .el-button {
      padding: 12px 24px;
      font-weight: 500;
      display: flex;
      align-items: center;
      gap: 8px;
      transition: all 0.3s;

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
      padding: 16px 24px;
      text-align: center;
      min-width: 150px;
      box-shadow: var(--el-box-shadow-light);

      .counter-label {
        font-size: 14px;
        color: var(--el-color-success);
        margin-bottom: 8px;
      }

      .counter-value {
        font-size: 28px;
        font-weight: 600;
        color: var(--el-color-primary);
      }
    }
  }
}
</style>
