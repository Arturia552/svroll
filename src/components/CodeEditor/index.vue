<template>
  <div ref="editorRef" class="edit-container" />
</template>
<script setup name="CodeEditor" lang="ts">
import MonacoLoader from "./monaco-loader"
import type { editor, IDisposable } from "./monaco-loader"

const props = defineProps({
  language: {
    type: String,
    default: "json",
  },
})

const modelValue = defineModel<string>("jsonEdit")
const editorRef = ref<HTMLElement | null>(null)
let editorInstance: editor.IStandaloneCodeEditor | null = null
let keyDownHandler: IDisposable | null = null

const monacoLoader = MonacoLoader.getInstance()

// 设置按键处理器
const setupKeyDownHandler = () => {
  // 清除之前的事件处理器
  if (keyDownHandler) {
    keyDownHandler.dispose()
    keyDownHandler = null
  }

  // 只在hex模式下添加按键限制
  if (props.language === "hex" && editorInstance) {
    keyDownHandler = editorInstance.onKeyDown((e) => {
      const key = e.browserEvent.key
      // 允许的按键：0-9, a-f, A-F, 以及控制键(退格、删除、箭头等)
      const isHexChar = /^[0-9A-Fa-f]$/.test(key)
      const isControlKey =
        e.browserEvent.ctrlKey ||
        e.browserEvent.metaKey ||
        [
          "Backspace",
          "Delete",
          "ArrowLeft",
          "ArrowRight",
          "ArrowUp",
          "ArrowDown",
          "Tab",
          "Home",
          "End",
          "Enter",
          " ",
        ].includes(key)

      if (!isHexChar && !isControlKey) {
        e.preventDefault()
        e.stopPropagation()
        return false
      }
      return true
    })
  }
}

// 格式化hex值
const formatHex = () => {
  if (!editorInstance || props.language !== "hex") return

  const currentValue = editorInstance.getValue()
  if (currentValue) {
    const formatted = monacoLoader.formatHexValue(currentValue)
    editorInstance.setValue(formatted)
    modelValue.value = formatted
  }
}

// 处理编辑器内容变化
const handleContentChange = async () => {
  if (!editorInstance) return

  const value = editorInstance.getValue()

  if (props.language === "hex") {
    // 如果是hex模式，验证输入的有效性
    if (!monacoLoader.isValidHex(value)) {
      // 如果包含非法字符，替换为上一个有效值或清空
      const cleaned = value.replace(/[^0-9A-Fa-f\s]/g, "")
      editorInstance.setValue(cleaned)
      modelValue.value = cleaned
      return
    }
  } else if (props.language === "json") {
    // 如果是JSON模式，验证JSON格式
    const validation = monacoLoader.validateJson(value)
    await monacoLoader.setErrorMarkers(editorInstance, validation)
  }

  modelValue.value = value
}

// 初始化编辑器
const initializeEditor = async () => {
  if (!editorRef.value) {
    console.warn("Editor container not ready")
    return
  }

  // 如果已经存在实例，先清理
  if (editorInstance) {
    editorInstance.dispose()
    editorInstance = null
  }

  try {
    // 注册hex语言
    await monacoLoader.registerHexLanguage()

    // 确保容器有尺寸
    await nextTick()

    // 检查容器尺寸
    const rect = editorRef.value.getBoundingClientRect()
    if (rect.width === 0 || rect.height === 0) {
      console.warn("Editor container has zero dimensions, waiting for layout...")
      // 等待布局完成
      await new Promise((resolve) => setTimeout(resolve, 50))
    }

    // 创建编辑器实例
    editorInstance = await monacoLoader.createEditor(editorRef.value, {
      language: props.language,
      minimap: {
        enabled: false,
      },
      tabSize: 2,
      colorDecorators: true,
      readOnly: false,
      theme: "vs-dark",
      automaticLayout: true,
      scrollBeyondLastLine: false,
    })

    // 设置按键处理器
    setupKeyDownHandler()

    // 监听内容变化
    editorInstance.onDidChangeModelContent(handleContentChange)

    // 初始化值
    if (modelValue.value) {
      editorInstance.setValue(modelValue.value)

      if (props.language === "json") {
        editorInstance.getAction("editor.action.formatDocument")?.run()
        // 初始化时也进行JSON格式校验
        const validation = monacoLoader.validateJson(modelValue.value)
        await monacoLoader.setErrorMarkers(editorInstance, validation)
      } else if (props.language === "hex") {
        const formatted = monacoLoader.formatHexValue(modelValue.value)
        if (formatted !== modelValue.value) {
          editorInstance.setValue(formatted)
          modelValue.value = formatted
        }
      }
    }
  } catch (error) {
    console.error("Failed to initialize Monaco Editor:", error)
  }
}

// 监听modelValue变化
watch(modelValue, async (newValue) => {
  if (!editorInstance) return

  if (editorInstance.getValue() !== newValue) {
    editorInstance.setValue(newValue || "")

    if (props.language === "json" && newValue) {
      // JSON模式下进行格式化
      editorInstance.getAction("editor.action.formatDocument")?.run()
    } else if (props.language === "hex" && newValue) {
      // Hex模式下的自定义格式化
      const formatted = monacoLoader.formatHexValue(newValue)
      if (formatted !== newValue) {
        editorInstance.setValue(formatted)
        modelValue.value = formatted
      }
    }
  }
})

// 监听language变化
watch(
  () => props.language,
  async (newLanguage) => {
    if (!editorInstance) return

    // 更新编辑器语言
    await monacoLoader.setModelLanguage(editorInstance, newLanguage)

    // 重新设置按键处理器
    setupKeyDownHandler()

    // 根据新模式处理内容
    const currentValue = editorInstance.getValue()
    if (newLanguage === "hex" && currentValue) {
      // 确保值只包含有效的hex字符
      const cleaned = currentValue.replace(/[^0-9A-Fa-f\s]/g, "")
      const formatted = monacoLoader.formatHexValue(cleaned)
      editorInstance.setValue(formatted)
      modelValue.value = formatted
    } else if (newLanguage === "json" && currentValue) {
      editorInstance.getAction("editor.action.formatDocument")?.run()
    }
  },
)

// 暴露方法
defineExpose({
  formatHex,
})

// 生命周期
onMounted(async () => {
  // 确保DOM完全渲染
  await nextTick()

  // 添加小延迟确保容器有正确的尺寸
  setTimeout(initializeEditor, 100)
})

onBeforeUnmount(() => {
  // 清理按键处理器
  if (keyDownHandler) {
    keyDownHandler.dispose()
    keyDownHandler = null
  }

  // 清理编辑器实例
  if (editorInstance) {
    editorInstance.dispose()
    editorInstance = null
  }
})
</script>
<style lang="scss" scoped>
.edit-container {
  height: 100%;
}
</style>
