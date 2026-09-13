import { onBeforeUnmount, onMounted } from 'vue'
import { lockBodyScroll, unlockBodyScroll } from '../scrollLock'

export function useDialogShell(onKey: (e: KeyboardEvent) => void, lockScroll = true) {
  const handler = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      const t = e.target as HTMLElement | null
      if (t && (t.tagName === 'TEXTAREA' || t.tagName === 'INPUT' || t.isContentEditable)) return
    }
    onKey(e)
  }
  onMounted(() => {
    window.addEventListener('keydown', handler, true)
    if (lockScroll) lockBodyScroll()
  })
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', handler, true)
    if (lockScroll) unlockBodyScroll()
  })
}
