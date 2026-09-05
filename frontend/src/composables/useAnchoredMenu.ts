import { ref } from 'vue'

/**
 * Menu anchored to the button that toggled it (x/y for MenuPop). The 4px gap
 * is optical, so px is allowed.
 */
export function useAnchoredMenu(edge: 'left' | 'right' = 'right') {
  const open = ref(false)
  const x = ref(0)
  const y = ref(0)

  function toggle(ev: MouseEvent) {
    if (open.value) {
      open.value = false
      return
    }
    const rect = (ev.currentTarget as HTMLElement | null)?.getBoundingClientRect()
    if (!rect) return
    x.value = edge === 'right' ? rect.right : rect.left
    y.value = rect.bottom + 4
    open.value = true
  }

  function close() {
    open.value = false
  }

  return { open, x, y, toggle, close }
}
