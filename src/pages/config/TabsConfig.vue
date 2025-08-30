<template>
  <div class="tabs-config">
    <div v-show="activeTab === 'basic'" class="config-panel">
      <basic-config ref="basicConfigRef" v-model:config-form="configForm" />
    </div>

    <div v-show="activeTab === 'client'" class="config-panel">
      <client-config ref="clientConfigRef" />
    </div>

    <div v-show="activeTab === 'data' && configForm.protocol === 'Mqtt'" class="config-panel">
      <data-model ref="dataModelRef" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import DataModel from './DataModel.vue'
import ClientConfig from './ClientConfig.vue'
import BasicConfig from './BasicConfig.vue'
import { ConnectConfig } from '@/types/mqttConfig'
import { ElMessage } from 'element-plus'

const configForm = defineModel<ConnectConfig>('configForm')
// 添加对activeTab状态的双向绑定
const activeTab = defineModel<'basic' | 'client' | 'data'>('activeTab', { default: 'basic' })

const basicConfigRef = ref()
const clientConfigRef = ref()
const dataModelRef = ref()

// 跟踪表单错误状态
const formErrors = reactive({
    basic: false,
    client: false,
    data: false
})

// 清除所有错误标记
const clearErrors = () => {
    formErrors.basic = false
    formErrors.client = false
    formErrors.data = false
}

// 检查客户端配置
const validateClientConfig = (): boolean => {
    // 如果有客户端，则检查是否至少有一个客户端
    if (configForm.value.clients && configForm.value.clients.length === 0) {
        formErrors.client = true
        ElMessage.warning('请添加至少一个客户端')
        return false
    }
    return true
}

// 检查数据配置
const validateDataConfig = (): boolean => {
    if(configForm.value.protocol !== 'Mqtt') {
        return true
    }

    // 检查是否有字段结构
    if (!configForm.value.fieldStruct || configForm.value.fieldStruct.length === 0) {
        formErrors.data = true
        ElMessage.warning('请添加至少一个数据字段')
        return false
    }
    return true
}

// 表单校验方法，供外部调用
const validateForm = async (): Promise<boolean> => {
    clearErrors()

    try {
        // 首先验证基础配置
        const basicValid = await basicConfigRef.value?.validateForm()

        if (!basicValid) {
            formErrors.basic = true
            activeTab.value = 'basic'
            return false
        }

        // 然后验证客户端配置
        if (!validateClientConfig()) {
            activeTab.value = 'client'
            return false
        }

        // 最后验证数据配置
        if (!validateDataConfig()) {
            activeTab.value = 'data'
            return false
        }

        return true

    } catch (error) {
        console.error('Form validation error:', error)
        ElMessage.error('表单验证出错，请检查各项配置')
        return false
    }
}

// 刷新数据模型
const refreshDataModel = () => {
    if (dataModelRef.value && configForm.value.protocol === 'Mqtt') {
        nextTick(() => {
            dataModelRef.value.refreshStructure()
        })
    }
}

// 暴露方法供父组件调用
defineExpose({
    refreshDataModel,
    formErrors,
    validateForm
})
</script>

<style scoped lang="scss">
.tabs-config {
    width: 100%;
    display: flex;
    flex-direction: column;
    position: relative;
    font-size: 13px; /* 整体缩小字号 */
}

.config-panel {
    width: 100%;
    flex: 1;
}
</style>
