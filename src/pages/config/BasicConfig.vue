<template>
  <div class="basic-config">
    <el-form
      ref="basicFormRef"
      :rules="rules"
      :model="config"
      label-position="top"
      :inline="false"
      class="basic-form"
      size="default"
    >
      <div class="form-section">
        <h3 class="section-title">
          协议设置
        </h3>
        <el-divider />
        <el-form-item label="协议类型" prop="protocol">
          <el-radio-group v-model="config.protocol">
            <el-radio-button label="MQTT" value="Mqtt" />
            <el-radio-button label="TCP" value="Tcp" />
          </el-radio-group>
        </el-form-item>
        
        <el-form-item label="注册包机制" prop="enableRegister">
          <el-radio-group v-model="config.enableRegister">
            <el-radio-button label="关闭" :value="false" />
            <el-radio-button label="启动" :value="true" />
          </el-radio-group>
        </el-form-item>
        
        <el-form-item
          v-if="config.enableRegister && config.protocol === 'Mqtt'"
          class="topic"
          label="主题配置"
          prop="topicConfig.register"
        >
          <el-card class="topic-card">
            <template v-for="(key, value) in config.topicConfig.register" :key="value">
              <div v-if="config.topicConfig.register[value]" class="topic-row">
                <div class="form-label">
                  {{ value === "publish" ? "发布" : "订阅" }}
                </div>
                <div class="form-content">
                  <template v-for="(column, index) in tableColumn" :key="`reg-${column.label}-${index}`">
                    <el-input v-model="config.topicConfig.register[value][column.prop]" :placeholder="column.label" />
                  </template>
                </div>
              </div>
            </template>
          </el-card>
        </el-form-item>
      </div>


      <div class="form-section">
        <h3 class="section-title">
          连接设置
        </h3>
        <el-divider />
        <el-form-item
          label="broker地址"
          prop="broker"
          :rules="[
            { required: true, message: '请输入broker地址', trigger: 'blur' },
          ]"
        >
          <el-input v-model="config.broker" placeholder="如 mqtt://broker.example.com:1883">
            <template #append>
              <el-button @click="validateUrl">
                测试连通
              </el-button>
            </template>
          </el-input>
        </el-form-item>
        
        <el-form-item v-if="config.protocol === 'Mqtt'" class="topic" label="消息主题" prop="topicConfig.data">
          <el-card class="topic-card">
            <template v-for="(key, value) in config.topicConfig.data" :key="value">
              <div v-if="config.topicConfig.data[value]" class="topic-row">
                <div class="form-label">
                  {{ value === "publish" ? "发布" : "订阅" }}
                </div>
                <div class="form-content">
                  <template v-for="(column, index) in tableColumn" :key="`data-${column.label}-${index}`">
                    <el-input v-model="config.topicConfig.data[value][column.prop]" :placeholder="column.label" />
                  </template>
                </div>
              </div>
            </template>
          </el-card>
        </el-form-item>
      </div>
      
      <div class="form-section">
        <h3 class="section-title">
          测试参数
        </h3>
        <el-divider />
        <div class="params-grid">
          <el-form-item label="随机生成" prop="enableRandom">
            <el-radio-group v-model="config.enableRandom">
              <el-radio-button label="关闭" :value="false" />
              <el-radio-button label="启动" :value="true" />
            </el-radio-group>
          </el-form-item>
          
          <el-form-item label="线程数" prop="threadSize">
            <el-input-number v-model.number="config.threadSize" :min="1" :max="1000" controls-position="right" />
          </el-form-item>
          
          <el-form-item label="每秒连接请求数" prop="maxConnectPerSecond">
            <el-input-number v-model.number="config.maxConnectPerSecond" :min="1" :max="1000" controls-position="right" />
          </el-form-item>
          
          <el-form-item label="数据发送间隔" prop="sendInterval">
            <el-input-number v-model.number="config.sendInterval" :min="1" :max="60" controls-position="right" />
          </el-form-item>
        </div>
      </div>
    </el-form>
  </div>
</template>

<script setup lang="ts" name="BasicConfig">
import { MqttConfig } from '@/types/mqttConfig';
import { ElMessage, FormRules, type FormInstance } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import { getNestedValue, isJsonValueNull } from '@/hooks/processJsonStruct';

const config = ref(inject<MqttConfig>("config"));
const basicFormRef = ref<FormInstance>();

// 验证主题配置
const validateTopic = (rule: any, value: any, callback: any) => {
  if (
    isJsonValueNull(getNestedValue(config.value, rule.field), [
    "keyIndex","extraKey","subscribe"
    ])
  ) {
    callback(new Error("请完善主题"));
  }
  callback();
};

const tableColumn = ref([
  { label: "主题", prop: "topic", },
  { label: "QoS", prop: "qos", },
  { label: "Key索引", prop: "keyIndex", },
  { label: "额外Key", prop: "extraKey" },
]);

const rules = ref<FormRules>({
  broker: [{ required: true, message: "请输入broker地址", trigger: "blur" }],
  "topicConfig.data": [{ validator: validateTopic, trigger: "blur" }],
  "topicConfig.register": [{ validator: validateTopic, trigger: "blur" }],
});

// 验证URL
const validateUrl = async () => {
  try {
    const msg = await invoke("validate_mqtt_url", {
      url: config.value.broker,
    });
    ElMessage.success(msg);
  } catch (error) {
    ElMessage.error(String(error));
  }
};

// 对外暴露验证表单方法
const validateForm = () => {
  return new Promise<boolean>((resolve) => {
    basicFormRef.value?.validate((valid) => {
        
      resolve(valid);
    });
  });
};

defineExpose({
  validateForm
});
</script>

<style scoped lang="scss">
.basic-config {
  width: 100%;
}

.basic-form {
  max-width: 800px;
  margin: 0 auto;
}

.form-section {
  margin-bottom: 32px;
  
  .section-title {
    margin-top: 0;
    margin-bottom: 8px;
    color: var(--el-color-primary);
    font-size: 18px;
    font-weight: 500;
  }
}

.params-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
}

.topic-card {
  box-shadow: var(--el-box-shadow-light);
  margin-top: 8px;
}

.topic-row {
  display: flex;
  margin-bottom: 16px;
  align-items: center;
  
  &:last-child {
    margin-bottom: 0;
  }
}

.form-label {
  width: 100px;
  font-weight: 500;
  color: var(--el-text-color-regular);
}

.form-content {
  display: flex;
  gap: 16px;
  flex: 1;
}

.topic {
  :deep(.el-form-item__content) {
    row-gap: 10px;
    width: 100%;
  }
}
</style>
