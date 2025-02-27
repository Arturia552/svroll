<template>
  <div class="client-config">
    <div class="actions-bar">
      <div class="title">
        客户端配置
      </div>
      <div class="buttons">
        <el-tooltip content="导入客户端文件" placement="top">
          <el-button type="primary" class="action-btn" @click="resolveClientFile">
            <el-icon><Upload /></el-icon>
            导入文件
          </el-button>
        </el-tooltip>
        
        <el-popconfirm 
          width="280" 
          title="请输入要生成的客户端数量" 
          confirm-button-text="生成" 
          cancel-button-text="取消"
          @cancel="onCancel"
          @confirm="confirm"
        >
          <template #reference>
            <el-button type="success" class="action-btn">
              <el-icon><Plus /></el-icon>
              随机生成
            </el-button>
          </template>
          <template #actions="{ confirm, cancel }">
            <el-input-number
              v-model="generateSize"
              style="margin-bottom: 16px;"
              :min="1"
              :max="10000"
              controls-position="right"
              placeholder="输入数量"
            />
            <div class="popconfirm-actions">
              <el-button size="small" @click="cancel">
                取消
              </el-button>
              <el-button type="primary" size="small" @click="confirm">
                生成
              </el-button>
            </div>
          </template>
        </el-popconfirm>
      </div>
    </div>
    
    <el-divider />
    
    <div class="table-section">
      <div class="table-header">
        <span>客户端列表 ({{ config.clients.length }})</span>
        <el-input
          v-if="config.clients.length > 0"
          v-model="searchQuery"
          placeholder="搜索客户端"
          suffix-icon="Search"
          class="search-input"
        />
      </div>
      
      <el-table
        :data="filteredClients"
        style="width: 100%"
        max-height="500"
        border
        stripe
      >
        <el-table-column prop="clientId" label="客户端ID" sortable />
        <el-table-column prop="username" label="用户名" />
        <el-table-column prop="password" label="密码" />
        <el-table-column label="操作" width="120">
          <template #default="scope">
            <el-button type="danger" link size="small" @click="removeClient(scope.$index)">
              <el-icon><Delete /></el-icon>
            </el-button>
          </template>
        </el-table-column>
        
        <template #empty>
          <div class="empty-data">
            <el-empty description="暂无客户端数据" />
          </div>
        </template>
      </el-table>
      
      <div v-if="config.clients.length > 0" class="pagination-container">
        <el-pagination
          background
          layout="prev, pager, next"
          :total="config.clients.length"
          :page-size="10"
        />
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { MqttClient, MqttConfig } from "@/types/mqttConfig";
import { open } from "@tauri-apps/api/dialog";
import { invoke } from "@tauri-apps/api/tauri";
import { ElMessage } from "element-plus";

const generateSize = ref(100);
const config = ref(inject<MqttConfig>("config"))
const searchQuery = ref('');

const filteredClients = computed(() => {
  if (!searchQuery.value) return config.value.clients;
  return config.value.clients.filter(client => 
    client.clientId.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
    client.username.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

const onCancel = () => {
  // Do nothing on cancel
};

const confirm = () => {
  generateRandom(generateSize.value);
  ElMessage.success(`成功生成 ${generateSize.value} 个客户端`);
}

const resolveClientFile = async () => {
  try {
    const filePath = (await open({
      multiple: false,
      filters: [
        { name: "文本文件", extensions: ["txt", "csv"] },
        { name: "所有文件", extensions: ["*"] }
      ],
    })) as string;

    if (filePath) {
      const clients: MqttClient[] = await invoke("process_client_file", { filePath });
      config.value.clients = clients;
      ElMessage.success(`成功导入 ${clients.length} 个客户端`);
    }
  } catch (error) {
    ElMessage.error(`导入失败: ${error}`);
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

const removeClient = (index: number) => {
  config.value.clients.splice(index, 1);
}
</script>
<style scoped lang="scss">
.client-config {
  padding: 16px;
}

.actions-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  
  .title {
    font-size: 18px;
    font-weight: 600;
    color: var(--el-text-color-primary);
  }
  
  .buttons {
    display: flex;
    gap: 12px;
    
    .action-btn {
      display: flex;
      align-items: center;
      gap: 6px;
      
      .el-icon {
        margin-right: 4px;
      }
    }
  }
}

.table-section {
  margin-top: 16px;
  
  .table-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    font-size: 16px;
    color: var(--el-text-color-regular);
    
    .search-input {
      width: 240px;
    }
  }
}

.popconfirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.pagination-container {
  margin-top: 16px;
  display: flex;
  justify-content: center;
}
</style>
