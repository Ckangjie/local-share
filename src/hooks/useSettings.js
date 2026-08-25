import { ref } from 'vue'

const STORAGE_KEY = 'localshare_tunnel_settings'

// 默认配置
const defaultSettings = {
  mode: 'custom', // 'quick' (随机临时域名) | 'custom' (固定域名多服务模式)
  token: '', // 兼容历史单 Token
  customDomain: 'du1.ccwu.cc',
  customConfig: {
    tunnelId: '',
    credentialsJson: '',
    baseDomain: 'du1.ccwu.cc',
    subdomainPattern: 'p{port}',
    token: ''
  }
}

function loadStoredSettings() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...defaultSettings }
    const parsed = JSON.parse(raw)
    const storedCustomConfig = parsed.customConfig ?? {}
    const effectiveToken = parsed.token || storedCustomConfig.token || ''
    const effectiveDomain = storedCustomConfig.baseDomain || parsed.customDomain || 'du1.ccwu.cc'
    return {
      mode: parsed.mode ?? defaultSettings.mode,
      token: effectiveToken,
      customDomain: effectiveDomain,
      customConfig: {
        tunnelId: storedCustomConfig.tunnelId ?? '',
        credentialsJson: storedCustomConfig.credentialsJson ?? '',
        baseDomain: effectiveDomain,
        subdomainPattern: storedCustomConfig.subdomainPattern ?? 'p{port}',
        token: effectiveToken
      }
    }
  } catch (err) {
    console.warn('读取本地配置失败，恢复默认值:', err)
    return { ...defaultSettings }
  }
}

// 全局单例响应式状态
const settingsState = ref(loadStoredSettings())
const isSettingsModalVisible = ref(false)

export function useSettings() {
  const openSettings = () => {
    isSettingsModalVisible.value = true
  }

  const closeSettings = () => {
    isSettingsModalVisible.value = false
  }

  const updateSettings = (partial) => {
    settingsState.value = {
      ...settingsState.value,
      ...partial
    }
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settingsState.value))
    } catch (err) {
      console.error('持久化配置失败:', err)
    }
  }

  return {
    settings: settingsState,
    isSettingsModalVisible,
    openSettings,
    closeSettings,
    updateSettings
  }
}
