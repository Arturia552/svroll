<template>
  <router-view />
</template>
<script setup lang="ts">

import { useDark } from '@vueuse/core'
useDark({
  selector: 'html',
  attribute: 'class',
  valueDark: 'dark',
  valueLight: 'light-theme',
  storageKey: 'theme-preference',
})



</script>
<style></style>
