<template>
  <div class="dashboard-panel">
    <el-row :gutter="0" class="dashboard-cards">
      <el-col :xs="24" :sm="12" :md="5">
        <el-card class="metric-card">
          <div class="metric-label">
            消息总数
          </div>
          <div class="metric-value primary">
            {{ counter }}
          </div>
          <div class="metric-chart">
            <div class="trend-indicator positive">
              <el-icon><arrow-up /></el-icon>
              <span>{{ calculateRate() }}/秒</span>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :md="5">
        <el-card class="metric-card">
          <div class="metric-label">
            连接成功
          </div>
          <div class="metric-value success">
            {{ clientInfo?.connected || 0 }}
          </div>
          <div class="metric-chart">
            <el-progress
              :percentage="calculatePercentage('connected')"
              :stroke-width="8"
              :show-text="false"
              status="success" />
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :md="5">
        <el-card class="metric-card">
          <div class="metric-label">
            连接失败
          </div>
          <div class="metric-value danger">
            {{ clientInfo?.failed || 0 }}
          </div>
          <div class="metric-chart">
            <el-progress
              :percentage="calculatePercentage('failed')"
              :stroke-width="8"
              :show-text="false"
              status="exception" />
          </div>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="12" :md="5">
        <el-card class="metric-card">
          <div class="metric-label">
            连接中
          </div>
          <div class="metric-value warning">
            {{ clientInfo?.connecting || 0 }}
          </div>
          <div class="metric-chart">
            <el-progress
              :percentage="calculatePercentage('connecting')"
              :stroke-width="8"
              :show-text="false"
              status="warning" />
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="0" style="gap: 20px; flex: 1">
      <el-card class="clients-card">
        <template #header>
          <div class="card-header">
            <span>客户端连接状态</span>
          </div>
        </template>
        <div ref="tableContainer" style="width: 100%; height: 100%">
          <el-table-v2
            :columns="columns"
            :data="clientsTableData"
            :width="tableWidth || 500"
            :height="tableHeight"
            :row-height="40"
            fixed />
        </div>
      </el-card>

      <el-card class="log-card" style="flex: 1">
        <template #header>
          <div class="card-header">
            <span>测试日志</span>
            <el-button link size="small" type="danger">
              清除
            </el-button>
          </div>
        </template>
        <div class="log-content">
          <div v-for="(log, index) in terminalLog" :key="index" class="log-item">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-message">{{ log.msg }}</span>
          </div>
        </div>
      </el-card>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch, PropType, h, inject, computed } from "vue"
import { useResizeObserver, useWindowSize } from "@vueuse/core"
import { ArrowUp } from "@element-plus/icons-vue"
import { ElTableV2 } from "element-plus"
import type { TableV2Props } from "element-plus"
import { rs2JsEntity } from "@/types/mqttConfig"
import { ElTag } from "element-plus"

const props = defineProps({
  counter: {
    type: Number,
    default: 0,
  },
  clientInfo: {
    type: Object as PropType<any>,
    default: () => ({
      connected: 0,
      disconnected: 0,
      failed: 0,
      connecting: 0,
    }),
  },
  terminalLog: {
    type: Array as PropType<rs2JsEntity[]>,
    default: () => [],
  },
})

const lastCounter = ref(0)
const lastTime = ref(Date.now())
const ratePerSecond = ref(0)

const clientsTableData = inject<any>("clientConnectionInfo")
const { height } = useWindowSize()
const tableContainer = ref<HTMLElement | null>(null)
const tableWidth = ref(0)

const tableHeight = computed(() => {
  return height.value - 500
})

// 定义列配置
const columns = ref<TableV2Props["columns"]>([
  {
    key: "clientId",
    dataKey: "clientId",
    title: "客户端ID",
    width: 120,
  },
  {
    key: "username",
    dataKey: "username",
    title: "用户名",
    width: 120,
  },
  {
    key: "connectionState",
    dataKey: "connectionState",
    title: "状态",
    width: 100,
    cellRenderer: ({ rowData }) => {
      let type, text
      switch (rowData.connectionState) {
        case "Connected":
          type = "success"
          text = "已连接"
          break
        case "Connecting":
          type = "warning"
          text = "连接中"
          break
        case "Failed":
          type = "danger"
          text = "连接失败"
          break
        case "Disconnected":
          type = "info"
          text = "已断开"
          break
        default:
          type = "info"
          text = "未知"
      }
      return h(ElTag, { type: type }, () => text)
    },
  },
])

const calculatePercentage = (type: string): number => {
  if (!props.clientInfo) return 0
  const total =
    props.clientInfo.connected +
    props.clientInfo.failed +
    props.clientInfo.connecting +
    props.clientInfo.disconnected
  if (total === 0) return 0
  return Math.round((props.clientInfo[type] / total) * 100)
}

const calculateRate = () => {
  return ratePerSecond.value.toFixed(1)
}

// 监控计数器变化，计算消息速率
watch(
  () => props.counter,
  (newVal) => {
    const currentTime = Date.now()
    const elapsedSeconds = (currentTime - lastTime.value) / 1000

    if (elapsedSeconds > 0.5) {
      // 至少间隔0.5秒更新一次速率
      ratePerSecond.value = (newVal - lastCounter.value) / elapsedSeconds
      lastCounter.value = newVal
      lastTime.value = currentTime
    }
  },
)

onMounted(() => {
  lastTime.value = Date.now()

  // 计算表格容器宽度
  if (tableContainer.value) {
    tableWidth.value = tableContainer.value.clientWidth

    useResizeObserver(tableContainer, (entries) => {
      const entry = entries[0]
      if (entry) {
        tableWidth.value = entry.contentRect.width
      }
    })
  }
})
</script>

<style scoped lang="scss">
:deep(.el-card__body) {
  padding: 16px !important;
}

.dashboard-panel {
  padding: 10px 0;
  height: 100%;
  overflow-y: auto;
}

.section-title {
  font-size: 18px;
  font-weight: 500;
  margin: 0 0 16px;
  color: var(--el-text-color-primary);
}

.dashboard-cards {
  gap: 20px;
}

.metric-card {
  padding: 10px;
  height: 140px; // 固定高度确保一致性
  display: flex;
  flex-direction: column;

  :deep(.el-card__body) {
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }

  .metric-label {
    font-size: 14px;
    color: var(--el-text-color-secondary);
    margin-bottom: 8px;
    flex-shrink: 0;
  }

  .metric-value {
    font-size: 32px;
    font-weight: 600;
    margin-bottom: 16px;
    flex-shrink: 0;
    line-height: 1.2;

    &.primary {
      color: var(--el-color-primary);
    }

    &.success {
      color: var(--el-color-success);
    }

    &.warning {
      color: var(--el-color-warning);
    }

    &.danger {
      color: var(--el-color-danger);
    }
  }

  .metric-chart {
    flex: 1;
    display: flex;
    align-items: flex-end;

    .trend-indicator {
      display: flex;
      align-items: center;
      font-size: 14px;

      &.positive {
        color: var(--el-color-success);
      }

      &.negative {
        color: var(--el-color-danger);
      }

      .el-icon {
        margin-right: 4px;
      }
    }

    :deep(.el-progress) {
      width: 100%;
    }
  }
}

.chart-container {
  .chart-card {
    height: 300px;

    .chart-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .chart {
      height: 200px;
    }
  }
}

.clients-card,
.log-card {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .el-tag {
      margin-left: 8px;
    }
  }
}

.log-content {
  overflow-y: auto;

  .log-item {
    padding: 4px 0;
    border-bottom: 1px dashed var(--el-border-color-lighter);
    font-size: 12px;

    &:last-child {
      border-bottom: none;
    }

    &.info {
      color: var(--el-text-color-regular);
    }

    &.error {
      color: var(--el-color-danger);
    }

    &.warning {
      color: var(--el-color-warning);
    }

    .log-time {
      color: var(--el-text-color-secondary);
      margin-right: 10px;
    }
  }
}
</style>
