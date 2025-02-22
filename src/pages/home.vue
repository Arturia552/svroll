<template>
  <div class="home">
    <div class="func-button-group">
      <el-button type="success" size="default" @click="showNewConfig"
        >编辑配置</el-button
      >
      <el-button type="primary" @click="exportConfig" :disabled="!valid"
        >导出配置</el-button
      >
    </div>
    <CodeEditor
      class="json-edit-container"
      v-model:jsonEdit="config.sendData"
    ></CodeEditor>

    <el-button type="primary" size="default" :disabled="!valid" @click="start">
      开始
    </el-button>
    <el-button type="danger" @click="stop">停止</el-button>
    {{ counter }}
  </div>
  <el-drawer
    class="basic-drawer"
    title="创建配置"
    v-model="configDrawerVisible"
    direction="rtl"
    size="100%"
    :destroy-on-close="false"
    :wrapperClosable="true"
    :show-close="false"
    :close-on-click-modal="false"
  >
    <NewConfig
      v-if="configDrawerVisible"
      v-model:valid="valid"
      @close="closeConfigDrawer"
      v-model:configForm="config"
    ></NewConfig>
  </el-drawer>
</template>
<script setup lang="ts" name="Home">
import { invoke } from "@tauri-apps/api/tauri";
import CodeEditor from "@/components/CodeEditor/index.vue";
import NewConfig from "@/pages/config/NewConfig.vue";
import { MqttConfig, rs2JsEntity } from "@/types/mqttConfig";
import { listen } from "@tauri-apps/api/event";

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

const start = async () => {
  counter.value = 0;
  convertType(config.value, mqttConfigTypeDef);
  console.log(config.value)
  const msg = await invoke("start_task", { param: config.value as MqttConfig });
  ElMessage.success(msg);
  receive();
};

const stop = async () => {
  const msg = await invoke("stop_task");
  ElMessage.success(msg);
};

const closeConfigDrawer = () => {
  configDrawerVisible.value = false;
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
    let entity: rs2JsEntity = JSON.parse(event.payload as string);
    if (entity.msgType === "counter") {
      counter.value = parseInt(entity.msg);
    }
  });
};
</script>
<style lang="scss" scoped>
.func-button-group {
  margin-bottom: 10px;
}

.json-edit-container {
  height: 200px;
}
</style>
