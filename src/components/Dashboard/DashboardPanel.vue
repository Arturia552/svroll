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
            <el-progress :percentage="calculatePercentage('connected')" :stroke-width="8" :show-text="false"
                         status="success"
            />
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
            <el-progress :percentage="calculatePercentage('failed')" :stroke-width="8" :show-text="false"
                         status="exception"
            />
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
            <el-progress :percentage="calculatePercentage('connecting')" :stroke-width="8" :show-text="false"
                         status="warning"
            />
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="0" class="chart-container">
      <el-col :span="24">
        <el-card class="chart-card">
          <template #header>
            <div class="chart-header">
              <el-select v-model="timeRange" placeholder="时间范围" size="small">
                <el-option label="最近1分钟" value="1m" />
                <el-option label="最近5分钟" value="5m" />
                <el-option label="最近15分钟" value="15m" />
              </el-select>
            </div>
          </template>
          <div ref="chartRef" class="chart" />
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="0" style="gap: 20px">
      <el-card class="clients-card">
        <template #header>
          <div class="card-header">
            <span>客户端连接状态</span>
          </div>
        </template>
        <div ref="tableContainer" style="width: 100%; height: 300px;">
          <el-table-v2 :columns="columns" :data="clientsTableData" :width="tableWidth || 400" :height="tableHeight"
                       :row-height="40" fixed
          />
        </div>
      </el-card>

      <el-card class="log-card" style="flex: 1;">
        <template #header>
          <div class="card-header">
            <span>测试日志</span>
            <el-button type="text" size="small">
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
import { onMounted, ref, watch, onBeforeUnmount, PropType, h, inject } from 'vue';
import { ArrowUp } from '@element-plus/icons-vue';
import * as echarts from 'echarts/core';
import { LineChart } from 'echarts/charts';
import { GridComponent, TooltipComponent, TitleComponent, LegendComponent } from 'echarts/components';
import { CanvasRenderer } from 'echarts/renderers';
import { ElTableV2 } from 'element-plus';
import type { TableV2Props } from 'element-plus';
import { rs2JsEntity } from '@/types/mqttConfig';


echarts.use([GridComponent, LineChart, TooltipComponent, TitleComponent, LegendComponent, CanvasRenderer]);

const props = defineProps({
  counter: {
    type: Number,
    default: 0
  },
  clientInfo: {
    type: Object as PropType<any>,
    default: () => ({
      connected: 0,
      disconnected: 0,
      failed: 0,
      connecting: 0
    })
  },
  terminalLog: {
    type: Array as PropType<rs2JsEntity[]>,
    default: () => []
  }
});

const chartRef = ref<HTMLElement | null>(null);
const timeRange = ref('5m');
const messageChart = ref<echarts.ECharts | null>(null);
const messageRateHistory = ref<number[]>([]);
const lastCounter = ref(0);
const lastTime = ref(Date.now());
const ratePerSecond = ref(0);

// 模拟的客户端数据

const clientsTableData = inject<any>('clientConnectionInfo');

// 表格容器引用和尺寸
const tableContainer = ref<HTMLElement | null>(null);
const tableHeight = ref(300);
const tableWidth = ref(0);

// 定义列配置
const columns = ref<TableV2Props['columns']>([
  {
    key: 'clientId',
    dataKey: 'clientId',
    title: '客户端ID',
    width: 120,
  },
  {
    key: 'connectionState',
    dataKey: 'connectionState',
    title: '状态',
    width: 100,
    cellRenderer: ({ rowData }) => {
      let type, text;
      switch (rowData.connectionState) {
        case 'Connected':
          type = 'success';
          text = '已连接';
          break;
        case 'Connecting':
          type = 'warning';
          text = '连接中';
          break;
        case 'Failed':
          type = 'danger';
          text = '连接失败';
          break;
        case 'Disconnected':
          type = 'info';
          text = '已断开';
          break;
        default:
          type = 'info';
          text = '未知';
      }
      return h('el-tag', { type }, text);
    }
  },
  {
    key: 'username',
    dataKey: 'username',
    title: '用户名',
    width: 120,
  },
  {
    key: 'messages',
    dataKey: 'messages',
    title: '消息数',
    width: 100,
  },
  {
    key: 'lastSeen',
    dataKey: 'lastSeen',
    title: '最后活动时间',
    width: 180,
  }
]);


const calculatePercentage = (type: string): number => {
  if (!props.clientInfo) return 0;
  const total = props.clientInfo.connected + props.clientInfo.failed + props.clientInfo.connecting + props.clientInfo.disconnected;
  if (total === 0) return 0;
  return Math.round((props.clientInfo[type] / total) * 100);
};

const calculateRate = () => {
  return ratePerSecond.value.toFixed(1);
};

const initChart = () => {
  if (chartRef.value) {
    messageChart.value = echarts.init(chartRef.value);
    updateChart();
  }
};

const updateChart = () => {
  if (!messageChart.value) return;

  const now = new Date();
  const xAxisData = [];
  const times = messageRateHistory.value.length;

  for (let i = times - 1; i >= 0; i--) {
    const time = new Date(now.getTime() - i * 1000);
    xAxisData.push(`${time.getHours()}:${time.getMinutes().toString().padStart(2, '0')}:${time.getSeconds().toString().padStart(2, '0')}`);
  }

  const option = {
    title: {
      text: '',
      left: 'center'
    },
    tooltip: {
      trigger: 'axis',
      formatter: '{b}<br />发送速率: {c} 消息/秒'
    },
    xAxis: {
      type: 'category',
      data: xAxisData,
      axisLabel: {
        interval: Math.floor(messageRateHistory.value.length / 5)
      }
    },
    yAxis: {
      type: 'value',
      name: '消息/秒',
      min: 0
    },
    series: [
      {
        name: '发送速率',
        type: 'line',
        data: messageRateHistory.value,
        areaStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              {
                offset: 0,
                color: 'rgba(64, 158, 255, 0.6)'
              },
              {
                offset: 1,
                color: 'rgba(64, 158, 255, 0.1)'
              }
            ]
          }
        },
        lineStyle: {
          width: 2,
          color: '#409EFF'
        },
        symbol: 'circle',
        symbolSize: 6
      }
    ],
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      top: '8%',
      containLabel: true
    }
  };

  messageChart.value.setOption(option);
};

// 监控计数器变化，计算消息速率
watch(() => props.counter, (newVal) => {
  const currentTime = Date.now();
  const elapsedSeconds = (currentTime - lastTime.value) / 1000;

  if (elapsedSeconds > 0.5) {  // 至少间隔0.5秒更新一次速率
    ratePerSecond.value = (newVal - lastCounter.value) / elapsedSeconds;
    lastCounter.value = newVal;
    lastTime.value = currentTime;

    messageRateHistory.value.push(Math.round(ratePerSecond.value));
    if (messageRateHistory.value.length > 60) { // 保持最多60个数据点
      messageRateHistory.value.shift();
    }

    updateChart();
  }
});

// 窗口尺寸变化时调整图表大小
const handleResize = () => {
  messageChart.value?.resize();
};

onMounted(() => {
  initChart();
  lastTime.value = Date.now();
  window.addEventListener('resize', handleResize);

  // 初始化历史数据
  for (let i = 0; i < 10; i++) {
    messageRateHistory.value.push(0);
  }

  // 计算表格容器宽度
  if (tableContainer.value) {
    tableWidth.value = tableContainer.value.clientWidth;

    // 监听容器大小变化
    const resizeObserver = new ResizeObserver(entries => {
      for (const entry of entries) {
        tableWidth.value = entry.contentRect.width;
      }
    });

    resizeObserver.observe(tableContainer.value);

    // 清理观察器
    onBeforeUnmount(() => {
      resizeObserver.disconnect();
    });
  }
});


onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize);
  messageChart.value?.dispose();
});
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

  .metric-label {
    font-size: 14px;
    color: var(--el-text-color-secondary);
    margin-bottom: 8px;
  }

  .metric-value {
    font-size: 32px;
    font-weight: 600;
    margin-bottom: 16px;

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
