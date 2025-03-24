<template>
  <div ref="editor" class="edit-container" />
</template>
<script setup name="CodeEditor" lang="ts">
import * as monaco from "monaco-editor";
const props = defineProps({
  language: {
    type: String,
    default: "json",
  }
});
const modelValue = defineModel<string>("jsonEdit");
const editor = ref(null);
let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null;

const isValidHex = (value: string): boolean => {
  const cleaned = value.replace(/\s+/g, '');
  return /^[0-9A-Fa-f]*$/.test(cleaned);
};

const formatHexValue = (value: string): string => {
  const cleaned = value.replace(/\s+/g, '');
  return cleaned.match(/.{1,2}/g)?.join(' ') || cleaned;
};

const formatHex = () => {
  if (!editorInstance || props.language !== 'hex') return;
  
  const currentValue = editorInstance.getValue();
  if (currentValue) {
    const formatted = formatHexValue(currentValue);
    editorInstance.setValue(formatted);
    modelValue.value = formatted;
  }
};

// 配置hex语言
const configureHexLanguage = () => {
  // 注册hex语言
  monaco.languages.register({ id: 'hex' });
  
  // 为hex配置输入规则
  monaco.languages.setLanguageConfiguration('hex', {
    wordPattern: /[0-9A-Fa-f]+/
  });
}

defineExpose({
  formatHex
});

onMounted(() => {
  // 配置hex语言
  configureHexLanguage();
  
  editorInstance = monaco.editor.create(editor.value, {
    language: props.language,
    minimap: {
      enabled: false,
    },
    tabSize: 2,
    colorDecorators: true,
    readOnly: false,
    theme: "vs-dark",
    automaticLayout: true, // 启用自动布局调整
    scrollBeyondLastLine: false, // 防止内容较少时出现过多空白
  });
  
  // 添加按键事件处理
  if (props.language === 'hex') {
    editorInstance.onKeyDown((e) => {
      // 如果当前模式是hex，验证按键是否为有效的hex字符
      if (props.language === 'hex') {
        const key = e.browserEvent.key;
        // 允许的按键：0-9, a-f, A-F, 以及控制键(退格、删除、箭头等)
        const isHexChar = /^[0-9A-Fa-f]$/.test(key);
        const isControlKey = e.browserEvent.ctrlKey || e.browserEvent.metaKey || 
                            ['Backspace', 'Delete', 'ArrowLeft', 'ArrowRight', 
                             'ArrowUp', 'ArrowDown', 'Tab', 'Home', 'End', 
                             'Enter', ' '].includes(key);
                             
        if (!isHexChar && !isControlKey) {
          e.preventDefault();
          e.stopPropagation();
          return false;
        }
      }
      return true;
    });
  }
  
  editorInstance.onDidChangeModelContent(() => {
    const value = editorInstance!.getValue();
    
    if (props.language === 'hex') {
      // 如果是hex模式，验证输入的有效性
      if (!isValidHex(value)) {
        // 如果包含非法字符，替换为上一个有效值或清空
        const cleaned = value.replace(/[^0-9A-Fa-f\s]/g, '');
        editorInstance!.setValue(cleaned);
        modelValue.value = cleaned;
        return;
      }
    }
    
    modelValue.value = value;
  });
  
  watch(modelValue, (newValue) => {
    if (!editorInstance) return;
    
    if (editorInstance.getValue() !== newValue) {
      editorInstance.setValue(newValue);
      
      if (props.language === 'json') {
        // JSON模式下进行格式化
        editorInstance.getAction("editor.action.formatDocument")?.run();
      } else if (props.language === 'hex' && newValue) {
        // Hex模式下的自定义格式化
        const formatted = formatHexValue(newValue);
        if (formatted !== newValue) {
          editorInstance.setValue(formatted);
          modelValue.value = formatted;
        }
      }
    }
  });
  
  // 初始化值
  if (modelValue.value) {
    editorInstance.setValue(modelValue.value);
    if (props.language === 'json') {
      editorInstance.getAction("editor.action.formatDocument")?.run();
    } else if (props.language === 'hex') {
      const formatted = formatHexValue(modelValue.value);
      if (formatted !== modelValue.value) {
        editorInstance.setValue(formatted);
        modelValue.value = formatted;
      }
    }
  }
});

// 监听language变化
watch(() => props.language, (newLanguage) => {
  if (!editorInstance) return;
  
  // 更新编辑器语言
  monaco.editor.setModelLanguage(
    editorInstance.getModel()!,
    newLanguage
  );
  
  // 如果切换到hex模式，添加按键监听
  if (newLanguage === 'hex') {
    editorInstance.onKeyDown((e) => {
      const key = e.browserEvent.key;
      const isHexChar = /^[0-9A-Fa-f]$/.test(key);
      const isControlKey = e.browserEvent.ctrlKey || e.browserEvent.metaKey || 
                          ['Backspace', 'Delete', 'ArrowLeft', 'ArrowRight', 
                           'ArrowUp', 'ArrowDown', 'Tab', 'Home', 'End', 
                           'Enter', ' '].includes(key);
                           
      if (!isHexChar && !isControlKey) {
        e.preventDefault();
        e.stopPropagation();
        return false;
      }
      return true;
    });
  }
  
  // 根据新模式处理内容
  const currentValue = editorInstance.getValue();
  if (newLanguage === 'hex' && currentValue) {
    // 确保值只包含有效的hex字符
    const cleaned = currentValue.replace(/[^0-9A-Fa-f\s]/g, '');
    const formatted = formatHexValue(cleaned);
    editorInstance.setValue(formatted);
    modelValue.value = formatted;
  } else if (newLanguage === 'json' && currentValue) {
    editorInstance.getAction("editor.action.formatDocument")?.run();
  }
});

onBeforeUnmount(() => {
  if (editorInstance) {
    editorInstance.dispose();
    editorInstance = null;
  }
});
</script>
<style lang="scss" scoped>
.edit-container {
  height: 100%;
}
</style>
