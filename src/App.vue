<template>
  <div class="app-container">
    <el-config-provider :message="config" :locale="zhCn">
      <title-bar title="svroll" />
      <div class="app-content">
        <router-view />
      </div>
    </el-config-provider>
  </div>
</template>

<script setup lang="ts">
import TitleBar from "@/components/TitleBar/index.vue"
import { invoke } from '@tauri-apps/api/core'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'


document.addEventListener("DOMContentLoaded", async () => {
                await invoke("close_splashscreen")
            })
const config = reactive({
    placement: "bottom",
})
</script>

<style lang="scss">
.app-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.app-content {
    flex: 1;
    overflow: auto;
}
</style>
