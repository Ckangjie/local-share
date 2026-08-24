<template>
  <div v-if="message" class="error-notice">
    <div class="icon">⚠️</div>
    <div class="content">
      <p class="msg">{{ message }}</p>
    </div>
    <div class="actions">
      <button class="btn-retry" @click="emit('retry')">重试</button>
      <button class="btn-close" @click="emit('close')">✕</button>
    </div>
  </div>
</template>

<script setup>
defineProps({
  message: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['close', 'retry'])
</script>

<style lang="scss" scoped>
@use '@/assets/styles/variables.scss' as *;

.error-notice {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: $radius-md;
}

.icon {
  font-size: 14px;
}

.content {
  flex: 1;
  min-width: 0;

  .msg {
    font-size: 12px;
    color: $color-danger;
    line-height: 1.4;
    word-break: break-all;
  }
}

.actions {
  display: flex;
  align-items: center;
  gap: 6px;

  .btn-retry {
    font-size: 11px;
    padding: 2px 6px;
    background: $color-danger;
    color: #fff;
    border-radius: $radius-sm;
    font-weight: 500;
  }

  .btn-close {
    font-size: 12px;
    color: $color-text-muted;
    padding: 2px 4px;

    &:hover {
      color: $color-text-primary;
    }
  }
}
</style>
