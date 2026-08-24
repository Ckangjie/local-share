<template>
  <div class="service-list-container">
    <div class="header">
      <div class="title-group">
        <span class="title">本地 Web 服务</span>
        <span v-if="!isScanning && allDisplayServices.length > 0" class="count-badge">
          {{ allDisplayServices.length }} 个发现
        </span>
      </div>
      <button 
        class="refresh-btn" 
        :disabled="isScanning" 
        title="重新扫描本地端口" 
        @click="emit('refresh')"
      >
        <svg 
          :class="['refresh-icon', { 'spin-loading': isScanning }]" 
          viewBox="0 0 24 24" 
          width="14" 
          height="14" 
          stroke="currentColor" 
          stroke-width="2" 
          fill="none" 
          stroke-linecap="round" 
          stroke-linejoin="round"
        >
          <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/>
        </svg>
        <span>{{ isScanning ? '扫描中' : '刷新' }}</span>
      </button>
    </div>

    <!-- 列表展示区域 -->
    <div class="list-wrapper">
      <div v-if="isScanning && allDisplayServices.length === 0" class="scanning-state">
        <div class="spinner"></div>
        <p class="scan-text">正在嗅探本机监听端口...</p>
      </div>

      <div v-else-if="allDisplayServices.length === 0" class="empty-state">
        <div class="empty-icon">🌐</div>
        <p class="empty-title">未检测到活跃的 Web 服务</p>
        <p class="empty-desc">请确认本地开发服务已启动，或在下方手动输入端口开启分享。</p>
      </div>

      <div v-else class="services-flow">
        <div
          v-for="item in allDisplayServices"
          :key="item.port"
          :class="[
            'service-card',
            {
              'is-running': activeTunnels[item.port]?.status === 'running',
              'is-starting': activeTunnels[item.port]?.status === 'starting',
              'is-error': activeTunnels[item.port]?.status === 'error'
            }
          ]"
        >
          <!-- 卡片顶栏：端口、技术栈徽章、名称与主状态/操作 -->
          <div class="card-head">
            <div class="head-left">
              <span class="port-badge">:{{ item.port }}</span>
              <span v-if="item.tagLabel" :class="['type-badge', `type-${item.tag || 'web'}`]">
                {{ item.tagLabel }}
              </span>
              <span class="service-name" :title="item.title">{{ item.title || 'Web Service' }}</span>
            </div>

            <!-- 右侧操作区 -->
            <div class="head-right">
              <!-- 运行中状态：计时器 + 停止按钮 -->
              <template v-if="activeTunnels[item.port]?.status === 'running'">
                <div class="running-indicator">
                  <span class="pulse-dot"></span>
                  <span class="timer-text">{{ formatDuration(activeTunnels[item.port].runningSeconds) }}</span>
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
              <template v-else-if="activeTunnels[item.port]?.status === 'starting'">
                <span class="starting-tag">
                  <span class="spinner-tiny"></span>
                  连接中...
                </span>
              </template>

              <!-- 异常状态：重试 / 移除 -->
              <template v-else-if="activeTunnels[item.port]?.status === 'error'">
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

              <!-- 未分享状态：一键分享主按钮 -->
              <template v-else>
                <button
                  class="btn-action btn-start"
                  @click.stop="emit('startShare', item.port)"
                >
                  ⚡ 分享
                </button>
              </template>
            </div>
          </div>

          <!-- 本地地址行 -->
          <div class="card-meta">
            <span class="meta-label">本地:</span>
            <span class="meta-value">{{ item.url || `http://localhost:${item.port}` }}</span>
          </div>

          <!-- 展开区一：正在运行公网面板 -->
          <div v-if="activeTunnels[item.port]?.status === 'running'" class="tunnel-body">
            <div class="public-url-box">
              <span class="url-text">{{ activeTunnels[item.port].publicUrl }}</span>
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
                <span>{{ activeTunnels[item.port].isCopied ? '已复制！' : '复制地址' }}</span>
              </button>

              <button
                class="btn-tunnel btn-open"
                @click.stop="emit('openBrowser', activeTunnels[item.port].publicUrl)"
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
          <div v-else-if="activeTunnels[item.port]?.status === 'starting'" class="starting-body">
            <span class="starting-text">正在向 Cloudflare 申请临时公网通道 (约 3~5 秒)...</span>
          </div>

          <!-- 展开区三：异常提示 -->
          <div v-else-if="activeTunnels[item.port]?.status === 'error'" class="error-body">
            <span class="error-text">{{ activeTunnels[item.port].errorMessage || '公网隧道连接失败' }}</span>
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
  services: {
    type: Array,
    default: () => []
  },
  activeTunnels: {
    type: Object,
    default: () => ({})
  },
  isScanning: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits([
  'startShare',
  'stopShare',
  'copyUrl',
  'openBrowser',
  'dismiss',
  'refresh'
])

// 合并自动扫描到的服务与手动自定义已分享的端口
const allDisplayServices = computed(() => {
  const result = [...props.services]
  const existingPorts = new Set(result.map((s) => s.port))

  // 将 activeTunnels 中存在但 services 中未扫描到的自定义端口加入列表顶部
  Object.keys(props.activeTunnels).forEach((rawPort) => {
    const port = Number(rawPort)
    if (!existingPorts.has(port)) {
      result.unshift({
        port,
        url: `http://localhost:${port}`,
        title: `自定义服务 (:${port})`,
        tag: 'custom',
        tagLabel: '手动分享'
      })
      existingPorts.add(port)
    }
  })

  return result
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

  .refresh-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: $color-text-secondary;
    padding: 3px 8px;
    border-radius: $radius-sm;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid transparent;
    transition: all $transition-fast;

    &:hover:not(:disabled) {
      background: rgba(255, 255, 255, 0.1);
      color: $color-text-primary;
      border-color: rgba(255, 255, 255, 0.1);
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
      background: rgba(255, 255, 255, 0.08);
      border-radius: $radius-sm;
      flex-shrink: 0;
    }

    .type-badge {
      font-size: 10px;
      font-weight: 600;
      padding: 1px 6px;
      border-radius: $radius-sm;
      white-space: nowrap;
      flex-shrink: 0;

      &.type-vite {
        background: rgba(16, 185, 129, 0.15);
        color: #34d399;
        border: 1px solid rgba(16, 185, 129, 0.3);
      }

      &.type-webpack {
        background: rgba(56, 189, 248, 0.15);
        color: #38bdf8;
        border: 1px solid rgba(56, 189, 248, 0.3);
      }

      &.type-live-server {
        background: rgba(249, 115, 22, 0.15);
        color: #fb923c;
        border: 1px solid rgba(249, 115, 22, 0.3);
      }

      &.type-next-nuxt {
        background: rgba(168, 85, 247, 0.15);
        color: #c084fc;
        border: 1px solid rgba(168, 85, 247, 0.3);
      }

      &.type-web {
        background: rgba(148, 163, 184, 0.15);
        color: #cbd5e1;
        border: 1px solid rgba(148, 163, 184, 0.3);
      }

      &.type-custom {
        background: rgba(255, 255, 255, 0.08);
        color: #94a3b8;
        border: 1px solid rgba(255, 255, 255, 0.12);
      }
    }

    .service-name {
      font-size: 12px;
      font-weight: 500;
      color: $color-text-secondary;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }

  .head-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }
}

.card-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-family: monospace;
  color: $color-text-muted;

  .meta-label {
    opacity: 0.7;
  }

  .meta-value {
    color: $color-text-muted;
  }
}

.running-indicator {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(16, 185, 129, 0.1);
  padding: 2px 6px;
  border-radius: $radius-sm;
  border: 1px solid rgba(16, 185, 129, 0.25);

  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: $radius-full;
    background: $color-success;
    animation: pulse-glow 2s infinite;
  }

  .timer-text {
    font-size: 10px;
    font-family: monospace;
    color: $color-success;
    font-weight: 600;
  }
}

.btn-action {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: $radius-sm;
  font-weight: 500;
  transition: all $transition-fast;

  &.btn-start {
    background: $color-primary;
    color: #fff;

    &:hover {
      background: $color-primary-hover;
    }
  }

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
      color: #fff;
    }
  }

  &.btn-dismiss {
    background: rgba(255, 255, 255, 0.06);
    color: $color-text-muted;

    &:hover {
      background: rgba(255, 255, 255, 0.15);
      color: $color-text-primary;
    }
  }
}

.starting-tag {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: $color-warning;
}

.tunnel-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
  padding-top: 6px;
  border-top: 1px dashed rgba(16, 185, 129, 0.2);

  .public-url-box {
    padding: 6px 8px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(59, 130, 246, 0.25);
    border-radius: $radius-sm;
    overflow: hidden;

    .url-text {
      font-size: 12px;
      font-family: monospace;
      color: $color-text-link;
      word-break: break-all;
      user-select: text;
    }
  }

  .tunnel-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;

    .btn-tunnel {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 4px;
      padding: 5px 8px;
      border-radius: $radius-sm;
      font-size: 11px;
      font-weight: 500;
      transition: all $transition-fast;

      &.btn-copy {
        background: rgba(59, 130, 246, 0.2);
        color: $color-primary;
        border: 1px solid rgba(59, 130, 246, 0.35);

        &:hover {
          background: $color-primary;
          color: #fff;
        }
      }

      &.btn-open {
        background: rgba(255, 255, 255, 0.06);
        color: $color-text-primary;
        border: 1px solid rgba(255, 255, 255, 0.08);

        &:hover {
          background: rgba(255, 255, 255, 0.12);
        }
      }
    }
  }
}

.starting-body {
  padding-top: 4px;
  font-size: 11px;
  color: $color-warning;
  line-height: 1.4;
}

.error-body {
  padding-top: 4px;
  font-size: 11px;
  color: $color-danger;
  line-height: 1.4;
}

.empty-state, .scanning-state {
  margin: auto;
  padding: 24px 16px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.empty-icon {
  font-size: 28px;
  opacity: 0.7;
}

.empty-title {
  font-size: 13px;
  font-weight: 600;
  color: $color-text-secondary;
}

.empty-desc {
  font-size: 11px;
  color: $color-text-muted;
  line-height: 1.5;
  max-width: 260px;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(59, 130, 246, 0.2);
  border-top-color: $color-primary;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.spinner-tiny {
  width: 10px;
  height: 10px;
  border: 2px solid rgba(245, 158, 11, 0.3);
  border-top-color: $color-warning;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.scan-text {
  font-size: 12px;
  color: $color-text-secondary;
}
</style>
