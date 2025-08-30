<template>
  <div class="app-container">
    <title-bar title="svroll" />
    <div class="app-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import TitleBar from "@/components/TitleBar/index.vue"
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
