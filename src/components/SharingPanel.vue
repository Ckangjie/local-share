<template>
  <div class="sharing-panel">
    <!-- 状态指示顶部 -->
    <div class="status-header">
      <div class="status-badge">
        <span class="status-dot"></span>
        <span class="status-title">公网分享中</span>
      </div>
      <div class="timer-tag">
        已运行 {{ formattedTime }}
      </div>
    </div>

    <!-- 本地源地址 -->
    <div class="info-block">
      <span class="label">本地服务地址</span>
      <div class="local-box">
        <span class="url-text">http://localhost:{{ localPort }}</span>
      </div>
    </div>

    <!-- 公网分享地址卡片 -->
    <div class="public-card">
      <div class="card-header">
        <span class="label">公网访问地址</span>
        <span class="cf-tag">Cloudflare Quick Tunnel</span>
      </div>
      <div class="url-display">
        <span class="url-link">{{ publicUrl }}</span>
      </div>

      <div class="action-buttons">
        <button class="btn btn-copy" @click="emit('copy')">
          <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
          <span>{{ isCopied ? '已复制！' : '复制地址' }}</span>
        </button>

        <button class="btn btn-open" @click="emit('openBrowser')">
          <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
            <polyline points="15 3 21 3 21 9"></polyline>
            <line x1="10" y1="14" x2="21" y2="3"></line>
          </svg>
          <span>在浏览器中打开</span>
        </button>
      </div>
    </div>

    <!-- 停止分享操作 -->
    <div class="footer-actions">
      <button 
        class="btn-stop" 
        :disabled="isStopping" 
        @click="emit('stop')"
      >
        <span v-if="isStopping" class="spinner-sm"></span>
        <span>{{ isStopping ? '正在停止...' : '停止分享' }}</span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { formatDuration } from '@/utils/format.js'

const props = defineProps({
  publicUrl: {
    type: String,
    required: true
  },
  localPort: {
    type: [Number, String],
    default: ''
  },
  runningSeconds: {
    type: Number,
    default: 0
  },
  isCopied: {
    type: Boolean,
    default: false
  },
  isStopping: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['copy', 'openBrowser', 'stop'])

const formattedTime = computed(() => formatDuration(props.runningSeconds))
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables.scss' as *;

.sharing-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
  flex: 1;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.25);
  border-radius: $radius-md;

  .status-badge {
    display: flex;
    align-items: center;
    gap: 8px;

    .status-dot {
      width: 8px;
      height: 8px;
      background: $color-success;
      border-radius: $radius-full;
      animation: pulse-glow 2s infinite;
    }

    .status-title {
      font-size: 13px;
      font-weight: 600;
      color: $color-success;
    }
  }

  .timer-tag {
    font-size: 12px;
    font-family: monospace;
    color: $color-text-secondary;
  }
}

.info-block {
  display: flex;
  flex-direction: column;
  gap: 4px;

  .label {
    font-size: 11px;
    color: $color-text-muted;
    font-weight: 500;
  }

  .local-box {
    padding: 8px 12px;
    background: $color-bg-card;
    border: 1px solid $color-border-subtle;
    border-radius: $radius-sm;

    .url-text {
      font-size: 12px;
      font-family: monospace;
      color: $color-text-secondary;
    }
  }
}

.public-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  background: linear-gradient(145deg, rgba(30, 41, 59, 0.9), rgba(15, 23, 42, 0.95));
  border: 1px solid rgba(59, 130, 246, 0.3);
  box-shadow: $shadow-glow-primary;
  border-radius: $radius-lg;

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .label {
      font-size: 12px;
      font-weight: 600;
      color: $color-text-primary;
    }

    .cf-tag {
      font-size: 10px;
      padding: 2px 6px;
      background: rgba(249, 115, 22, 0.15);
      color: $color-accent-cf;
      border-radius: $radius-sm;
      font-weight: 500;
    }
  }

  .url-display {
    padding: 10px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px dashed rgba(59, 130, 246, 0.4);
    border-radius: $radius-sm;
    overflow: hidden;

    .url-link {
      font-size: 13px;
      font-family: monospace;
      color: $color-text-link;
      word-break: break-all;
      user-select: text;
    }
  }

  .action-buttons {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;

    .btn {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      padding: 8px 12px;
      border-radius: $radius-sm;
      font-size: 12px;
      font-weight: 500;
      transition: all $transition-fast;

      &.btn-copy {
        background: $color-primary;
        color: #fff;

        &:hover {
          background: $color-primary-hover;
        }
      }

      &.btn-open {
        background: rgba(255, 255, 255, 0.08);
        color: $color-text-primary;

        &:hover {
          background: rgba(255, 255, 255, 0.15);
        }
      }
    }
  }
}

.footer-actions {
  margin-top: auto;
  padding-top: 8px;

  .btn-stop {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px;
    background: rgba(239, 68, 68, 0.15);
    color: $color-danger;
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: $radius-md;
    font-size: 13px;
    font-weight: 600;

    &:hover:not(:disabled) {
      background: $color-danger;
      color: #fff;
    }
  }
}

.spinner-sm {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(239, 68, 68, 0.3);
  border-top-color: $color-danger;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
</style>
