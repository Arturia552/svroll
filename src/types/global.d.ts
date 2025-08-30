declare module "*.vue" {
  import { DefineComponent } from "vue"

  const component: DefineComponent<{}, {}, any>
  export default component
}

declare type ClassName = { [className: string]: any } | ClassName[] | string;

declare module "*.svg" {
  const CONTENT: string
  export default CONTENT
}

declare type Recordable<T = any> = Record<string, T>;

// Tauri-specific globals
declare global {
  interface Window {
    __TAURI__?: any;
  }
}

// Tauri theme types
declare type ThemeMode = "light" | "dark";

declare interface TauriWindowState {
  isMaximized: boolean;
  isMinimized: boolean;
  isFullscreen: boolean;
}
