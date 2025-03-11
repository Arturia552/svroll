<template>
  <div class="home">
    <!-- <theme-switch /> -->
    <el-card class="main-card">
      <div class="header">
        <div class="func-button-group">
          <el-button type="success" size="default" :disabled="isRunning" @click="showNewConfig">
            <el-icon><Edit /></el-icon>编辑配置
          </el-button>
          <el-button type="primary" :disabled="!valid || isRunning" @click="exportConfig">
            <el-icon><Download /></el-icon>导出配置
          </el-button>
          <el-button type="primary" :disabled="isRunning" @click="loadConfig">
            <el-icon><Upload /></el-icon>导入配置
          </el-button>
        </div>
      </div>

      <div class="editor-section">
        <!-- 根据测试状态显示编辑器或仪表盘 -->
        <code-editor
          v-if="!isRunning"
          v-model:jsonEdit="config.sendData"
          class="json-edit-container"
        />
        <dashboard-panel
          v-else
          :counter="counter"
          :client-info="clientInfo"
          class="dashboard-container"
        />
      </div>

      <div class="control-section">
        <div class="controls">
          <el-button type="primary" size="large" :disabled="!valid || isRunning" @click="start">
            <el-icon><VideoPlay /></el-icon>开始测试
          </el-button>
          <el-button type="danger" size="large" @click="stop">
            <el-icon><VideoPause /></el-icon>停止测试
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
  <el-drawer
    v-model="configDrawerVisible"
    class="basic-drawer"
    direction="rtl"
    size="100%"
    :destroy-on-close="false"
    :wrapper-closable="true"
    :show-close="false"
    :close-on-click-modal="false"
  >
    <tabs-config
      v-if="configDrawerVisible"
      v-model:config-form="config"
      v-model:valid="valid"
      @close="closeConfigDrawer"
      @submit="handleConfigSubmit"
    />
  </el-drawer>
</template>
<script setup lang="ts" name="Home">
import { invoke } from "@tauri-apps/api/core";
import CodeEditor from "@/components/CodeEditor/index.vue";
import TabsConfig from "@/pages/config/TabsConfig.vue";
import DashboardPanel from "@/components/Dashboard/DashboardPanel.vue";
import { convert2Type, MqttConfig, rs2JsEntity } from "@/types/mqttConfig";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

const valid = ref<boolean>(false);
const isRunning = ref<boolean>(false);
const clientInfo = ref<any>({
  connected: 0,
  disconnected: 0,
  failed: 0,
  connecting: 0
});
const clientConnectionInfo = ref<any>([])
const config = ref<MqttConfig>({
  sendData: "",
  protocol: "Mqtt",
  clients: [],
  threadSize: 100,
  enableRegister: false,
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
        extraKey: null
      },
      subscribe: {
        topic: null,
        qos: null,
        keyIndex: null,
        extraKey: null
      },
    }
  },
});

provide("config", config);
provide("clientConnectionInfo", clientConnectionInfo)

const mqttConfigTypeDef: MqttConfig = {
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
        extraKey: ""
      },
      subscribe: {
        topic: "",
        qos: 0,
        keyIndex: 0,
        extraKey: ""
      },
    }
  },
};
const configDrawerVisible = ref<boolean>(false);
const counter = ref<number>(0);

const showNewConfig = () => {
  configDrawerVisible.value = true;
};

const exportConfig = async () => {
  const filePath = await save({
    filters: [{ name: "JSON 文件", extensions: ["json"] }],
    title: "导出配置",
  });
  convert2Type(config.value, mqttConfigTypeDef);
  if (!filePath || filePath?.trim() === "") return;
  const content = JSON.stringify(config.value);
  try {
    await writeTextFile(filePath, content);
    ElMessage.success("导出成功");
  } catch (e) {
    ElMessage.error(e);
  }
};


const loadConfig = async () => {
  const filePath = await open({
    filters: [{ name: "JSON 文件", extensions: ["json"] }],
    title: "导入配置",
  });
  if (!filePath || filePath === "") return;
  try {
    config.value = await invoke<MqttConfig>("load_config", { filePath });    
    convertType(config.value, mqttConfigTypeDef);
    console.log(config.value)
    valid.value = true;
    ElMessage.success("导入成功");
  } catch (e) {
    ElMessage.error(String(e));
  }
};

const start = async () => {
  counter.value = 0;
  convertType(config.value, mqttConfigTypeDef);
  console.log(config.value)
  const msg = await invoke("start_task", { param: config.value as MqttConfig });
  ElMessage.success(msg);
  isRunning.value = true;
  receive();
};

const stop = async () => {
  const msg = await invoke("stop_task");
  ElMessage.success(msg);
  isRunning.value = false;
};

const closeConfigDrawer = () => {
  configDrawerVisible.value = false;
};

// 此方法可以保留，但主要通过v-model:valid实现状态传递
const handleConfigSubmit = () => {
  // valid.value已经通过v-model双向绑定自动更新
  nextTick(() => {
    console.log("配置已提交，表单验证状态:", valid.value);
  })
};

function convertType(obj: any, typeDef: any): void {
  if (typeof obj !== typeof typeDef) {
    return;
  }

  if (typeof obj === "object" && obj !== null) {
    for (const key in typeDef) {
      if (typeDef.hasOwnProperty(key)) {
        if (typeof obj[key] !== typeof typeDef[key]) {
          // 强制转换类型
          if (typeof typeDef[key] === "number") {
            obj[key] = Number(obj[key]);
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
        convertType(obj[key], typeDef[key]);
      }
    }
  }
}

const receive = () => {
  listen("rs2js", async (event) => {
    const entity: rs2JsEntity = JSON.parse(event.payload as string);
    if (entity.msgType === "counter") {
      counter.value = parseInt(entity.msg);
    } else if (entity.msgType === "clientInfo") {
      try {
        clientInfo.value = JSON.parse(entity.msg);
      } catch (e) {
        console.error("解析客户端信息失败:", e);
      }
    }
    console.log(event.payload);
    const clients = await invoke("get_mqtt_clients")
    clientConnectionInfo.value = clients
  });
};
</script>
<style lang="scss" scoped>
.home {
  padding: 10px;
  height: 100vh; /* 使用视口高度 */
  box-sizing: border-box; /* 确保padding不增加元素总高度 */
  display: flex;
  flex-direction: column;
  overflow: hidden; /* 防止出现滚动条导致布局抖动 */
  background-color: var(--el-bg-color-page);
}

.main-card {
  border-radius: var(--el-border-radius-base);
  box-shadow: var(--el-box-shadow);
  flex: 1; /* 占用所有可用空间 */
  display: flex;
  flex-direction: column;
  overflow: hidden; /* 防止内容溢出 */
  
  :deep(.el-card__body) {
    padding: 24px;
    display: flex;
    flex-direction: column;
    height: 100%; /* 确保卡片体占满卡片容器 */
    overflow: auto; /* 内容过多时允许滚动 */
  }
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  border-bottom: 1px solid var(--el-border-color-light);
  padding-bottom: 16px;
  flex-shrink: 0; /* 防止头部被压缩 */
  
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
  flex: 1; /* 编辑区域占用剩余空间 */
  min-height: 200px; /* 设置最小高度 */
  display: flex;
  flex-direction: column;
  
  .section-title {
    font-size: 16px;
    font-weight: 500;
    margin-bottom: 12px;
    color: var(--el-text-color-regular);
  }
}

.json-edit-container {
  height: 100%; /* 使用100%高度填充父容器 */
  min-height: 260px; /* 保持最小高度 */
  border-radius: var(--el-border-radius-base);
  overflow: hidden;
  box-shadow: var(--el-box-shadow-light);
  flex: 1; /* 占用编辑区域的所有剩余空间 */
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
  margin-top: auto; /* 将控制区域推到底部 */
  flex-shrink: 0; /* 防止控制区域被压缩 */
  
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
