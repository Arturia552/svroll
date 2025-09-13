<template>
  <div class="basic-config config-component">
    <el-form ref="basicFormRef"
             :rules="rules"
             :model="config"
             label-position="top"
             :inline="false"
             :show-message="false"
             class="basic-form"
             size="small">
      <div class="form-section">
        <div class="section-title">
          协议设置
        </div>
        <el-divider />
        <el-form-item label="协议类型" prop="protocol">
          <el-radio-group v-model="config.protocol">
            <el-radio-button label="MQTT" value="Mqtt" />
            <el-radio-button label="TCP" value="Tcp" />
          </el-radio-group>
        </el-form-item>
      </div>
      <div class="form-section">
        <div class="section-title">
          连接设置
        </div>
        <el-divider />
        <el-form-item label="broker地址"
                      prop="broker"
                      :rules="[
                        {
                          required: true,
                          message: '请输入broker地址',
                          trigger: 'blur',
                        },
                      ]">
          <el-input v-model="config.broker" />
        </el-form-item>

        <el-form-item v-if="config.protocol === 'Mqtt'"
                      class="topic"
                      label="消息主题"
                      prop="topicConfig.data">
          <div class="topic-container">
            <template v-for="(key, value) in config.topicConfig.data" :key="value">
              <div v-if="config.topicConfig.data[value]" class="topic-row">
                <div class="form-content">
                  <template v-for="(column, index) in tableColumn" :key="`data-${column.label}-${index}`">
                    <el-select v-if="column.prop === 'qos'"
                               v-model="config.topicConfig.data[value][column.prop]"
                               :placeholder="column.label"
                               style="width: 20%">
                      <el-option label="0" value="0" />
                      <el-option label="1" value="1" />
                      <el-option label="2" value="2" />
                    </el-select>
                    <el-input v-else
                              v-model="config.topicConfig.data[value][column.prop]"
                              :placeholder="column.label" />
                  </template>
                </div>
              </div>
            </template>
          </div>
        </el-form-item>
      </div>

      <div class="form-section">
        <div class="section-title">
          测试参数
        </div>
        <el-divider />
        <div class="params-grid">
          <el-form-item label="随机生成" prop="enableRandom">
            <el-radio-group v-model="config.enableRandom">
              <el-radio-button label="关闭" :value="false" />
              <el-radio-button label="启动" :value="true" />
            </el-radio-group>
          </el-form-item>

          <el-form-item label="线程数" prop="threadSize">
            <el-input-number v-model.number="config.threadSize"
                             :min="1"
                             :max="1000"
                             controls-position="right" />
          </el-form-item>

          <el-form-item label="每秒连接请求数" prop="maxConnectPerSecond">
            <el-input-number v-model.number="config.maxConnectPerSecond"
                             :min="1"
                             :max="1000"
                             controls-position="right" />
          </el-form-item>

          <el-form-item label="数据发送间隔" prop="sendInterval">
            <el-input-number v-model.number="config.sendInterval"
                             :min="1"
                             :max="60"
                             controls-position="right" />
          </el-form-item>
        </div>
      </div>
    </el-form>
  </div>
</template>

<script setup lang="ts" name="BasicConfig">
import { ConnectConfig } from "@/types/mqttConfig"
import { FormRules, type FormInstance } from "element-plus"
import { getNestedValue, isJsonValueNull } from "@/hooks/processJsonStruct"
import type { Ref } from "vue"

const config = inject<Ref<ConnectConfig>>("config")
if (!config) {
  throw new Error("Config not provided")
}
const basicFormRef = ref<FormInstance>()

// 验证主题配置
const validateTopic = (rule: any, value: any, callback: any) => {
  if (
    isJsonValueNull(getNestedValue(config.value, rule.field), [
      "keyIndex",
      "extraKey",
      "subscribe",
    ])
  ) {
    callback(new Error("请完善主题"))
  }
  callback()
}

const msgColumns = [
  { label: "主题", prop: "topic" },
  { label: "QoS", prop: "qos" },
  { label: "Key索引", prop: "keyIndex" },
  { label: "额外Key", prop: "extraKey" },
]

const tableColumn = computed(() => {
  return msgColumns.filter((column) => column.prop !== "extraKey")
})

const rules = ref<FormRules>({
  broker: [{ required: true, message: "请输入服务端地址", trigger: "blur" }],
  "topicConfig.data": [{ validator: validateTopic, trigger: "blur" }],
})

const validateForm = () => {
  return new Promise<boolean>((resolve) => {
    basicFormRef.value?.validate((valid) => {
      resolve(valid)
    })
  })
}

defineExpose({
  validateForm,
})
</script>

<style scoped lang="scss">
// 特定于BasicConfig的样式
.basic-form {
  max-width: 100%;
}

.topic {
  :deep(.el-form-item__content) {
    row-gap: 8px;
    width: 100%;
  }

  .topic-container {
    border: 1px solid var(--el-border-color-light);
    border-radius: 4px;
    padding: 12px;
    width: 100%;
    background-color: var(--el-bg-color);
  }

  .topic-row {
    .form-label {
      font-size: 12px;
      margin-bottom: 6px;
      font-weight: 500;
      color: var(--el-text-color-regular);
    }

    .form-content {
      display: flex;
      gap: 8px;

      .el-input {
        flex: 1;
      }
    }
  }
}
</style>
