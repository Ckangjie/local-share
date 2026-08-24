import { ref } from 'vue'

const STORAGE_KEY = 'localshare_tunnel_settings'

// 默认配置
const defaultSettings = {
  mode: 'custom', // 'quick' (随机临时域名) | 'custom' (固定域名 Token 模式)
  token: '',
  customDomain: 'du1.ccwu.cc'
}

function loadStoredSettings() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...defaultSettings }
    const parsed = JSON.parse(raw)
    return {
      mode: parsed.mode ?? defaultSettings.mode,
      token: parsed.token ?? defaultSettings.token,
      customDomain: parsed.customDomain ?? defaultSettings.customDomain
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
