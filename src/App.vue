<template>
  <div class="app-window">
    <!-- 顶部应用标题与全局操作栏 -->
    <header class="app-header" data-tauri-drag-region>
      <div class="brand">
        <span class="logo">⚡</span>
        <h1 class="app-name">Local Share</h1>
      </div>

      <div class="header-right">
        <!-- 模式标签按钮 -->
        <button
          class="mode-badge-btn"
          :title="settings.mode === 'custom' ? `固定域名模式 (${settings.customConfig?.baseDomain || settings.customDomain || 'ccwu.cc'})` : '随机域名模式'"
          @click="openSettings"
        >
          <span class="mode-icon">{{ settings.mode === 'custom' ? '🔒' : '🎲' }}</span>
          <span class="mode-name">{{ settings.mode === 'custom' ? (settings.customConfig?.baseDomain || settings.customDomain || 'ccwu.cc') : '随机' }}</span>
        </button>

        <!-- 设置入口按钮 -->
        <button
          class="btn-icon-setting"
          title="穿透与域名设置"
          @click="openSettings"
        >
          ⚙️
        </button>

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

    <!-- 全局扫描异常提示 -->
    <div v-if="scanError" class="notice-area">
      <ErrorNotice
        :message="scanError"
        @close="scanError = ''"
        @retry="scanServices"
      />
    </div>

    <!-- 主体工作台内容区 -->
    <main class="app-body">
      <!-- 多服务卡片流工作台 -->
      <ServiceList
        :services="services"
        :active-tunnels="activeTunnels"
        :is-scanning="isScanning"
        @start-share="startShare"
        @stop-share="stopShare"
        @copy-url="copyPublicUrl"
        @open-browser="openInBrowser"
        @dismiss="dismissTunnel"
        @refresh="scanServices"
      />

      <!-- 手动添加端口分享 -->
      <div class="bottom-tools">
        <CustomPortInput @share="startShare" />
      </div>
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

    <!-- 穿透模式与域名设置弹窗 -->
    <SettingsModal
      :is-visible="isSettingsModalVisible"
      :settings="settings"
      @close="closeSettings"
      @save="updateSettings"
    />
  </div>
</template>

<script setup>
import { computed, onMounted } from 'vue'
import { useServices } from '@/hooks/useServices.js'
import { useTunnel } from '@/hooks/useTunnel.js'
import { useSettings } from '@/hooks/useSettings.js'
import ServiceList from '@/components/ServiceList.vue'
import CustomPortInput from '@/components/CustomPortInput.vue'
import ErrorNotice from '@/components/ErrorNotice.vue'
import EngineModal from '@/components/EngineModal.vue'
import SettingsModal from '@/components/SettingsModal.vue'

const {
  services,
  isScanning,
  scanError,
  scanServices
} = useServices()

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

const {
  settings,
  isSettingsModalVisible,
  openSettings,
  closeSettings,
  updateSettings
} = useSettings()

// 顶部状态文案
const currentStatusText = computed(() => {
  if (isAnyStarting.value) return '正在连接...'
  if (activeCount.value > 0) return `${activeCount.value} 个分享中`
  return '就绪'
})

onMounted(() => {
  scanServices()
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
  gap: 10px;
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

    .mode-badge-btn {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 3px 8px;
      background: rgba(59, 130, 246, 0.12);
      border: 1px solid rgba(59, 130, 246, 0.25);
      border-radius: $radius-full;
      color: $color-primary;
      font-size: 11px;
      font-weight: 500;
      cursor: pointer;
      max-width: 120px;
      transition: all $transition-fast;

      &:hover {
        background: rgba(59, 130, 246, 0.2);
        border-color: rgba(59, 130, 246, 0.4);
      }

      .mode-icon {
        font-size: 10px;
      }

      .mode-name {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
    }

    .btn-icon-setting {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 24px;
      height: 24px;
      background: rgba(255, 255, 255, 0.05);
      border: 1px solid $color-border-subtle;
      border-radius: $radius-sm;
      font-size: 12px;
      cursor: pointer;
      transition: all $transition-fast;

      &:hover {
        background: rgba(255, 255, 255, 0.12);
        border-color: rgba(255, 255, 255, 0.2);
      }
    }
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

.notice-area {
  flex-shrink: 0;
}

.app-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.bottom-tools {
  flex-shrink: 0;
}
</style>
