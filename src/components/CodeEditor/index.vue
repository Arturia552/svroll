<template>
  <div ref="editor" class="edit-container" />
</template>
<script setup name="CodeEditor" lang="ts">
import * as monaco from "monaco-editor";
const props = defineProps({
  language: {
    type: String,
    default: "json",
  },
});
const modelValue = defineModel<string>("jsonEdit");
const editor = ref(null);
onMounted(() => {
  const editorInstance = monaco.editor.create(editor.value, {
    language: props.language,
    minimap: {
      enabled: false,
    },
    tabSize: 2,
    colorDecorators: true,
    readOnly: false,
    theme: "vs-dark",
  });
  editorInstance.onDidChangeModelContent(() => {
    modelValue.value = editorInstance.getValue();
  });
  watch(modelValue, (newValue) => {
    if (editorInstance.getValue() !== newValue) {
      editorInstance.setValue(newValue);
      editorInstance.getAction("editor.action.formatDocument").run();
    }
  });
});
</script>
<style lang="scss" scoped>
.edit-container {
  height: 100%;
}
</style>
