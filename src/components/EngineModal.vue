<template>
  <div v-if="isVisible" class="engine-modal-backdrop">
    <div class="engine-modal">
      <div class="modal-header">
        <div class="icon-wrap">⚡</div>
        <h3 class="modal-title">准备穿透加速引擎</h3>
      </div>

      <div class="modal-content">
        <p class="desc-text">
          首次使用需要自动就绪轻量穿透引擎，仅需下载一次，之后即可永久秒开。
        </p>

        <!-- 进度条区 -->
        <div v-if="isDownloading" class="progress-section">
          <div class="progress-bar-bg">
            <div class="progress-bar-fill" :style="{ width: `${percent}%` }"></div>
          </div>
          <div class="progress-meta">
            <span class="status-label">正在下载加速组件...</span>
            <span class="percent-label">{{ percent }}%</span>
          </div>
          <p class="size-detail" v-if="total > 0">
            {{ formatMb(downloaded) }} MB / {{ formatMb(total) }} MB
          </p>
        </div>

        <!-- 异常提示 -->
        <div v-if="errorMessage" class="error-banner">
          <span class="error-icon">⚠️</span>
          <span class="error-text">{{ errorMessage }}</span>
        </div>
      </div>

      <div class="modal-actions">
        <button
          v-if="!isDownloading"
          class="btn-confirm"
          @click="emit('startDownload')"
        >
          {{ errorMessage ? '重试下载' : '立即就绪引擎' }}
        </button>
        <button
          v-if="!isDownloading && canCancel"
          class="btn-cancel"
          @click="emit('cancel')"
        >
          取消
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue'

const props = defineProps({
  isVisible: {
    type: Boolean,
    default: false
  },
  isDownloading: {
    type: Boolean,
    default: false
  },
  percent: {
    type: Number,
    default: 0
  },
  downloaded: {
    type: Number,
    default: 0
  },
  total: {
    type: Number,
    default: 0
  },
  errorMessage: {
    type: String,
    default: ''
  },
  canCancel: {
    type: Boolean,
    default: true
  }
})

const emit = defineEmits(['startDownload', 'cancel'])

const formatMb = (bytes) => {
  if (!bytes) return '0.0'
  return (bytes / (1024 * 1024)).toFixed(1)
}
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables.scss' as *;

.engine-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
  animation: fadeIn 0.2s ease;
}

.engine-modal {
  width: 100%;
  max-width: 360px;
  background: $color-bg-card;
  border: 1px solid $color-border-subtle;
  border-radius: $radius-lg;
  box-shadow: $shadow-glow-primary;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: scaleIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 10px;

  .icon-wrap {
    width: 32px;
    height: 32px;
    background: rgba(59, 130, 246, 0.15);
    color: $color-primary;
    border-radius: $radius-md;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
  }

  .modal-title {
    font-size: 15px;
    font-weight: 700;
    color: $color-text-primary;
  }
}

.modal-content {
  display: flex;
  flex-direction: column;
  gap: 12px;

  .desc-text {
    font-size: 12px;
    line-height: 1.6;
    color: $color-text-secondary;
  }
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: $radius-sm;
  border: 1px solid rgba(255, 255, 255, 0.05);

  .progress-bar-bg {
    width: 100%;
    height: 6px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 99px;
    overflow: hidden;

    .progress-bar-fill {
      height: 100%;
      background: linear-gradient(90deg, #3b82f6, #60a5fa);
      border-radius: 99px;
      transition: width 0.2s ease;
    }
  }

  .progress-meta {
    display: flex;
    justify-content: space-between;
    font-size: 11px;

    .status-label {
      color: $color-text-secondary;
    }

    .percent-label {
      font-family: monospace;
      color: $color-primary;
      font-weight: 700;
    }
  }

  .size-detail {
    font-size: 10px;
    font-family: monospace;
    color: $color-text-muted;
    text-align: right;
  }
}

.error-banner {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 8px 10px;
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.25);
  border-radius: $radius-sm;

  .error-icon {
    font-size: 13px;
  }

  .error-text {
    font-size: 11px;
    color: $color-danger;
    line-height: 1.4;
  }
}

.modal-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;

  .btn-confirm {
    flex: 1;
    padding: 9px;
    background: $color-primary;
    color: #fff;
    border-radius: $radius-sm;
    font-size: 12px;
    font-weight: 600;
    box-shadow: $shadow-glow-primary;
    transition: all $transition-fast;

    &:hover {
      background: $color-primary-hover;
    }
  }

  .btn-cancel {
    padding: 9px 14px;
    background: rgba(255, 255, 255, 0.08);
    color: $color-text-secondary;
    border-radius: $radius-sm;
    font-size: 12px;
    font-weight: 500;
    transition: all $transition-fast;

    &:hover {
      background: rgba(255, 255, 255, 0.12);
      color: $color-text-primary;
    }
  }
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes scaleIn {
  from { transform: scale(0.95); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}
</style>
