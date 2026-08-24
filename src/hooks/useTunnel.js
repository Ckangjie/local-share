import { ref, computed, onMounted, onUnmounted } from 'vue'
import { copyToClipboard } from '@/utils/clipboard.js'
import { useSettings } from '@/hooks/useSettings.js'

export function useTunnel() {
  const { settings, openSettings } = useSettings()

  // activeTunnels 结构: Record<number, { port, publicUrl, status: 'starting'|'running'|'stopping'|'error', runningSeconds, errorMessage, isCopied, timerId }>
  const activeTunnels = ref({})

  // 引擎按需下载状态
  const isEngineModalVisible = ref(false)
  const isDownloadingEngine = ref(false)
  const engineDownloadPercent = ref(0)
  const engineDownloadedBytes = ref(0)
  const engineTotalBytes = ref(0)
  const engineDownloadError = ref('')
  const pendingSharePort = ref(null)

  const activePorts = computed(() => Object.keys(activeTunnels.value).map(Number))
  const activeCount = computed(() => {
    return Object.values(activeTunnels.value).filter(
      (t) => t.status === 'running' || t.status === 'starting'
    ).length
  })
  const hasActiveTunnels = computed(() => activeCount.value > 0)
  const isAnyStarting = computed(() =>
    Object.values(activeTunnels.value).some((t) => t.status === 'starting')
  )

  let unlistenTunnelClosed = null
  let unlistenEngineProgress = null

  /**
   * 检查本地是否已具备穿透引擎
   */
  const checkEngineReady = async () => {
    if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const isReady = await invoke('check_engine_status')
        return Boolean(isReady)
      } catch (err) {
        console.warn('检查引擎状态失败:', err)
        return true
      }
    }
    return true
  }

  /**
   * 触发下载穿透引擎
   */
  const triggerDownloadEngine = async () => {
    isDownloadingEngine.value = true
    engineDownloadError.value = ''
    engineDownloadPercent.value = 0

    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('download_engine')
      isDownloadingEngine.value = false
      isEngineModalVisible.value = false

      // 下载完成后自动恢复挂起的分享端口
      if (pendingSharePort.value) {
        const portToResume = pendingSharePort.value
        pendingSharePort.value = null
        await startShare(portToResume)
      }
    } catch (err) {
      console.error('下载穿透引擎失败:', err)
      isDownloadingEngine.value = false
      engineDownloadError.value = err?.message ?? String(err) ?? '下载失败，请检查网络后重试'
    }
  }

  /**
   * 取消下载引擎
   */
  const cancelEngineDownload = () => {
    isEngineModalVisible.value = false
    pendingSharePort.value = null
    engineDownloadError.value = ''
  }

  /**
   * 启动指定端口的独立分享
   * @param {number|string} rawPort
   */
  const startShare = async (rawPort) => {
    const port = Number(rawPort)
    if (!port) return

    const existing = activeTunnels.value[port]
    if (existing && (existing.status === 'starting' || existing.status === 'running')) {
      return
    }

    // 1. 优先检查引擎是否就绪
    const isEngineReady = await checkEngineReady()
    if (!isEngineReady) {
      pendingSharePort.value = port
      isEngineModalVisible.value = true
      return
    }

    // 2. 校验配置（固定域名模式需要 Token）
    const isCustomMode = settings.value?.mode === 'custom'
    const token = isCustomMode ? settings.value?.token?.trim() : ''
    const customDomain = isCustomMode ? (settings.value?.customDomain?.trim() || 'du1.ccwu.cc') : ''

    if (isCustomMode && !token) {
      openSettings()
      throw new Error('当前为固定域名模式，请先在设置中填入 Cloudflare Tunnel Token')
    }

    // 3. 初始化/重置该端口状态
    activeTunnels.value[port] = {
      port,
      publicUrl: '',
      status: 'starting',
      runningSeconds: 0,
      errorMessage: '',
      isCopied: false,
      timerId: null
    }

    try {
      let generatedUrl = ''

      if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
        const { invoke } = await import('@tauri-apps/api/core')
        generatedUrl = await invoke('start_tunnel', {
          port,
          token: token || null,
          customDomain: customDomain || null
        })
      } else {
        throw new Error('未检测到桌面端运行环境')
      }

      if (!generatedUrl) {
        throw new Error('未获取到有效的公网访问地址')
      }

      const tunnel = activeTunnels.value[port]
      if (tunnel) {
        tunnel.publicUrl = generatedUrl
        tunnel.status = 'running'
        tunnel.errorMessage = ''

        // 启动该端口专属秒级计时器
        if (tunnel.timerId) clearInterval(tunnel.timerId)
        tunnel.timerId = setInterval(() => {
          if (activeTunnels.value[port]) {
            activeTunnels.value[port].runningSeconds += 1
          }
        }, 1000)
      }
    } catch (error) {
      console.error(`端口 :${port} 启动分享失败:`, error)
      const tunnel = activeTunnels.value[port]
      if (tunnel) {
        tunnel.status = 'error'
        tunnel.errorMessage = error?.message ?? String(error) ?? '启动失败'
        if (tunnel.timerId) {
          clearInterval(tunnel.timerId)
          tunnel.timerId = null
        }
      }
    }
  }

  /**
   * 停止指定端口的分享
   * @param {number|string} rawPort
   */
  const stopShare = async (rawPort) => {
    const port = Number(rawPort)
    if (!port || !activeTunnels.value[port]) return

    const tunnel = activeTunnels.value[port]
    tunnel.status = 'stopping'
    if (tunnel.timerId) {
      clearInterval(tunnel.timerId)
      tunnel.timerId = null
    }

    try {
      if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('stop_tunnel', { port })
      }
    } catch (error) {
      console.warn(`停止端口 :${port} 失败:`, error)
    } finally {
      delete activeTunnels.value[port]
    }
  }

  /**
   * 停止全部正在运行的分享
   */
  const stopAll = async () => {
    Object.values(activeTunnels.value).forEach((tunnel) => {
      if (tunnel.timerId) {
        clearInterval(tunnel.timerId)
        tunnel.timerId = null
      }
      tunnel.status = 'stopping'
    })

    try {
      if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('stop_all_tunnels')
      }
    } catch (error) {
      console.warn('停止全部隧道失败:', error)
    } finally {
      activeTunnels.value = {}
    }
  }

  /**
   * 复制指定端口公网地址
   * @param {number|string} rawPort
   */
  const copyPublicUrl = async (rawPort) => {
    const port = Number(rawPort)
    const tunnel = activeTunnels.value[port]
    if (!tunnel?.publicUrl) return false

    const isSuccess = await copyToClipboard(tunnel.publicUrl)
    if (isSuccess) {
      tunnel.isCopied = true
      setTimeout(() => {
        if (activeTunnels.value[port]) {
          activeTunnels.value[port].isCopied = false
        }
      }, 2000)
    }
    return isSuccess
  }

  /**
   * 在默认系统浏览器打开指定 URL
   * @param {string} url
   */
  const openInBrowser = async (url) => {
    if (!url) return
    try {
      if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
        const { open } = await import('@tauri-apps/plugin-shell')
        await open(url)
      } else {
        window.open(url, '_blank')
      }
    } catch (err) {
      window.open(url, '_blank')
    }
  }

  /**
   * 移除/重置指定端口的异常卡片
   * @param {number|string} rawPort
   */
  const dismissTunnel = (rawPort) => {
    const port = Number(rawPort)
    if (activeTunnels.value[port]) {
      if (activeTunnels.value[port].timerId) {
        clearInterval(activeTunnels.value[port].timerId)
      }
      delete activeTunnels.value[port]
    }
  }

  onMounted(async () => {
    // Tauri 2 事件监听
    if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
      try {
        const { listen } = await import('@tauri-apps/api/event')

        unlistenTunnelClosed = await listen('tunnel-closed', (event) => {
          const port = Number(event.payload)
          const tunnel = activeTunnels.value[port]
          if (tunnel && tunnel.status === 'running') {
            tunnel.status = 'error'
            tunnel.errorMessage = 'Tunnel 连接已断开，请检查网络或重新分享'
            if (tunnel.timerId) {
              clearInterval(tunnel.timerId)
              tunnel.timerId = null
            }
          }
        })

        unlistenEngineProgress = await listen('engine-download-progress', (event) => {
          const data = event.payload
          if (data) {
            engineDownloadedBytes.value = data.downloaded || 0
            engineTotalBytes.value = data.total || 0
            engineDownloadPercent.value = Math.min(100, Math.max(0, data.percent || 0))
          }
        })
      } catch (err) {
        console.warn('注册 Tauri 事件监听器失败:', err)
      }
    }
  })

  onUnmounted(() => {
    Object.values(activeTunnels.value).forEach((tunnel) => {
      if (tunnel.timerId) {
        clearInterval(tunnel.timerId)
      }
    })
    if (typeof unlistenTunnelClosed === 'function') unlistenTunnelClosed()
    if (typeof unlistenEngineProgress === 'function') unlistenEngineProgress()
  })

  return {
    activeTunnels,
    activePorts,
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
  }
}

