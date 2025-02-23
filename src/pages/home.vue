<template>
  <div class="home">
    <div class="func-button-group">
      <el-button type="success" size="default" @click="showNewConfig">
        编辑配置
      </el-button>
      <el-button type="primary" @click="exportConfig">
        导出配置
      </el-button>
    </div>
    <code-editor v-model:jsonEdit="config.sendData" class="json-edit-container" />

    <el-button type="primary" size="default" :disabled="!valid" @click="start">
      开始
    </el-button>
    <el-button type="danger" @click="stop">
      停止
    </el-button>
    {{ counter }}
  </div>
  <el-drawer v-model="configDrawerVisible" class="basic-drawer" title="创建配置" direction="rtl" size="100%"
             :destroy-on-close="false" :wrapper-closable="true" :show-close="false" :close-on-click-modal="false"
  >
    <new-config v-if="configDrawerVisible" v-model:valid="valid" v-model:configForm="config"
                @close="closeConfigDrawer"
    />
  </el-drawer>
</template>
<script setup lang="ts" name="Home">
import { invoke } from "@tauri-apps/api/tauri";
import CodeEditor from "@/components/CodeEditor/index.vue";
import NewConfig from "@/pages/config/NewConfig.vue";
import { convert2Type, MqttConfig, mqttConfigTypeDef, rs2JsEntity } from "@/types/mqttConfig";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/api/dialog";
import { writeTextFile } from "@tauri-apps/api/fs";

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
});

provide("config", config);

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
  if (!filePath || filePath?.trim() === "") return;
  const content = JSON.stringify(config.value);
  try {
    await writeTextFile(filePath, content);
    ElMessage.success("导出成功");
  } catch (e) {
    ElMessage.error(e);
  }
};

const start = async () => {
  counter.value = 0;
  convert2Type(config.value, mqttConfigTypeDef);
  console.log(config.value);
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
.func-button-group {
  margin-bottom: 10px;
}

.json-edit-container {
  height: 200px;
}
</style>
