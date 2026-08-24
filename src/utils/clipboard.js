/**
 * 复制文本到系统剪贴板
 * @param {string} text
 * @returns {Promise<boolean>}
 */
export async function copyToClipboard(text) {
  if (!text) return false

  try {
    if (navigator?.clipboard?.writeText) {
      await navigator.clipboard.writeText(text)
      return true
    }

    // 兼容回退方案
    const textarea = document.createElement('textarea')
    textarea.value = text
    textarea.style.position = 'fixed'
    textarea.style.opacity = '0'
    document.body.appendChild(textarea)
    textarea.select()
    const isSuccess = document.execCommand('copy')
    document.body.removeChild(textarea)
    return isSuccess
  } catch (error) {
    console.error('复制到剪贴板失败:', error)
    return false
  }
}
