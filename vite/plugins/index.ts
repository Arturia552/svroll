import vue from '@vitejs/plugin-vue'

import createAutoImport from './auto-import'
import createSvgIcon from './svg-icon'
import createSetupExtend from './setup-extend'
import createAutoComponents from './auto-components'
import svgLoader from 'vite-svg-loader'
import monacoEditorPlugin from 'vite-plugin-monaco-editor-esm'
import { PluginOption } from 'vite'

export default function createVitePlugins(isBuild = false): PluginOption[] {
  const vitePlugins = [vue()]
  vitePlugins.push(createAutoImport())
  vitePlugins.push(createSetupExtend())
  vitePlugins.push(createAutoComponents())
  vitePlugins.push(svgLoader())
  vitePlugins.push(monacoEditorPlugin({}))
  vitePlugins.push(createSvgIcon(isBuild))
  return vitePlugins
}
