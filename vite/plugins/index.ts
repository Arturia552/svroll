import vue from '@vitejs/plugin-vue'
import eslint from 'vite-plugin-eslint'

import createAutoImport from './auto-import'
import createSvgIcon from './svg-icon'
import createSetupExtend from './setup-extend'
import createAutoComponents from './auto-components'
import svgLoader from 'vite-svg-loader'
import monacoEditorPlugin from 'vite-plugin-monaco-editor-esm'
import { PluginOption } from 'vite'

export default function createVitePlugins(isBuild = false): PluginOption[] {
  const vitePlugins = [vue()]
  
  // 添加 ESLint 插件
  if (!isBuild) {
    vitePlugins.push(eslint({
      include: ['src/**/*.{js,jsx,ts,tsx,vue}'],
      exclude: ['node_modules', 'dist'],
      cache: false,
      fix: true
    }))
  }
  
  vitePlugins.push(createAutoImport())
  vitePlugins.push(createSetupExtend())
  vitePlugins.push(createAutoComponents())
  vitePlugins.push(svgLoader())
  vitePlugins.push(monacoEditorPlugin({}))
  vitePlugins.push(createSvgIcon(isBuild))
  return vitePlugins
}
