<template>
  <div>
    <el-button icon="Files" @click="resolveClientFile"></el-button>
    <el-popconfirm width="220" title="客户端数量" @cancel="onCancel" @confirm="confirm">
      <template #reference>
        <el-button>随机生成</el-button>
      </template>
      <template #actions="{ confirm, cancel }">
        <el-input style="margin-bottom: 10px;" size="small" v-model="generateSize" />
        <el-button size="small" @click="cancel">取消</el-button>
        <el-button type="danger" size="small" @click="confirm">
          生成
        </el-button>
      </template>
    </el-popconfirm>
    <el-table :data="config.clients" style="width: 100%">
      <el-table-column prop="clientId" label="客户端ID" />
      <el-table-column prop="username" label="用户名" />
      <el-table-column prop="password" label="密码" />
    </el-table>
  </div>
</template>
<script setup lang="ts">
import { MqttClient, MqttConfig } from "@/types/mqttConfig";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { ElMessage } from "element-plus";

const generateSize = ref(0);
const config = ref(inject<MqttConfig>("config"))
const onCancel = () => {
};

const confirm = () => {
  generateRandom(generateSize.value)
}

const resolveClientFile = async () => {
  try {
    const filePath = (await open({
      multiple: false,
      filters: [{ name: "All files", extensions: ["*"] }],
    })) as string;

    if (filePath) {
      const clients: MqttClient[] = await invoke("process_client_file", { filePath });
      config.value.clients = clients;
    }
  } catch (error) {
    ElMessage.error(error);
  }
}

const generateRandom = (size: number) => {
  const clients: MqttClient[] = [];
  for (let i = 0; i < size; i++) {
    clients.push({
      clientId: `client_${i}`,
      username: `user_${i}`,
      password: `password_${i}`
    });
  }
  config.value.clients = clients;
}

</script>
