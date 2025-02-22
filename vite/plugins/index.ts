import vue from '@vitejs/plugin-vue'

import createAutoImport from './auto-import'
import createSvgIcon from './svg-icon'
import createSetupExtend from './setup-extend'
import createAutoComponents from './auto-components'
import svgLoader from 'vite-svg-loader'
import monacoEditorPlugin from 'vite-plugin-monaco-editor'
import { PluginOption } from 'vite'
import zh_CN from 'vscode-loc.git/i18n/vscode-language-pack-zh-hans/translations/main.i18n.json'

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
