<template>
  <div class="service-list-container">
    <div class="header">
      <div class="title-group">
        <span class="title">已分享端口</span>
        <span v-if="tunnelList.length > 0" class="count-badge">
          {{ tunnelList.length }} 个运行中
        </span>
      </div>
    </div>

    <!-- 列表展示区域 -->
    <div class="list-wrapper">
      <div v-if="tunnelList.length === 0" class="empty-state">
        <div class="empty-icon">🌐</div>
        <p class="empty-title">暂无分享中的端口</p>
        <p class="empty-desc">在上方输入本地开发端口（如 5173、3000、8080），点击“开始分享”秒级生成公网访问链接。</p>
      </div>

      <div v-else class="services-flow">
        <div
          v-for="item in tunnelList"
          :key="item.port"
          :class="[
            'service-card',
            {
              'is-running': item.status === 'running',
              'is-starting': item.status === 'starting',
              'is-error': item.status === 'error'
            }
          ]"
        >
          <!-- 卡片顶栏：端口与状态/操作 -->
          <div class="card-head">
            <div class="head-left">
              <span class="port-badge">:{{ item.port }}</span>
              <span class="service-name">本地 Web 服务</span>
            </div>

            <!-- 右侧操作区 -->
            <div class="head-right">
              <!-- 运行中状态：计时器 + 停止按钮 -->
              <template v-if="item.status === 'running'">
                <div class="running-indicator">
                  <span class="pulse-dot"></span>
                  <span class="timer-text">{{ formatDuration(item.runningSeconds) }}</span>
                </div>
                <button
                  class="btn-action btn-stop"
                  title="停止当前端口分享"
                  @click.stop="emit('stopShare', item.port)"
                >
                  停止
                </button>
              </template>

              <!-- 启动中状态 -->
              <template v-else-if="item.status === 'starting'">
                <span class="starting-tag">
                  <span class="spinner-tiny"></span>
                  连接中...
                </span>
              </template>

              <!-- 异常状态：重试 / 移除 -->
              <template v-else-if="item.status === 'error'">
                <button
                  class="btn-action btn-retry"
                  title="重试"
                  @click.stop="emit('startShare', item.port)"
                >
                  重试
                </button>
                <button
                  class="btn-action btn-dismiss"
                  title="关闭"
                  @click.stop="emit('dismiss', item.port)"
                >
                  ✕
                </button>
              </template>
            </div>
          </div>

          <!-- 本地地址行 -->
          <div class="card-meta">
            <span class="meta-label">本地:</span>
            <span class="meta-value">http://127.0.0.1:{{ item.port }}</span>
          </div>

          <!-- 展开区一：正在运行公网面板 -->
          <div v-if="item.status === 'running'" class="tunnel-body">
            <div class="public-url-box">
              <span class="url-text">{{ item.publicUrl }}</span>
            </div>
            <div class="tunnel-actions">
              <button
                class="btn-tunnel btn-copy"
                @click.stop="emit('copyUrl', item.port)"
              >
                <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
                <span>{{ item.isCopied ? '已复制！' : '复制地址' }}</span>
              </button>

              <button
                class="btn-tunnel btn-open"
                @click.stop="emit('openBrowser', item.publicUrl)"
              >
                <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                  <polyline points="15 3 21 3 21 9"></polyline>
                  <line x1="10" y1="14" x2="21" y2="3"></line>
                </svg>
                <span>浏览器打开</span>
              </button>
            </div>
          </div>

          <!-- 展开区二：启动中提示 -->
          <div v-else-if="item.status === 'starting'" class="starting-body">
            <span class="starting-text">正在向 Cloudflare 申请临时公网通道 (约 3~5 秒)...</span>
          </div>

          <!-- 展开区三：异常提示 -->
          <div v-else-if="item.status === 'error'" class="error-body">
            <span class="error-text">{{ item.errorMessage || '公网隧道连接失败' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { formatDuration } from '@/utils/format.js'

const props = defineProps({
  activeTunnels: {
    type: Object,
    default: () => ({})
  }
})

const emit = defineEmits([
  'startShare',
  'stopShare',
  'copyUrl',
  'openBrowser',
  'dismiss'
])

const tunnelList = computed(() => {
  return Object.values(props.activeTunnels).sort((a, b) => b.port - a.port)
})
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables.scss' as *;

.service-list-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
  flex: 1;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;

    .title {
      font-size: 13px;
      font-weight: 600;
      color: $color-text-primary;
    }

    .count-badge {
      font-size: 11px;
      padding: 1px 6px;
      background: rgba(59, 130, 246, 0.15);
      color: $color-primary;
      border-radius: $radius-sm;
    }
  }
}

.list-wrapper {
  flex: 1;
  overflow-y: auto;
  min-height: 180px;
  background: $color-bg-card;
  border: 1px solid $color-border-subtle;
  border-radius: $radius-md;
  padding: 8px;
  display: flex;
  flex-direction: column;
}

.services-flow {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.service-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: $radius-sm;
  transition: all $transition-fast;

  &:hover {
    background: $color-bg-card-hover;
    border-color: rgba(255, 255, 255, 0.12);
  }

  &.is-running {
    background: linear-gradient(145deg, rgba(30, 41, 59, 0.95), rgba(15, 23, 42, 0.95));
    border: 1px solid rgba(16, 185, 129, 0.35);
    box-shadow: $shadow-glow-success;
  }

  &.is-starting {
    border-color: rgba(245, 158, 11, 0.35);
    background: rgba(245, 158, 11, 0.03);
  }

  &.is-error {
    border-color: rgba(239, 68, 68, 0.35);
    background: rgba(239, 68, 68, 0.03);
  }
}

.card-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;

  .head-left {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    flex: 1;

    .port-badge {
      font-size: 13px;
      font-weight: 700;
      color: $color-text-primary;
      font-family: monospace;
      padding: 1px 6px;
      background: rgba(255, 255, 255, 0.06);
      border-radius: $radius-sm;
    }

    .service-name {
      font-size: 12px;
      color: $color-text-secondary;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }

  .head-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;

    .running-indicator {
      display: flex;
      align-items: center;
      gap: 5px;

      .pulse-dot {
        width: 6px;
        height: 6px;
        background: $color-success;
        border-radius: $radius-full;
        box-shadow: 0 0 8px $color-success;
        animation: pulse 1.8s infinite;
      }

      .timer-text {
        font-size: 11px;
        font-family: monospace;
        color: $color-success;
      }
    }

    .starting-tag {
      display: flex;
      align-items: center;
      gap: 4px;
      font-size: 11px;
      color: $color-warning;

      .spinner-tiny {
        width: 10px;
        height: 10px;
        border: 2px solid rgba(245, 158, 11, 0.2);
        border-top-color: $color-warning;
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
      }
    }

    .btn-action {
      font-size: 11px;
      padding: 3px 8px;
      border-radius: $radius-sm;
      font-weight: 500;
      transition: all $transition-fast;

      &.btn-stop {
        background: rgba(239, 68, 68, 0.15);
        color: $color-danger;
        border: 1px solid rgba(239, 68, 68, 0.3);

        &:hover {
          background: $color-danger;
          color: #fff;
        }
      }

      &.btn-retry {
        background: rgba(245, 158, 11, 0.15);
        color: $color-warning;
        border: 1px solid rgba(245, 158, 11, 0.3);

        &:hover {
          background: $color-warning;
          color: #000;
        }
      }

      &.btn-dismiss {
        background: rgba(255, 255, 255, 0.05);
        color: $color-text-muted;
        border: 1px solid transparent;

        &:hover {
          background: rgba(255, 255, 255, 0.12);
          color: $color-text-primary;
        }
      }
    }
  }
}

.card-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: $color-text-muted;

  .meta-value {
    font-family: monospace;
    color: $color-text-secondary;
  }
}

.tunnel-body {
  margin-top: 4px;
  display: flex;
  flex-direction: column;
  gap: 6px;

  .public-url-box {
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(16, 185, 129, 0.25);
    border-radius: $radius-sm;
    padding: 6px 8px;
    overflow: hidden;

    .url-text {
      font-size: 12px;
      color: #34d399;
      font-family: monospace;
      word-break: break-all;
      user-select: all;
    }
  }

  .tunnel-actions {
    display: flex;
    gap: 6px;

    .btn-tunnel {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 5px;
      padding: 5px 8px;
      font-size: 11px;
      font-weight: 500;
      border-radius: $radius-sm;
      transition: all $transition-fast;

      &.btn-copy {
        background: rgba(16, 185, 129, 0.12);
        color: #34d399;
        border: 1px solid rgba(16, 185, 129, 0.25);

        &:hover {
          background: rgba(16, 185, 129, 0.22);
        }
      }

      &.btn-open {
        background: rgba(59, 130, 246, 0.12);
        color: #60a5fa;
        border: 1px solid rgba(59, 130, 246, 0.25);

        &:hover {
          background: rgba(59, 130, 246, 0.22);
        }
      }
    }
  }
}

.starting-body {
  margin-top: 2px;
  font-size: 11px;
  color: $color-warning;
}

.error-body {
  margin-top: 2px;
  font-size: 11px;
  color: $color-danger;
  word-break: break-all;
}

.empty-state {
  margin: auto 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 32px 16px;
  gap: 8px;

  .empty-icon {
    font-size: 32px;
    opacity: 0.7;
    margin-bottom: 4px;
  }

  .empty-title {
    font-size: 13px;
    font-weight: 600;
    color: $color-text-secondary;
  }

  .empty-desc {
    font-size: 11px;
    color: $color-text-muted;
    max-width: 280px;
    line-height: 1.5;
  }
}

@keyframes pulse {
  0% { transform: scale(0.95); opacity: 0.8; }
  50% { transform: scale(1.15); opacity: 1; }
  100% { transform: scale(0.95); opacity: 0.8; }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
