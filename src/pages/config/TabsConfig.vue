<template>
  <div class="tabs-config">
    <el-tabs v-model="activeName" class="custom-tabs" type="border-card">
      <el-tab-pane label="基础配置" name="basic">
        <template #label>
          <div class="tab-label">
            <el-icon>
              <Setting />
            </el-icon>
            <span>基础配置</span>
            <el-tag v-if="formErrors.basic" type="danger" size="small" effect="dark" class="error-tag">
              !
            </el-tag>
          </div>
        </template>
        <basic-config ref="basicConfigRef" v-model:config-form="configForm" />
      </el-tab-pane>
      <el-tab-pane label="客户端配置" name="client">
        <template #label>
          <div class="tab-label">
            <el-icon>
              <User />
            </el-icon>
            <span>客户端配置</span>
            <el-tag v-if="formErrors.client" type="danger" size="small" effect="dark" class="error-tag">
              !
            </el-tag>
          </div>
        </template>
        <client-config ref="clientConfigRef" />
      </el-tab-pane>
      <el-tab-pane v-if="configForm.protocol === 'Mqtt'" label="数据配置" name="data">
        <template #label>
          <div class="tab-label">
            <el-icon>
              <Document />
            </el-icon>
            <span>数据配置</span>
            <el-tag v-if="formErrors.data" type="danger" size="small" effect="dark" class="error-tag">
              !
            </el-tag>
          </div>
        </template>
        <data-model ref="dataModelRef" />
      </el-tab-pane>
    </el-tabs>

    <div class="action-buttons">
      <el-button type="primary" :loading="submitting" @click="onSubmit">
        确定
      </el-button>
      <el-button @click="$emit('close')">
        取消
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import DataModel from './DataModel.vue';
import ClientConfig from './ClientConfig.vue';
import BasicConfig from './BasicConfig.vue';
import { ConnectConfig } from '@/types/mqttConfig';
import { ElMessage } from 'element-plus';

const configForm = defineModel<ConnectConfig>('configForm');
// 添加对valid状态的双向绑定
const valid = defineModel<boolean>('valid', { default: false });

const emit = defineEmits<{
    (e: 'close'): void;
    (e: 'submit'): void;
}>();

const activeName = ref<string>("basic");
const basicConfigRef = ref();
const clientConfigRef = ref();
const dataModelRef = ref();
const submitting = ref(false);

// 跟踪表单错误状态
const formErrors = reactive({
    basic: false,
    client: false,
    data: false
});

// 清除所有错误标记
const clearErrors = () => {
    formErrors.basic = false;
    formErrors.client = false;
    formErrors.data = false;
};

// 检查客户端配置
const validateClientConfig = (): boolean => {
    // 如果有客户端，则检查是否至少有一个客户端
    if (configForm.value.clients && configForm.value.clients.length === 0) {
        formErrors.client = true;
        ElMessage.warning('请添加至少一个客户端');
        return false;
    }
    return true;
};

// 检查数据配置
const validateDataConfig = (): boolean => {
    if(configForm.value.protocol !== 'Mqtt') {
        return true;
    }

    // 检查是否有字段结构
    if (!configForm.value.fieldStruct || configForm.value.fieldStruct.length === 0) {
        formErrors.data = true;
        ElMessage.warning('请添加至少一个数据字段');
        return false;
    }
    return true;
};

// 表单提交
const onSubmit = async () => {
    submitting.value = true;
    clearErrors();

    try {
        // 首先验证基础配置
        const basicValid = await basicConfigRef.value?.validateForm();

        if (!basicValid) {
            formErrors.basic = true;
            activeName.value = 'basic';
            return;
        }

        // 然后验证客户端配置
        if (!validateClientConfig()) {
            activeName.value = 'client';
            return;
        }

        // 最后验证数据配置
        if (!validateDataConfig()) {
            activeName.value = 'data';
            return;
        }

        // 所有验证通过，设置valid为true
        nextTick(() => {
            valid.value = true;
        });
        // 提交表单
        emit('submit');
        emit('close');

    } catch (error) {
        console.error('Form validation error:', error);
        ElMessage.error('表单验证出错，请检查各项配置');
    } finally {
        submitting.value = false;
    }
};
</script>

<style scoped lang="scss">
.tabs-config {
    width: 100%;
    display: flex;
    flex-direction: column;
    min-height: calc(100vh - 150px);
    position: relative;
    padding-bottom: 80px; /* 为固定的按钮留出空间 */
}

.custom-tabs {
    flex: 1;

    :deep(.el-tabs__header) {
        margin-bottom: 0;
    }

    :deep(.el-tabs__item) {
        padding: 0 24px;
        height: 48px;
        line-height: 48px;
        font-size: 16px;
        transition: all 0.3s;

        &.is-active {
            font-weight: 600;
        }
    }

    :deep(.el-tabs__content) {
        background: var(--el-bg-color);
        border-radius: 0 var(--el-border-radius-base) var(--el-border-radius-base) var(--el-border-radius-base);
        box-shadow: var(--el-box-shadow-light);
        padding: 24px;
    }
}

.tab-label {
    display: flex;
    align-items: center;
    gap: 8px;
    position: relative;

    .el-icon {
        margin-right: 4px;
    }

    .error-tag {
        font-weight: bold;
        margin-left: 4px;
        border-radius: 50%;
        width: 18px;
        height: 18px;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
    }
}

.action-buttons {
    display: flex;
    justify-content: center;
    gap: 16px;
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background-color: var(--el-bg-color);
    padding: 16px 0;
    box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
    z-index: 10;

    .el-button {
        min-width: 120px;
        padding: 12px 24px;
        font-size: 16px;
    }
}
</style>
