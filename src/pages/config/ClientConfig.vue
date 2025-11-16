<template>
  <div class="client-config config-component">
    <div class="actions-bar">
      <div class="buttons">
        <el-tooltip content="导入客户端文件" placement="top">
          <el-button type="primary" size="small" class="action-btn" @click="resolveClientFile">
            <el-icon>
              <Upload />
            </el-icon>
            导入文件
          </el-button>
        </el-tooltip>

        <el-popconfirm
          width="280"
          title="请输入要生成的客户端数量"
          confirm-button-text="生成"
          cancel-button-text="取消"
          @cancel="onCancel"
          @confirm="confirm">
          <template #reference>
            <el-button type="success" size="small" class="action-btn">
              <el-icon>
                <Plus />
              </el-icon>
              随机生成
            </el-button>
          </template>
          <template #actions="{ confirm: confirmAction, cancel: cancelAction }">
            <el-input-number
              v-model="generateSize"
              style="margin-bottom: 16px"
              :min="1"
              :max="10000"
              controls-position="right"
              placeholder="输入数量" />
            <div class="popconfirm-actions">
              <el-button size="small" @click="cancelAction">
                取消
              </el-button>
              <el-button type="primary" size="small" @click="confirmAction">
                生成
              </el-button>
            </div>
          </template>
        </el-popconfirm>
      </div>
    </div>

    <el-divider />

    <div ref="tableSectionRef" class="table-section">
      <div class="table-header">
        <el-input
          v-if="config.clients.length > 0"
          v-model="searchQuery"
          placeholder="搜索客户端"
          suffix-icon="Search"
          class="search-input" />
      </div>

      <el-table-v2
        :data="filteredClients"
        :columns="columns"
        :width="tableWidth"
        :height="ClientTableHeight"
        :row-height="35"
        fixed />
    </div>
  </div>
</template>
<script setup lang="ts">
import { ClientInfo, ConnectConfig } from "@/types/mqttConfig"
import { open } from "@tauri-apps/plugin-dialog"
import { invoke } from "@tauri-apps/api/core"
import { ElMessage } from "element-plus"
import { h, ref, computed, inject } from "vue"
import { Delete } from "@element-plus/icons-vue"
import { useWindowSize, useElementSize } from "@vueuse/core"

const generateSize = ref(100)
const config = inject<Ref<ConnectConfig>>("config")
if (!config) {
  throw new Error("Config not provided")
}
const searchQuery = ref("")
const tableSectionRef = ref<HTMLElement>()
const { width: tableSectionWidth } = useElementSize(tableSectionRef)
const { height } = useWindowSize()

const tableWidth = computed(() => {
  return tableSectionWidth.value || 1000
})

const ClientTableHeight = computed(() => {
  return Math.max(480, height.value - 480) // 调整为更小的高度，最小520px
})

// 定义表格列
const columns = computed(() => [
  {
    key: "clientId",
    dataKey: "clientId",
    title: "客户端ID",
    width: tableWidth.value * 0.2,
    sortable: true,
  },
  {
    key: "username",
    dataKey: "username",
    title: "用户名",
    width: tableWidth.value * 0.2,
  },
  {
    key: "password",
    dataKey: "password",
    title: "密码",
    width: tableWidth.value * 0.2,
  },
  {
    key: "identifyKey",
    dataKey: "identifyKey",
    title: "标识密钥",
    width: tableWidth.value * 0.28,
  },
  {
    key: "action",
    dataKey: "action",
    title: "操作",
    width: tableWidth.value * 0.1,
    cellRenderer: ({ rowIndex }) => {
      return h(
        "div",
        {
          style: { display: "flex", justifyContent: "center" },
        },
        [
          h(
            "el-button",
            {
              style: {
                border: "none",
                width: "20px",
                background: "transparent",
                color: "var(--el-color-danger)",
                cursor: "pointer",
              },
              onClick: () => removeClient(rowIndex),
            },
            [h(Delete)],
          ),
        ],
      )
    },
  },
])

const filteredClients = computed(() => {
  if (!searchQuery.value) return config.value.clients
  return config.value.clients.filter(
    (client) =>
      client.clientId.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      client.username.toLowerCase().includes(searchQuery.value.toLowerCase()),
  )
})

const onCancel = () => {
  // Do nothing on cancel
}

const confirm = () => {
  generateRandom(generateSize.value)
  ElMessage.success(`成功生成 ${generateSize.value} 个客户端`)
}

const resolveClientFile = async () => {
  try {
    const filePath = (await open({
      multiple: false,
      filters: [
        { name: "文本文件", extensions: ["txt", "csv"] },
        { name: "所有文件", extensions: ["*"] },
      ],
    })) as string

    if (filePath) {
      const clients: ClientInfo[] = await invoke("process_client_file", {
        filePath,
      })
      config.value.clients = clients
      ElMessage.success(`成功导入 ${clients.length} 个客户端`)
    }
  } catch (error) {
    ElMessage.error(`导入失败: ${error}`)
  }
}

const generateRandom = (size: number) => {
  const clients: ClientInfo[] = []
  for (let i = 0; i < size; i++) {
    clients.push({
      clientId: `client_${i}`,
      username: `user_${i}`,
      password: `password_${i}`,
      identifyKey: `key_${i}`,
    })
  }
  config.value.clients = clients
}

const removeClient = (index: number) => {
  config.value.clients.splice(index, 1)
}
</script>
<style scoped lang="scss">
// 特定于ClientConfig的样式，通用样式已在common.scss中定义
</style>
