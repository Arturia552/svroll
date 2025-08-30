<template>
  <div class="custom-titlebar" data-tauri-drag-region>
    <div class="titlebar-content">
      <div class="titlebar-left">
        <!-- <div class="app-icon">
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="currentColor">
            <path d="M12 2L2 7L12 12L22 7L12 2Z" opacity="0.6" />
            <path d="M2 17L12 22L22 17" />
            <path d="M2 12L12 17L22 12" />
          </svg>
        </div> -->
        <span class="app-title">{{ props.title }}</span>
      </div>

      <div class="titlebar-center">
        <slot name="center" />
      </div>

      <div class="titlebar-right">
        <slot name="actions" />
        <theme-switch />
        <div class="window-controls">
          <button
            class="control-button minimize-btn"
            :aria-label="'Minimize'"
            @click="minimizeWindow">
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect
                x="2"
                y="5"
                width="8"
                height="2"
                fill="currentColor" />
            </svg>
          </button>

          <button
            class="control-button maximize-btn"
            :aria-label="isMaximized ? 'Restore' : 'Maximize'"
            @click="maximizeWindow">
            <svg
              v-if="!isMaximized"
              width="12"
              height="12"
              viewBox="0 0 12 12">
              <rect
                x="2"
                y="2"
                width="8"
                height="8"
                stroke="currentColor"
                stroke-width="1"
                fill="none" />
            </svg>
            <svg v-else
                 width="12"
                 height="12"
                 viewBox="0 0 12 12">
              <rect
                x="2"
                y="3"
                width="6"
                height="6"
                stroke="currentColor"
                stroke-width="1"
                fill="none" />
              <rect
                x="4"
                y="1"
                width="6"
                height="6"
                stroke="currentColor"
                stroke-width="1"
                fill="none" />
            </svg>
          </button>

          <button
            class="control-button close-btn"
            :aria-label="'Close'"
            @click="closeWindow">
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path
                d="M2 2L10 10M10 2L2 10"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ThemeSwitch from "@/components/ThemeSwitch/index.vue"

interface Props {
    title?: string;
}

const props = withDefaults(defineProps<Props>(), {
    title: "svroll",
})

const isMaximized = ref(false)

const minimizeWindow = async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window")
    await getCurrentWindow().minimize()
}

const maximizeWindow = async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window")
    const window = getCurrentWindow()

    if (isMaximized.value) {
        await window.unmaximize()
    } else {
        await window.maximize()
    }
}

const closeWindow = async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window")
    await getCurrentWindow().close()
}

const setupWindowListeners = async () => {
    try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window")
        const window = getCurrentWindow()

        isMaximized.value = await window.isMaximized()

        const unlistenResize = await window.onResized(() => {
            updateMaximizedState()
        })

        onUnmounted(() => {
            unlistenResize()
        })
    } catch (error) {
        console.warn("Failed to setup window listeners:", error)
    }
}

const updateMaximizedState = async () => {
    try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window")
        isMaximized.value = await getCurrentWindow().isMaximized()
    } catch (error) {
        console.warn("Failed to update maximized state:", error)
    }
}

onMounted(() => {
    setupWindowListeners()
})
</script>

<style scoped lang="scss">
.custom-titlebar {
    height: 40px;
    background: var(--el-bg-color);
    border-bottom: 1px solid var(--el-border-color);
    display: flex;
    align-items: center;
    user-select: none;
    position: relative;
    z-index: 9999;

    &[data-tauri-drag-region] {
        -webkit-app-region: drag;
    }
}

.titlebar-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    height: 100%;
    padding: 0 8px;
}

.titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex-shrink: 1;
}

.app-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    color: var(--el-color-primary);
    flex-shrink: 0;
}

.app-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--el-text-color-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.titlebar-center {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    min-width: 0;

    &:empty {
        -webkit-app-region: drag;
    }
}

.titlebar-right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
}

.window-controls {
    display: flex;
    align-items: center;
    -webkit-app-region: no-drag;
}

.control-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--el-text-color-regular);
    cursor: pointer;
    transition: all 0.2s ease;
    border-radius: 4px;

    &:hover {
        background: var(--el-fill-color-light);
        color: var(--el-text-color-primary);
    }

    &:active {
        background: var(--el-fill-color);
    }
}

.close-btn {
    &:hover {
        background: #e02f44;
        color: #ffffff;
    }

    &:active {
        background: #c02040;
    }
}

:root.dark {
    .custom-titlebar {
        background: var(--el-bg-color);
        border-bottom-color: var(--el-border-color);
    }

    .control-button {
        &:hover {
            background: var(--el-fill-color-light);
        }
    }
}

@media (max-width: 600px) {
    .app-title {
        display: none;
    }

    .titlebar-left {
        gap: 4px;
    }
}
</style>
