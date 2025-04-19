<template>
  <div class="history-container">
    <div class="header">
      <h2 class="title">
        历史配置
      </h2>
      <el-button type="danger" size="small" @click="clearHistory">
        <template #icon>
          <el-icon size="14">
            <delete />
          </el-icon>
        </template> 清空历史
      </el-button>
    </div>
    <el-empty v-if="historyList.length === 0 && loading === false" description="暂无历史记录" />
    <div v-else v-loading="loading" element-loading-background="initial" class="history-list">
      <div
        v-for="(item, index) in historyList" 
        :key="index" 
        class="history-item"
        @click="loadHistoryConfig(item.id)"
      >
        <div class="history-item-content">
          <div class="history-info">
            <div class="protocol-tag">
              <el-tag :type="getProtocolTagType(item.recordType)">
                {{ item.recordType }}
              </el-tag>
            </div>
            <div class="history-time">
              {{ item.created_at }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from 'element-plus';
import { Delete } from '@element-plus/icons-vue';
import "element-plus/es/components/message/style/css";
import "element-plus/es/components/message-box/style/css";

interface HistoryItem {
  id: number;
  recordType: string;
  data: object;
  created_at: string;
}

const loading = ref(true);
const historyList = ref<HistoryItem[]>([]);
const emit = defineEmits(['load-config']);

onMounted(async () => {
  loading.value = true;
  await fetchHistoryList();
});

const fetchHistoryList = async () => {
  try {
    historyList.value = await invoke<HistoryItem[]>("get_history_config");
  } catch (error) {
    ElMessage.error(`获取历史配置失败: ${error}`);
  } finally {
    loading.value = false;
  }
};

const loadHistoryConfig = async (id: number) => {
  try {
    const config = await invoke("load_history_config", { id });
    emit('load-config', config);
    ElMessage.success('历史配置加载成功');
  } catch (error) {
    ElMessage.error(`加载历史配置失败: ${error}`);
  }
};

const clearHistory = async () => {
  ElMessageBox.confirm('确认要清空所有历史记录吗？此操作不可恢复。', '警告', {
    confirmButtonText: '确认',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(async () => {
    try {
      await invoke("clear_history_config");
      historyList.value = [];
      ElMessage.success('历史记录已清空');
    } catch (error) {
      ElMessage.error(`清空历史记录失败: ${error}`);
    }
  }).catch(() => {
    // 用户取消操作
  });
};

const getProtocolTagType = (protocol: string): any => {
  const types: {[key: string]: string} = {
    'mqtt': 'success',
    'tcp': 'warning',
  };
  return types[protocol] || 'default';
};
</script>

<style lang="scss" scoped>
.history-container {
  padding: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
  
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    
    .title {
      margin: 0;
      font-size: 18px;
      font-weight: 500;
    }
  }
  
  .history-list {
    flex: 1;
    overflow-y: auto;
    
    .history-item {
      margin-bottom: 10px;
      margin-right: 20px;
      cursor: pointer;
      transition: all 0.3s;
      padding: 12px 15px;
      border-radius: 4px;
      background-color: var(--el-fill-color-light);
      border-left: 3px solid var(--el-color-primary);
      
      &:hover {
        transform: translateY(-2px);
        background-color: var(--el-fill-color);
        box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
      }
      
      .history-item-content {
        .history-info {
          display: flex;
          justify-content: space-between;
          align-items: center;
          
          .history-time {
            font-size: 12px;
            color: var(--el-text-color-secondary);
          }
        }
        
        .history-description {
          font-size: 14px;
          color: var(--el-text-color-regular);
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
      }
    }
  }
}
</style>
