import vue from "@vitejs/plugin-vue"
import eslint from "vite-plugin-eslint"

import createAutoImport from "./auto-import"
import createSvgIcon from "./svg-icon"
import createSetupExtend from "./setup-extend"
import createAutoComponents from "./auto-components"
import svgLoader from "vite-svg-loader"
import monacoEditorPlugin from "vite-plugin-monaco-editor-esm"
import { PluginOption } from "vite"
import { visualizer } from "rollup-plugin-visualizer"

export default function createVitePlugins(isBuild = false): PluginOption[] {
  const vitePlugins = [vue()]

  // 添加 ESLint 插件
  if (!isBuild) {
    vitePlugins.push(
      eslint({
        include: ["src/**/*.{js,jsx,ts,tsx,vue}"],
        exclude: ["node_modules", "dist"],
        cache: false,
        fix: true,
      }),
    )
  }

  vitePlugins.push(createAutoImport())
  vitePlugins.push(createSetupExtend())
  vitePlugins.push(createAutoComponents())
  vitePlugins.push(svgLoader())

  // 简化Monaco Editor配置，只加载必需的功能
  vitePlugins.push(
    monacoEditorPlugin({
      // 只加载JSON语言worker，减少包体积
      languageWorkers: ["json"],
      // 启用全局API
      globalAPI: true,
    }),
  )

  vitePlugins.push(createSvgIcon(isBuild))

  if (isBuild) {
    vitePlugins.push(
      visualizer({
        filename: "dist/stats.html",
        open: false, // 构建时不自动打开
        gzipSize: true,
        brotliSize: true,
      }),
    )
  }

  return vitePlugins
}
