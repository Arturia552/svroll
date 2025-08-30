import { createApp } from "vue"

import "element-plus/theme-chalk/src/index.scss"
import * as ElementPlusIconsVue from "@element-plus/icons-vue"
import "./assets/styles/index.scss"
import "./assets/styles/common.scss"

import App from "./App.vue"
import store from "./store"
import router from "./router"

const app = createApp(App)
app.use(store)
app.use(router)
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.mount("#app")

if ((window as any).__TAURI__) {
  document.addEventListener("contextmenu", (e) => {
    if (import.meta.env.PROD) {
      e.preventDefault()
    }
  })

  document.addEventListener("selectstart", (e) => {
    const target = e.target as HTMLElement
    if (target?.closest("[data-tauri-drag-region]")) {
      e.preventDefault()
    }
  })

  console.log("Tauri app initialized")
}
