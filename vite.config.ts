import { ConfigEnv, UserConfig } from "vite"
import path from "path"
import createVitePlugins from "./vite/plugins"

export default ({ command }: ConfigEnv): UserConfig => ({
  plugins: [...createVitePlugins(command === "build")],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
    extensions: [".mjs", ".js", ".ts", ".jsx", ".tsx", ".json", ".vue"],
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  optimizeDeps: {
    esbuildOptions: {
      plugins: [],
    },
    include: ["monaco-editor"],
    exclude: [
      "monaco-editor/esm/vs/language/typescript/ts.worker",
      "monaco-editor/esm/vs/language/html/html.worker",
      "monaco-editor/esm/vs/language/css/css.worker",
    ],
  },
  build: {
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          // Monaco Editor单独打包
          if (id.includes("monaco-editor")) {
            return "monaco"
          }
          // Vue相关库
          if (id.includes("vue") || id.includes("@vue")) {
            return "vue"
          }
          // Element Plus相关库
          if (id.includes("element-plus")) {
            return "element-plus"
          }
          // 其他第三方库
          if (id.includes("node_modules")) {
            return "vendor"
          }
        },
        chunkFileNames: (chunkInfo) => {
          return `js/${chunkInfo.name}-[hash].js`
        },
      },
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@use "@/assets/styles/element/index.scss" as *;`,
        api: "modern-compiler",
      },
    },
    postcss: {
      plugins: [
        {
          postcssPlugin: "internal:charset-removal",
          AtRule: {
            charset: (atRule: any) => {
              if (atRule.name === "charset") {
                atRule.remove()
              }
            },
          },
        },
      ],
    },
  },
})
