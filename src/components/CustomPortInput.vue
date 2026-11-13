<template>
  <div class="custom-port-container">
    <div class="input-group">
      <span class="prefix">http://localhost:</span>
      <input
        v-model="inputVal"
        type="number"
        min="1"
        max="65535"
        placeholder="输入本地端口 (例如 5173、3000、8080)"
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
</template>

<script setup>
import { ref, computed } from 'vue'
import { isValidPort } from '@/utils/format.js'

const emit = defineEmits(['share'])

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
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.input-group {
  display: flex;
  align-items: center;
  gap: 8px;
  background: $color-bg-input;
  border: 1px solid $color-border-subtle;
  border-radius: $radius-sm;
  padding: 6px 10px;
  transition: border-color $transition-fast;

  &:focus-within {
    border-color: $color-primary;
    box-shadow: 0 0 0 1px $color-primary;
  }

  .prefix {
    font-size: 13px;
    color: $color-text-muted;
    font-family: monospace;
    white-space: nowrap;
    user-select: none;
  }

  .port-input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    padding: 2px 0;
    font-size: 13px;
    color: $color-text-primary;
    font-family: monospace;

    &::-webkit-inner-spin-button,
    &::-webkit-outer-spin-button {
      -webkit-appearance: none;
      margin: 0;
    }

    &:focus {
      outline: none;
    }
  }

  .apply-btn {
    font-size: 12px;
    padding: 5px 12px;
    background: $color-primary;
    color: #fff;
    border-radius: $radius-sm;
    font-weight: 500;
    white-space: nowrap;
    transition: background $transition-fast;

    &:hover:not(:disabled) {
      background: $color-primary-hover;
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }
}

.error-tip {
  font-size: 11px;
  color: $color-danger;
  padding-left: 2px;
}
</style>
