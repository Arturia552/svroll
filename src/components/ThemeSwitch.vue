<template>
  <el-switch
    v-model="isDark"
    class="theme-switch"
    inline-prompt
    :active-icon="Moon"
    :inactive-icon="Sunny"
  />
</template>

<script setup lang="ts">
import { useDark } from '@vueuse/core';
import { Moon, Sunny } from '@element-plus/icons-vue';

const isDark = useDark({
  selector: 'html',
  attribute: 'class',
  valueDark: 'dark',
  valueLight: 'light-theme',
  storageKey: 'theme-preference',
});

</script>

<style scoped>
.theme-switch {
  position: fixed;
  right: 20px;
  top: 20px;
  z-index: 1000;
  transition: opacity 0.3s;
  
  &:hover {
    opacity: 0.9;
  }
}
</style>
