import { ref, computed } from 'vue'

/**
 * 桌面端调用系统底层网络扫描
 */
async function callScanWebServices() {
  if (window.__TAURI_INTERNALS__ || window.__TAURI__) {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke('scan_web_services')
  }

  throw new Error('未检测到桌面端运行环境')
}

export function useServices() {
  const services = ref([])
  const isScanning = ref(false)
  const selectedPort = ref(null)
  const scanError = ref('')

  const hasServices = computed(() => services.value.length > 0)
  const selectedService = computed(() =>
    services.value.find((item) => item.port === selectedPort.value) ?? null
  )

  /**
   * 扫描本地正在运行的 Web 服务
   */
  const scanServices = async () => {
    if (isScanning.value) return

    isScanning.value = true
    scanError.value = ''

    try {
      const list = await callScanWebServices()
      services.value = Array.isArray(list) ? list : []

      const isCurrentSelectedValid = services.value.some(
        (item) => item.port === selectedPort.value
      )
      if (!isCurrentSelectedValid && services.value.length > 0) {
        selectedPort.value = services.value[0]?.port ?? null
      }
    } catch (error) {
      console.error('扫描本地服务失败:', error)
      scanError.value = error?.message ?? String(error) ?? '扫描本地服务失败'
    } finally {
      isScanning.value = false
    }
  }

  /**
   * 选中指定端口
   * @param {number} port
   */
  const selectPort = (port) => {
    selectedPort.value = port
  }

  return {
    services,
    isScanning,
    selectedPort,
    scanError,
    hasServices,
    selectedService,
    scanServices,
    selectPort
  }
}
