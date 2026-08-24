<template>
  <div v-if="isVisible" class="settings-modal-backdrop" @click.self="emit('close')">
    <div class="settings-modal">
      <div class="modal-header">
        <div class="icon-wrap">⚙️</div>
        <h3 class="modal-title">穿透与域名设置</h3>
        <button class="btn-close" title="关闭" @click="emit('close')">✕</button>
      </div>

      <div class="modal-content">
        <!-- 穿透模式切换 -->
        <div class="form-item">
          <label class="item-label">穿透模式</label>
          <div class="mode-tabs">
            <button
              :class="['mode-tab', { 'is-active': localMode === 'custom' }]"
              @click="localMode = 'custom'"
            >
              固定域名 (Token)
            </button>
            <button
              :class="['mode-tab', { 'is-active': localMode === 'quick' }]"
              @click="localMode = 'quick'"
            >
              临时随机域名
            </button>
          </div>
        </div>

        <!-- 固定域名模式配置 -->
        <template v-if="localMode === 'custom'">
          <div class="form-item">
            <label class="item-label" for="custom-domain-input">固定公网域名</label>
            <input
              id="custom-domain-input"
              v-model.trim="localCustomDomain"
              class="text-input"
              type="text"
              placeholder="例如 du1.ccwu.cc"
              spellcheck="false"
            />
            <span class="field-hint">分享成功后将通过此域名访问本地服务</span>
          </div>

          <div class="form-item">
            <label class="item-label" for="tunnel-token-input">Cloudflare Tunnel Token</label>
            <textarea
              id="tunnel-token-input"
              v-model.trim="localToken"
              class="token-textarea"
              rows="3"
              placeholder="粘贴 Cloudflare Zero Trust 生成的 Token (以 eyJh... 开头)"
              spellcheck="false"
            ></textarea>
            <span class="field-hint">
              可在 Cloudflare Zero Trust -> Networks -> Tunnels 获取
            </span>
          </div>
        </template>

        <!-- 临时随机域名模式说明 -->
        <template v-else>
          <div class="quick-mode-desc">
            <p>免配置即可开启公网分享，每次启动将随机分配形如 <code>*.trycloudflare.com</code> 的临时二级域名。</p>
          </div>
        </template>
      </div>

      <div class="modal-actions">
        <button class="btn-cancel" @click="emit('close')">取消</button>
        <button class="btn-save" @click="handleSave">保存配置</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, defineProps, defineEmits } from 'vue'

const props = defineProps({
  isVisible: {
    type: Boolean,
    default: false
  },
  settings: {
    type: Object,
    default: () => ({
      mode: 'custom',
      token: '',
      customDomain: 'du1.ccwu.cc'
    })
  }
})

const emit = defineEmits(['close', 'save'])

const localMode = ref('custom')
const localToken = ref('')
const localCustomDomain = ref('du1.ccwu.cc')

watch(
  () => props.isVisible,
  (visible) => {
    if (visible) {
      localMode.value = props.settings?.mode ?? 'custom'
      localToken.value = props.settings?.token ?? ''
      localCustomDomain.value = props.settings?.customDomain ?? 'du1.ccwu.cc'
    }
  },
  { immediate: true }
)

const handleSave = () => {
  let cleanDomain = localCustomDomain.value
    .replace(/^https?:\/\//i, '')
    .replace(/\/.*$/, '')
    .trim()

  if (!cleanDomain) {
    cleanDomain = 'du1.ccwu.cc'
  }

  emit('save', {
    mode: localMode.value,
    token: localToken.value.trim(),
    customDomain: cleanDomain
  })
  emit('close')
}
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables.scss' as *;

.settings-modal-backdrop {
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

.settings-modal {
  width: 100%;
  max-width: 400px;
  background: $color-bg-card;
  border: 1px solid $color-border-subtle;
  border-radius: $radius-lg;
  box-shadow: $shadow-glow-primary;
  padding: 18px 20px;
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
    width: 28px;
    height: 28px;
    background: rgba(59, 130, 246, 0.15);
    color: $color-primary;
    border-radius: $radius-md;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 15px;
  }

  .modal-title {
    font-size: 15px;
    font-weight: 700;
    color: $color-text-primary;
    flex: 1;
  }

  .btn-close {
    background: transparent;
    color: $color-text-muted;
    font-size: 14px;
    padding: 4px;
    border-radius: $radius-sm;
    transition: all $transition-fast;

    &:hover {
      background: rgba(255, 255, 255, 0.1);
      color: $color-text-primary;
    }
  }
}

.modal-content {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 6px;

  .item-label {
    font-size: 12px;
    font-weight: 600;
    color: $color-text-primary;
  }

  .field-hint {
    font-size: 11px;
    color: $color-text-muted;
    line-height: 1.4;
  }
}

.mode-tabs {
  display: flex;
  background: $color-bg-input;
  padding: 3px;
  border-radius: $radius-sm;
  border: 1px solid $color-border-subtle;
  gap: 4px;

  .mode-tab {
    flex: 1;
    padding: 6px;
    font-size: 12px;
    font-weight: 500;
    color: $color-text-secondary;
    border-radius: $radius-sm;
    background: transparent;
    transition: all $transition-fast;

    &.is-active {
      background: $color-primary;
      color: #fff;
      font-weight: 600;
    }
  }
}

.text-input,
.token-textarea {
  width: 100%;
  background: $color-bg-input;
  border: 1px solid $color-border-subtle;
  border-radius: $radius-sm;
  color: $color-text-primary;
  font-size: 12px;
  padding: 8px 10px;
  outline: none;
  font-family: inherit;
  transition: border-color $transition-fast;

  &:focus {
    border-color: $color-border-focus;
  }

  &::placeholder {
    color: $color-text-muted;
  }
}

.token-textarea {
  resize: vertical;
  min-height: 60px;
  font-family: monospace;
  font-size: 11px;
  line-height: 1.4;
  word-break: break-all;
}

.quick-mode-desc {
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid $color-border-subtle;
  border-radius: $radius-sm;

  p {
    font-size: 12px;
    line-height: 1.6;
    color: $color-text-secondary;

    code {
      background: rgba(255, 255, 255, 0.1);
      color: $color-primary;
      padding: 2px 4px;
      border-radius: 3px;
      font-family: monospace;
    }
  }
}

.modal-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 4px;

  .btn-save {
    padding: 7px 16px;
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
    padding: 7px 14px;
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
