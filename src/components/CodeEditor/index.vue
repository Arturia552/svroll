<template>
  <div ref="editor" class="edit-container" />
</template>
<script setup name="CodeEditor" lang="ts">
import * as monaco from "monaco-editor"
const props = defineProps({
    language: {
        type: String,
        default: "json",
    },
})
const modelValue = defineModel<string>("jsonEdit")
const editor = ref(null)
let editorInstance: monaco.editor.IStandaloneCodeEditor | null = null
let keyDownHandler: monaco.IDisposable | null = null

const isValidHex = (value: string): boolean => {
    const cleaned = value.replace(/\s+/g, "")
    return /^[0-9A-Fa-f]*$/.test(cleaned)
}

const formatHexValue = (value: string): string => {
    const cleaned = value.replace(/\s+/g, "")
    return cleaned.match(/.{1,2}/g)?.join(" ") || cleaned
}

const formatHex = () => {
    if (!editorInstance || props.language !== "hex") return

    const currentValue = editorInstance.getValue()
    if (currentValue) {
        const formatted = formatHexValue(currentValue)
        editorInstance.setValue(formatted)
        modelValue.value = formatted
    }
}

// 验证JSON格式（支持嵌套结构）
const validateJson = (
    jsonStr: string,
): {
    isValid: boolean;
    error?: string;
    position?: { line: number; column: number };
} => {
    if (!jsonStr.trim()) return { isValid: true }

    try {
        const parsed = JSON.parse(jsonStr)

        // 必须是对象类型
        if (
            typeof parsed !== "object" ||
            parsed === null ||
            Array.isArray(parsed)
        ) {
            return {
                isValid: false,
                error: "JSON数据必须是对象格式",
                position: { line: 1, column: 1 },
            }
        }

        // 支持嵌套对象，只进行基本的JSON格式验证
        return { isValid: true }
    } catch (error) {
        const errorMsg = (error as Error).message
        let line = 1
        let column = 1

        // 尝试从错误信息中提取位置信息
        const positionMatch = errorMsg.match(
            /at position (\d+)|line (\d+) column (\d+)/i,
        )
        if (positionMatch) {
            if (positionMatch[1]) {
                // "at position X" 格式
                const position = parseInt(positionMatch[1])
                const lines = jsonStr.substring(0, position).split("\n")
                line = lines.length
                column = lines[lines.length - 1].length + 1
            } else if (positionMatch[2] && positionMatch[3]) {
                // "line X column Y" 格式
                line = parseInt(positionMatch[2])
                column = parseInt(positionMatch[3])
            }
        }

        return {
            isValid: false,
            error: "JSON格式错误: " + errorMsg,
            position: { line, column },
        }
    }
}

// 配置hex语言
const configureHexLanguage = () => {
    // 注册hex语言
    monaco.languages.register({ id: "hex" })

    // 为hex配置输入规则
    monaco.languages.setLanguageConfiguration("hex", {
        wordPattern: /[0-9A-Fa-f]+/,
    })
}

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

defineExpose({
    formatHex,
})

onMounted(() => {
    // 配置hex语言
    configureHexLanguage()

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
    })

    // 设置按键处理器
    setupKeyDownHandler()

    editorInstance.onDidChangeModelContent(() => {
        const value = editorInstance!.getValue()

        if (props.language === "hex") {
            // 如果是hex模式，验证输入的有效性
            if (!isValidHex(value)) {
                // 如果包含非法字符，替换为上一个有效值或清空
                const cleaned = value.replace(/[^0-9A-Fa-f\s]/g, "")
                editorInstance!.setValue(cleaned)
                modelValue.value = cleaned
                return
            }
        } else if (props.language === "json") {
            // 如果是JSON模式，验证JSON格式
            const validation = validateJson(value)
            if (!validation.isValid && validation.error) {
                // 显示错误标记，但不阻止输入
                const position = validation.position || { line: 1, column: 1 }
                monaco.editor.setModelMarkers(
                    editorInstance!.getModel()!,
                    "jsonValidator",
                    [
                        {
                            startLineNumber: position.line,
                            startColumn: position.column,
                            endLineNumber: position.line,
                            endColumn: position.column + 1,
                            message: validation.error,
                            severity: monaco.MarkerSeverity.Error,
                        },
                    ],
                )
            } else {
                // 清除错误标记
                monaco.editor.setModelMarkers(
                    editorInstance!.getModel()!,
                    "jsonValidator",
                    [],
                )
            }
        }

        modelValue.value = value
    })

    watch(modelValue, (newValue) => {
        if (!editorInstance) return

        if (editorInstance.getValue() !== newValue) {
            editorInstance.setValue(newValue)

            if (props.language === "json") {
                // JSON模式下进行格式化
                editorInstance.getAction("editor.action.formatDocument")?.run()
            } else if (props.language === "hex" && newValue) {
                // Hex模式下的自定义格式化
                const formatted = formatHexValue(newValue)
                if (formatted !== newValue) {
                    editorInstance.setValue(formatted)
                    modelValue.value = formatted
                }
            }
        }
    })

    // 初始化值
    if (modelValue.value) {
        editorInstance.setValue(modelValue.value)
        if (props.language === "json") {
            editorInstance.getAction("editor.action.formatDocument")?.run()
            // 初始化时也进行JSON格式校验
            const validation = validateJson(modelValue.value)
            if (!validation.isValid && validation.error) {
                const position = validation.position || { line: 1, column: 1 }
                monaco.editor.setModelMarkers(
                    editorInstance.getModel()!,
                    "jsonValidator",
                    [
                        {
                            startLineNumber: position.line,
                            startColumn: position.column,
                            endLineNumber: position.line,
                            endColumn: position.column + 1,
                            message: validation.error,
                            severity: monaco.MarkerSeverity.Error,
                        },
                    ],
                )
            }
        } else if (props.language === "hex") {
            const formatted = formatHexValue(modelValue.value)
            if (formatted !== modelValue.value) {
                editorInstance.setValue(formatted)
                modelValue.value = formatted
            }
        }
    }
})

// 监听language变化
watch(
    () => props.language,
    (newLanguage) => {
        if (!editorInstance) return

        // 更新编辑器语言
        monaco.editor.setModelLanguage(editorInstance.getModel()!, newLanguage)

        // 重新设置按键处理器
        setupKeyDownHandler()

        // 根据新模式处理内容
        const currentValue = editorInstance.getValue()
        if (newLanguage === "hex" && currentValue) {
            // 确保值只包含有效的hex字符
            const cleaned = currentValue.replace(/[^0-9A-Fa-f\s]/g, "")
            const formatted = formatHexValue(cleaned)
            editorInstance.setValue(formatted)
            modelValue.value = formatted
        } else if (newLanguage === "json" && currentValue) {
            editorInstance.getAction("editor.action.formatDocument")?.run()
        }
    },
)

onBeforeUnmount(() => {
    // 清理按键处理器
    if (keyDownHandler) {
        keyDownHandler.dispose()
        keyDownHandler = null
    }

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
