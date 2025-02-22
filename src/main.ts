import { createApp } from "vue";

import ElementPlus from "element-plus";
import locale from "element-plus/dist/locale/zh-cn.mjs";
import * as ElementPlusIconsVue from "@element-plus/icons-vue";
import "./assets/styles/index.scss";

import App from "./App.vue";
import store from "./store";
import router from "./router";

let app = createApp(App);
app.use(store);
app.use(router);
app.use(ElementPlus, {
  locale: locale,
});
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component);
}
app.mount("#app");
