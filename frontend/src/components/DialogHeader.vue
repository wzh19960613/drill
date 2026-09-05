<script setup lang="ts">
import { X } from 'lucide-vue-next'

/**
 * Unified dialog header: main title + subtitle + right-side actions + close button.
 * Responsive: on narrower windows the main and subtitle stack on two rows; on very narrow
 * ones the title centers, action buttons wrap and fill the row, and the close button pins
 * to the top-right. Traffic-light avoidance is handled globally by the html.pwa rules.
 * flush: the dialog's top edge touches the screen (full-screen dialogs) — in that case the
 * header padding includes the status-bar safe area; centered dialogs (default) do not touch
 * the screen, the safe distance is handled by the outer margins, and the header keeps no
 * large empty space.
 */
withDefaults(
  defineProps<{ title?: string; subtitle?: string; flush?: boolean; noBorder?: boolean }>(),
  {
    title: '',
    subtitle: '',
    flush: false,
    noBorder: false,
  },
)

const emit = defineEmits<{ (e: 'close'): void }>()
</script>

<template>
  <header
    class="dlg-head"
    :class="{ flush, 'no-border': noBorder, 'has-actions': !!$slots.actions }"
  >
    <div class="dlg-titles">
      <h3 class="dlg-title"><slot name="title">{{ title }}</slot></h3>
      <span v-if="subtitle || $slots.subtitle" class="dlg-sub">
        <slot name="subtitle">{{ subtitle }}</slot>
      </span>
    </div>
    <div v-if="$slots.actions" class="dlg-actions"><slot name="actions" /></div>
    <button class="icon-btn dlg-close" aria-label="关闭" @click="emit('close')">
      <X style="width: 1rem; height: 1rem" />
    </button>
  </header>
</template>

<style scoped>
.dlg-head {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 1.125rem;
  border-bottom:1px solid var(--line);
  flex: none;
}

.dlg-head.no-border {
  border-bottom: none;
}

@container (max-width: 62.4375rem) {
  .dlg-head.flush {
    padding-top: calc(1.125rem + env(safe-area-inset-top));
  }
}

.dlg-titles {
  display: flex;
  align-items: baseline;
  gap: 0.625rem;
  min-width: 0;
  flex: 1;
}

.dlg-title {
  margin: 0;
  font-size: 0.9375rem;
  font-weight: 700;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dlg-sub {
  color: var(--muted);
  font-size: 0.7812rem;
  font-weight: 400;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dlg-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: none;
}

.dlg-close,
.dlg-actions :deep(.icon-btn) {
  background: inherit;
}

.dlg-close:hover,
.dlg-actions :deep(.icon-btn):hover {
  background: var(--hover);
}

.dlg-close {
  flex: none;
}

@container (max-width: 47.5rem) {
  .dlg-titles {
    flex-direction: column;
    align-items: flex-start;
    gap:1px;
  }

  .dlg-titles :deep(*) {
    white-space: nowrap;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}

@container (max-width: 30rem) {
  .dlg-head {
    flex-wrap: wrap;
    row-gap: 0.5rem;
  }

  .dlg-titles {
    flex: 1 1 100%;
    align-items: center;
    justify-content: center;
    padding-right: 3rem;
  }

  .dlg-close {
    position: absolute;
    top: calc(0.5rem + env(safe-area-inset-top));
    right: 0.5rem;
  }

  .dlg-head:not(.flush) .dlg-close {
    top: 0.5rem;
  }

  /* With header actions the whole head stays on one row: title truncates on
     the left, icon actions and the close button keep their spot on the right */
  .dlg-head.has-actions {
    flex-wrap: nowrap;
  }

  .dlg-head.has-actions .dlg-titles {
    flex: 1 1 auto;
    align-items: flex-start;
    justify-content: flex-start;
    padding-right: 0;
  }

  .dlg-head.has-actions .dlg-actions {
    flex: 0 1 auto;
    margin-left: auto;
  }

  .dlg-head.has-actions .dlg-close {
    position: static;
  }

  .dlg-head:not(.has-actions) .dlg-actions {
    flex: 1 1 100%;
  }

  .dlg-head:not(.has-actions) .dlg-actions :deep(.btn) {
    flex: 1;
    justify-content: center;
  }
}
</style>
