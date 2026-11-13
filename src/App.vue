<template>
  <div class="app-window">
    <!-- 顶部应用标题与全局操作栏 -->
    <header class="app-header" data-tauri-drag-region>
      <div class="brand">
        <span class="logo">⚡</span>
        <h1 class="app-name">Local Share</h1>
      </div>

      <div class="header-right">
        <!-- 动态状态徽章 -->
        <div class="status-indicator">
          <span :class="['dot', { 'is-online': hasActiveTunnels, 'is-loading': isAnyStarting }]"></span>
          <span class="status-text">{{ currentStatusText }}</span>
        </div>

        <!-- 一键停止所有分享按钮 -->
        <button
          v-if="hasActiveTunnels"
          class="btn-stop-all"
          title="停止并关闭所有公网分享"
          @click="stopAll"
        >
          全部停止
        </button>
      </div>
    </header>

    <!-- 主体工作台内容区 -->
    <main class="app-body">
      <!-- 端口分享输入栏 -->
      <CustomPortInput @share="startShare" />

      <!-- 已分享端口列表卡片流 -->
      <ServiceList
        :active-tunnels="activeTunnels"
        @start-share="startShare"
        @stop-share="stopShare"
        @copy-url="copyPublicUrl"
        @open-browser="openInBrowser"
        @dismiss="dismissTunnel"
      />
    </main>

    <!-- 首次按需下载穿透引擎进度弹窗 -->
    <EngineModal
      :is-visible="isEngineModalVisible"
      :is-downloading="isDownloadingEngine"
      :percent="engineDownloadPercent"
      :downloaded="engineDownloadedBytes"
      :total="engineTotalBytes"
      :error-message="engineDownloadError"
      @start-download="triggerDownloadEngine"
      @cancel="cancelEngineDownload"
    />
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useTunnel } from '@/hooks/useTunnel.js'
import ServiceList from '@/components/ServiceList.vue'
import CustomPortInput from '@/components/CustomPortInput.vue'
import EngineModal from '@/components/EngineModal.vue'

const {
  activeTunnels,
  activeCount,
  hasActiveTunnels,
  isAnyStarting,
  isEngineModalVisible,
  isDownloadingEngine,
  engineDownloadPercent,
  engineDownloadedBytes,
  engineTotalBytes,
  engineDownloadError,
  triggerDownloadEngine,
  cancelEngineDownload,
  startShare,
  stopShare,
  stopAll,
  copyPublicUrl,
  openInBrowser,
  dismissTunnel
} = useTunnel()

// 顶部状态文案
const currentStatusText = computed(() => {
  if (isAnyStarting.value) return '正在连接...'
  if (activeCount.value > 0) return `${activeCount.value} 个分享中`
  return '就绪'
})
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables.scss' as *;

.app-window {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: $color-bg-primary;
  padding: 14px 16px;
  gap: 12px;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 8px;
  border-bottom: 1px solid $color-border-subtle;
  user-select: none;

  .brand {
    display: flex;
    align-items: center;
    gap: 6px;

    .logo {
      font-size: 16px;
    }

    .app-name {
      font-size: 15px;
      font-weight: 700;
      letter-spacing: -0.2px;
      color: $color-text-primary;
    }
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;

    .dot {
      width: 6px;
      height: 6px;
      background: $color-text-muted;
      border-radius: $radius-full;
      transition: all $transition-fast;

      &.is-online {
        background: $color-success;
        box-shadow: $shadow-glow-success;
      }

      &.is-loading {
        background: $color-warning;
        animation: spin 1s linear infinite;
      }
    }

    .status-text {
      font-size: 11px;
      color: $color-text-secondary;
      font-weight: 500;
    }
  }

  .btn-stop-all {
    font-size: 11px;
    padding: 3px 8px;
    background: rgba(239, 68, 68, 0.15);
    color: $color-danger;
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: $radius-sm;
    font-weight: 500;
    transition: all $transition-fast;

    &:hover {
      background: $color-danger;
      color: #fff;
    }
  }
}

.app-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
</style>
