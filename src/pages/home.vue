<template>
  <div class="home">
    <!-- <theme-switch /> -->
    <el-card class="main-card">
      <div class="header">
        <h2 class="title">
          IoT 连接测试工具
        </h2>
        <div class="func-button-group">
          <el-button type="success" size="default" @click="showNewConfig">
            <el-icon><Edit /></el-icon>编辑配置
          </el-button>
          <el-button type="primary" :disabled="!valid" @click="exportConfig">
            <el-icon><Download /></el-icon>导出配置
          </el-button>
          <el-button type="primary" @click="loadConfig">
            <el-icon><Upload /></el-icon>导入配置
          </el-button>
        </div>
      </div>

      <div class="editor-section">
        <code-editor
          v-model:jsonEdit="config.sendData"
          class="json-edit-container"
        />
      </div>

      <div class="control-section">
        <div class="controls">
          <el-button type="primary" size="large" :disabled="!valid" @click="start">
            <el-icon><VideoPlay /></el-icon>开始测试
          </el-button>
          <el-button type="danger" size="large" @click="stop">
            <el-icon><VideoPause /></el-icon>停止测试
          </el-button>
        </div>
        <div class="status-panel">
          <div class="counter-card">
            <div class="counter-label">
              消息计数
            </div>
            <div class="counter-value">
              {{ counter }}
            </div>
          </div>
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
import { invoke } from "@tauri-apps/api/tauri";
import CodeEditor from "@/components/CodeEditor/index.vue";
import TabsConfig from "@/pages/config/TabsConfig.vue";
import { MqttConfig, rs2JsEntity } from "@/types/mqttConfig";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/api/dialog";

const valid = ref<boolean>(false);
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

const exportConfig = () => {
  const jsonString = JSON.stringify(config.value, null, 2);
  const blob = new Blob([jsonString], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "config.json";
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
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
    ElMessage.success("导入成功");
  } catch (e) {
    ElMessage.error(String(e));
  }
};

const start = async () => {
  counter.value = 0;
  convertType(config.value, mqttConfigTypeDef);
  console.log(config.value)
  // const msg = await invoke("start_task", { param: config.value as MqttConfig });
  // ElMessage.success(msg);
  receive();
};

const stop = async () => {
  const msg = await invoke("stop_task");
  ElMessage.success(msg);
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
  listen("rs2js", (event) => {
    console.log(event);
    const entity: rs2JsEntity = JSON.parse(event.payload as string);
    if (entity.msgType === "counter") {
      counter.value = parseInt(entity.msg);
    }
  });
};
</script>
<style lang="scss" scoped>
.home {
  padding: 20px;
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
