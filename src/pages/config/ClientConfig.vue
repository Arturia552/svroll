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
        <el-input
          v-if="config.clients.length > 0"
          v-model="searchQuery"
          placeholder="搜索客户端"
          suffix-icon="Search"
          class="search-input"
        />
      </div>
      
      <div v-if="filteredClients.length === 0" class="empty-data">
        <el-empty description="暂无客户端数据" />
      </div>
      <el-table-v2
        v-else
        :data="filteredClients"
        :columns="columns"
        :width="tableWidth"
        :height="500"
        :row-height="40"
        fixed
      />
    </div>
  </div>
</template>
<script setup lang="ts">
import { ClientInfo, ConnectConfig } from "@/types/mqttConfig";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import { h, ref, computed, inject, onMounted } from 'vue';
import { Delete } from '@element-plus/icons-vue';

const generateSize = ref(100);
const config = ref(inject<ConnectConfig>("config"))
const searchQuery = ref('');
const tableWidth = ref(0);

// 根据窗口大小计算表格宽度
onMounted(() => {
  tableWidth.value = document.querySelector('.table-section')?.clientWidth || 1000;
  window.addEventListener('resize', () => {
    tableWidth.value = document.querySelector('.table-section')?.clientWidth || 1000;
  });
});

// 定义表格列
const columns = computed(() => [
  {
    key: 'clientId',
    dataKey: 'clientId',
    title: '客户端ID',
    width: tableWidth.value * 0.28,
    sortable: true,
  },
  {
    key: 'username',
    dataKey: 'username',
    title: '用户名',
    width: tableWidth.value * 0.3,
  },
  {
    key: 'password',
    dataKey: 'password',
    title: '密码',
    width: tableWidth.value * 0.3,
  },
  {
    key: 'action',
    dataKey: 'action',
    title: '操作',
    width: tableWidth.value * 0.1,
    cellRenderer: ({ rowIndex }) => {
      return h('div', {
        style: { display: 'flex', justifyContent: 'center' }
      }, [
        h('el-button', {
          style: {
            border: 'none',
            width: '20px',
            background: 'transparent',
            color: 'var(--el-color-danger)',
            cursor: 'pointer',
          },
          onClick: () => removeClient(rowIndex)
        }, [
          h(Delete)
        ])
      ]);
    }
  }
]);

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
      const clients: ClientInfo[] = await invoke("process_client_file", { filePath });
      config.value.clients = clients;
      ElMessage.success(`成功导入 ${clients.length} 个客户端`);
    }
  } catch (error) {
    ElMessage.error(`导入失败: ${error}`);
  }
}

const generateRandom = (size: number) => {
  const clients: ClientInfo[] = [];
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

.actions-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  
  .title {
    font-size: 18px;
    font-weight: 600;
    color: var(--el-color-primary);
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
  width: 100%;
  
  .table-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 16px;
    color: var(--el-text-color-regular);
    margin-bottom: 16px;
    
    .search-input {
      width: 240px;
    }
  }

  .empty-data {
    display: flex;
    justify-content: center;
    padding: 40px 0;
  }
}

.popconfirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
