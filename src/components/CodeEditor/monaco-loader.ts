import type { editor, languages, MarkerSeverity, IDisposable } from "monaco-editor"

// Monaco Editor的动态加载器
class MonacoLoader {
  private static instance: MonacoLoader
  private monacoPromise: Promise<typeof import("monaco-editor")> | null = null
  private isLoaded = false

  static getInstance(): MonacoLoader {
    if (!MonacoLoader.instance) {
      MonacoLoader.instance = new MonacoLoader()
    }
    return MonacoLoader.instance
  }

  async loadMonaco(): Promise<typeof import("monaco-editor")> {
    if (this.isLoaded && this.monacoPromise) {
      return this.monacoPromise
    }

    if (!this.monacoPromise) {
      this.monacoPromise = this.dynamicImportMonaco()
    }

    try {
      const monaco = await this.monacoPromise
      this.isLoaded = true
      return monaco
    } catch (error) {
      console.error("Failed to load Monaco Editor:", error)
      // 重置状态以允许重试
      this.monacoPromise = null
      this.isLoaded = false
      throw error
    }
  }

  private async dynamicImportMonaco(): Promise<typeof import("monaco-editor")> {
    // 动态导入Monaco Editor
    // vite-plugin-monaco-editor-esm 插件会处理环境配置
    const monaco = await import("monaco-editor")
    return monaco
  }

  // 注册自定义hex语言
  async registerHexLanguage(): Promise<void> {
    const monaco = await this.loadMonaco()

    // 注册hex语言
    monaco.languages.register({ id: "hex" })

    // 为hex配置输入规则
    monaco.languages.setLanguageConfiguration("hex", {
      wordPattern: /[0-9A-Fa-f]+/,
      brackets: [],
      autoClosingPairs: [],
      surroundingPairs: [],
    })

    // 设置hex语言的token化规则
    monaco.languages.setMonarchTokensProvider("hex", {
      tokenizer: {
        root: [
          [/[0-9A-Fa-f]/, "number.hex"],
          [/\s+/, "white"],
          [/./, "invalid"],
        ],
      },
    })
  }

  // 创建编辑器实例
  async createEditor(
    container: HTMLElement,
    options: editor.IStandaloneEditorConstructionOptions,
  ): Promise<editor.IStandaloneCodeEditor> {
    const monaco = await this.loadMonaco()

    // 检查容器是否有效
    if (!container) {
      throw new Error("Editor container is null or undefined")
    }

    // 确保容器在DOM中且有尺寸
    const rect = container.getBoundingClientRect()
    if (rect.width === 0 || rect.height === 0) {
      // 给容器设置最小尺寸以防止错误
      container.style.minHeight = "200px"
      container.style.minWidth = "100%"
    }

    try {
      return monaco.editor.create(container, options)
    } catch (error) {
      throw new Error(`Failed to create Monaco Editor: ${error}`)
    }
  }

  // 验证JSON格式
  validateJson(jsonStr: string): {
    isValid: boolean
    error?: string
    position?: { line: number; column: number }
  } {
    if (!jsonStr.trim()) return { isValid: true }

    try {
      const parsed = JSON.parse(jsonStr)

      // 必须是对象类型
      if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
        return {
          isValid: false,
          error: "JSON数据必须是对象格式",
          position: { line: 1, column: 1 },
        }
      }

      return { isValid: true }
    } catch (error) {
      const errorMsg = (error as Error).message
      let line = 1
      let column = 1

      // 尝试从错误信息中提取位置信息
      const positionMatch = errorMsg.match(/at position (\d+)|line (\d+) column (\d+)/i)
      if (positionMatch) {
        if (positionMatch[1]) {
          const position = parseInt(positionMatch[1])
          const lines = jsonStr.substring(0, position).split("\n")
          line = lines.length
          column = lines[lines.length - 1].length + 1
        } else if (positionMatch[2] && positionMatch[3]) {
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

  // 设置错误标记
  async setErrorMarkers(
    editor: editor.IStandaloneCodeEditor,
    validationResult: {
      isValid: boolean
      error?: string
      position?: { line: number; column: number }
    },
  ): Promise<void> {
    const monaco = await this.loadMonaco()

    if (!validationResult.isValid && validationResult.error) {
      const position = validationResult.position || { line: 1, column: 1 }
      monaco.editor.setModelMarkers(editor.getModel()!, "jsonValidator", [
        {
          startLineNumber: position.line,
          startColumn: position.column,
          endLineNumber: position.line,
          endColumn: position.column + 1,
          message: validationResult.error,
          severity: monaco.MarkerSeverity.Error,
        },
      ])
    } else {
      monaco.editor.setModelMarkers(editor.getModel()!, "jsonValidator", [])
    }
  }

  // 设置编辑器语言
  async setModelLanguage(editor: editor.IStandaloneCodeEditor, language: string): Promise<void> {
    const monaco = await this.loadMonaco()
    monaco.editor.setModelLanguage(editor.getModel()!, language)
  }

  // 格式化hex值
  formatHexValue(value: string): string {
    const cleaned = value.replace(/\s+/g, "")
    return cleaned.match(/.{1,2}/g)?.join(" ") || cleaned
  }

  // 验证hex值
  isValidHex(value: string): boolean {
    const cleaned = value.replace(/\s+/g, "")
    return /^[0-9A-Fa-f]*$/.test(cleaned)
  }
}

export default MonacoLoader
export type { editor, languages, MarkerSeverity, IDisposable }
