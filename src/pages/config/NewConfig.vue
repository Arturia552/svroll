<template>
  <el-tabs v-model="activeName" class="basic-tabs">
    <el-tab-pane label="基础配置" name="basic" />
    <el-tab-pane label="客户端配置" name="client" />
    <el-tab-pane label="数据配置" name="data" />
  </el-tabs>

  <data-model
    v-show="activeName === 'data'"
  />
  <client-config
    v-show="activeName === 'client'"
  />
  <el-form
    v-show="activeName === 'basic'"
    ref="newConfigFormRef"
    :rules="rules"
    :model="newConfigForm"
    label-position="top"
    :inline="false"
    size="default"
  >
    <el-form-item
      label="broker地址"
      prop="broker"
      :rules="[
        { required: true, message: '请输入broker地址', trigger: 'blur' },
      ]"
    >
      <el-input v-model="newConfigForm.broker">
        <template #append>
          <el-button @click="validateUrl">
            测试连通
          </el-button>
        </template>
      </el-input>
    </el-form-item>
    <el-form-item class="topic" label="消息主题" prop="topicConfig.data">
      <template v-for="(key, value) in newConfigForm.topicConfig.data" :key="value">
        <div style="display: flex">
          <div class="form-label">
            {{ value }}
          </div>
          <div class="form-content">
            <template v-for="(k, v) in key" :key="k">
              <el-input
                v-model="newConfigForm.topicConfig.data[value][v]"
                :placeholder="v"
              />
            </template>
          </div>
        </div>
      </template>
    </el-form-item>
    <el-form-item label="协议类型" prop="protocol">
      <el-radio-group v-model="newConfigForm.protocol">
        <el-radio-button label="MQTT" value="Mqtt" />
        <el-radio-button label="TCP" value="Tcp" />
      </el-radio-group>
    </el-form-item>
    <el-form-item label="注册包机制" prop="enableRegister">
      <el-radio-group v-model="newConfigForm.enableRegister">
        <el-radio-button label="关闭" :value="false" />
        <el-radio-button label="启动" :value="true" />
      </el-radio-group>
    </el-form-item>
    <el-form-item
      v-if="newConfigForm.enableRegister"
      class="topic"
      label="主题配置"
      prop="topicConfig.register"
    >
      <template v-for="(key, value) in newConfigForm.topicConfig.register" :key="value">
        <div style="display: flex">
          <div class="form-label">
            {{ value }}
          </div>
          <div class="form-content">
            <template v-for="(k, v) in key" :key="k">
              <el-input
                v-model="newConfigForm.topicConfig.register[value][v]"
                :placeholder="v"
              />
            </template>
          </div>
        </div>
      </template>
    </el-form-item>
    <el-form-item label="随机生成" prop="enableRandom">
      <el-radio-group v-model="newConfigForm.enableRandom">
        <el-radio-button label="关闭" :value="false" />
        <el-radio-button label="启动" :value="true" />
      </el-radio-group>
    </el-form-item>
    <el-form-item label="线程数" prop="threadSize">
      <el-input v-model.number="newConfigForm.threadSize" />
    </el-form-item>
    <el-form-item label="每秒连接请求数" prop="maxConnectPerSecond">
      <el-input v-model.number="newConfigForm.maxConnectPerSecond" />
    </el-form-item>
    <el-form-item label="数据发送间隔" prop="sendInterval">
      <el-input v-model.number="newConfigForm.sendInterval" />
    </el-form-item>
    <el-form-item>
      <el-button type="primary" @click="onSubmit">
        确定
      </el-button>
      <el-button @click="cancel">
        取消
      </el-button>
    </el-form-item>
  </el-form>
</template>
<script setup name="NewConfig" lang="ts">
import { MqttConfig } from "@/types/mqttConfig";
import {
  ElMessage,
  type FormInstance,
} from "element-plus";
import { invoke } from "@tauri-apps/api/tauri";
import DataModel from "./DataModel.vue";
import ClientConfig from "./ClientConfig.vue";
import { getNestedValue, isJsonValueNull } from "@/hooks/processJsonStruct";

const emit = defineEmits<{
  (e: "close"): void;
}>();
const validConfig = defineModel<boolean>("valid");

const newConfigFormRef = ref<FormInstance>();
const newConfigForm = defineModel<MqttConfig>("configForm");
const activeName = ref<string>("basic");

const validateTopic = (rule: any, value: any, callback: any) => {
  if (
    isJsonValueNull(getNestedValue(newConfigForm.value, rule.field), [
      "keyIndex",
    ])
  ) {
    callback(new Error("请完善主题"));
  }
  callback();
};

const rules: any = ref({
  broker: [{ required: true, message: "请输入broker地址", trigger: "blur" }],
  "topicConfig.data": [{ validator: validateTopic, trigger: "blur" }],
  "topicConfig.register": [{ validator: validateTopic, trigger: "blur" }],
});

const validateUrl = async () => {
  try {
    const msg = await invoke("validate_mqtt_url", {
      url: newConfigForm.value.broker,
    });
    ElMessage.success(msg);
  } catch (error) {
    ElMessage.error(error);
  }
};

const onSubmit = async () => {
  newConfigFormRef.value?.validate((valid) => {
    if (valid) {
      validConfig.value = true;
      emit("close");
    }
  });
};

const cancel = () => {
  emit("close");
};
</script>
<style scoped lang="scss">
.form-label {
  display: inline-block;
  width: 100px;
  text-align: left;
}

.form-content {
  display: flex;
  gap: 10px;
}

.topic {
  :deep(.el-form-item__content) {
    row-gap: 10px;
    width: 500px;
  }
}
</style>
