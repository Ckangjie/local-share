/**
 * 格式化秒数为 HH:mm:ss
 * @param {number} totalSeconds
 * @returns {string}
 */
export function formatDuration(totalSeconds = 0) {
  const safeSeconds = Math.max(0, Math.floor(totalSeconds ?? 0))
  const hours = Math.floor(safeSeconds / 3600)
  const minutes = Math.floor((safeSeconds % 3600) / 60)
  const seconds = safeSeconds % 60

  const pad = (num) => String(num).padStart(2, '0')
  return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`
}

/**
 * 校验端口是否在有效合法范围 1 ~ 65535
 * @param {number|string} port
 * @returns {boolean}
 */
export function isValidPort(port) {
  const numericPort = Number(port)
  return Number.isInteger(numericPort) && numericPort >= 1 && numericPort <= 65535
}
