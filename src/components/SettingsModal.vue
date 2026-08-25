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
              固定域名 (多服务映射)
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
            <label class="item-label" for="custom-domain-input">主域名 / 根域名</label>
            <input
              id="custom-domain-input"
              v-model.trim="localBaseDomain"
              class="text-input"
              type="text"
              placeholder="例如 ccwu.cc 或 dev.example.com"
              spellcheck="false"
            />
            <span class="field-hint">各服务将自动分配独立二级域名（如 <code>p5173.ccwu.cc</code>）</span>
          </div>

          <div class="form-item">
            <label class="item-label" for="tunnel-id-input">Cloudflare Tunnel ID</label>
            <input
              id="tunnel-id-input"
              v-model.trim="localTunnelId"
              class="text-input"
              type="text"
              placeholder="例如 8a1b2c3d-4e5f-6a7b-8c9d-0e1f2a3b4c5d"
              spellcheck="false"
            />
            <span class="field-hint">通过 <code>cloudflared tunnel create &lt;name&gt;</code> 生成的 UUID</span>
          </div>

          <div class="form-item">
            <label class="item-label" for="tunnel-credentials-input">
              Cloudflare Tunnel Token / 凭据内容 (Credentials JSON)
            </label>
            <textarea
              id="tunnel-credentials-input"
              v-model="localCredentialsJson"
              class="token-textarea"
              rows="3"
              placeholder="直接粘贴 Cloudflare Token (以 eyJh 开头) 或 ~/.cloudflared/*.json 凭据"
              spellcheck="false"
              @input="handleCredentialsInput"
            ></textarea>
            <span class="field-hint">
              支持直接粘贴 Zero Trust Token 或 JSON 凭据，将自动识别 TunnelID 并生成动态 Ingress 规则
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
import { ref, watch } from 'vue'

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
      customDomain: 'ccwu.cc',
      customConfig: {
        tunnelId: '',
        credentialsJson: '',
        baseDomain: 'ccwu.cc',
        subdomainPattern: 'p{port}'
      }
    })
  }
})

const emit = defineEmits(['close', 'save'])

const localMode = ref('custom')
const localBaseDomain = ref('ccwu.cc')
const localTunnelId = ref('')
const localCredentialsJson = ref('')

watch(
  () => props.isVisible,
  (visible) => {
    if (visible) {
      localMode.value = props.settings?.mode ?? 'custom'
      const cfg = props.settings?.customConfig ?? {}
      localBaseDomain.value = cfg.baseDomain || props.settings?.customDomain || 'ccwu.cc'
      localTunnelId.value = cfg.tunnelId || ''
      localCredentialsJson.value = cfg.credentialsJson || props.settings?.token || ''
    }
  },
  { immediate: true }
)

const handleCredentialsInput = () => {
  const content = localCredentialsJson.value.trim()
  if (!content) return

  // 1. 尝试直接作为 JSON 解析
  try {
    const parsed = JSON.parse(content)
    if (parsed.TunnelID && !localTunnelId.value) {
      localTunnelId.value = String(parsed.TunnelID).trim()
    } else if (parsed.t && !localTunnelId.value) {
      localTunnelId.value = String(parsed.t).trim()
    }
    return
  } catch (err) {}

  // 2. 尝试作为 Cloudflare Base64 Token (以 eyJh 开头) 自动解码
  try {
    const decodedStr = atob(content)
    const parsed = JSON.parse(decodedStr)
    if (parsed.t) {
      if (!localTunnelId.value) {
        localTunnelId.value = String(parsed.t).trim()
      }
      // 自动转为标准 Credentials JSON 格式
      const standardJson = {
        AccountTag: parsed.a || '',
        TunnelSecret: parsed.s || '',
        TunnelID: parsed.t || ''
      }
      localCredentialsJson.value = JSON.stringify(standardJson, null, 2)
    }
  } catch (err) {}
}

const handleSave = () => {
  let cleanDomain = localBaseDomain.value
    .replace(/^https?:\/\//i, '')
    .replace(/\/.*$/, '')
    .trim()

  if (!cleanDomain) {
    cleanDomain = 'du1.ccwu.cc'
  }

  let finalTunnelId = localTunnelId.value.trim()
  let credsContent = localCredentialsJson.value.trim()
  let finalToken = props.settings?.token || props.settings?.customConfig?.token || ''

  if (credsContent) {
    if (credsContent.startsWith('eyJh')) {
      finalToken = credsContent
      try {
        const decoded = JSON.parse(atob(credsContent))
        if (decoded.t) {
          finalTunnelId = String(decoded.t).trim()
        }
      } catch (e) {}
    } else {
      try {
        const parsed = JSON.parse(credsContent)
        const a = parsed.AccountTag || parsed.a || ''
        const t = parsed.TunnelID || parsed.t || finalTunnelId || ''
        const s = parsed.TunnelSecret || parsed.s || ''
        if (t) finalTunnelId = t
        if (a && t && s) {
          finalToken = btoa(JSON.stringify({ a, t, s }))
        }
      } catch (e) {
        if (!finalToken) finalToken = credsContent
      }
    }
  }

  emit('save', {
    mode: localMode.value,
    token: finalToken,
    customDomain: cleanDomain,
    customConfig: {
      tunnelId: finalTunnelId,
      credentialsJson: credsContent,
      baseDomain: cleanDomain,
      subdomainPattern: 'p{port}',
      token: finalToken
    }
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
