<template>
  <div class="custom-port-container">
    <div class="header-toggle" @click="isExpanded = !isExpanded">
      <span class="toggle-text">手动分享其他端口</span>
      <span :class="['arrow', { 'is-expanded': isExpanded }]">▼</span>
    </div>

    <div v-show="isExpanded" class="input-body">
      <div class="input-group">
        <span class="prefix">http://localhost:</span>
        <input
          v-model="inputVal"
          type="number"
          min="1"
          max="65535"
          placeholder="例如 8080"
          class="port-input"
          @input="handleInput"
          @keyup.enter="handleShare"
        />
        <button 
          class="apply-btn" 
          :disabled="!canApply" 
          @click="handleShare"
        >
          ⚡ 开始分享
        </button>
      </div>
      <p v-if="errorText" class="error-tip">{{ errorText }}</p>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { isValidPort } from '@/utils/format.js'

const emit = defineEmits(['share'])

const isExpanded = ref(false)
const inputVal = ref('')
const errorText = ref('')

const canApply = computed(() => {
  if (!inputVal.value) return false
  return isValidPort(inputVal.value)
})

const handleInput = () => {
  errorText.value = ''
}

const handleShare = () => {
  if (!isValidPort(inputVal.value)) {
    errorText.value = '请输入 1 ~ 65535 之间的有效端口号'
    return
  }

  const numericPort = Number(inputVal.value)
  emit('share', numericPort)
  inputVal.value = ''
  errorText.value = ''
}
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables.scss' as *;

.custom-port-container {
  background: $color-bg-card;
  border: 1px solid $color-border-subtle;
  border-radius: $radius-md;
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.header-toggle {
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  user-select: none;

  .toggle-text {
    font-size: 12px;
    color: $color-text-secondary;
    font-weight: 500;
  }

  .arrow {
    font-size: 10px;
    color: $color-text-muted;
    transition: transform $transition-fast;

    &.is-expanded {
      transform: rotate(180deg);
    }
  }

  &:hover .toggle-text {
    color: $color-text-primary;
  }
}

.input-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-top: 4px;
}

.input-group {
  display: flex;
  align-items: center;
  gap: 6px;
  background: $color-bg-input;
  border: 1px solid $color-border-subtle;
  border-radius: $radius-sm;
  padding: 4px 8px;

  .prefix {
    font-size: 12px;
    color: $color-text-muted;
    font-family: monospace;
    white-space: nowrap;
  }

  .port-input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    padding: 4px 0;
    font-size: 13px;
    color: $color-text-primary;
    font-family: monospace;

    &::-webkit-inner-spin-button,
    &::-webkit-outer-spin-button {
      -webkit-appearance: none;
      margin: 0;
    }
  }

  .apply-btn {
    font-size: 12px;
    padding: 4px 10px;
    background: $color-primary;
    color: #fff;
    border-radius: $radius-sm;
    font-weight: 500;

    &:hover:not(:disabled) {
      background: $color-primary-hover;
    }
  }
}

.error-tip {
  font-size: 11px;
  color: $color-danger;
}
</style>
