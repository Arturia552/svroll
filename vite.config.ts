import { ConfigEnv, UserConfig } from 'vite';
import path from 'path';
import createVitePlugins from './vite/plugins';
import zh_CN from 'vscode-loc.git/i18n/vscode-language-pack-zh-hans/translations/main.i18n.json'

export default ({ mode, command }: ConfigEnv): UserConfig => ({
  plugins: createVitePlugins(command === 'build'),
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
    extensions: ['.mjs', '.js', '.ts', '.jsx', '.tsx', '.json', '.vue'],
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  optimizeDeps: {
    esbuildOptions: {
      plugins: [
      
      ],
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@use "@/assets/styles/element/index.scss" as *;`,
        api: 'modern-compiler',
      },
    },
    postcss: {
      plugins: [
        {
          postcssPlugin: 'internal:charset-removal',
          AtRule: {
            charset: (atRule: any) => {
              if (atRule.name === 'charset') {
                atRule.remove();
              }
            },
          },
        },
      ],
    },
  },
});
