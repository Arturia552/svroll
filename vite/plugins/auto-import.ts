import autoImport from "unplugin-auto-import/vite";
import { ElementPlusResolver } from "unplugin-vue-components/resolvers";

export default function createAutoImport() {
  return autoImport({
    resolvers: [ElementPlusResolver({ importStyle: "sass" })],
    imports: ["vue", "vue-router", "pinia"],
    dts: true,
    eslintrc: {
      enabled: true,
    },
  });
}
