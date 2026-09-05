import { onBeforeUnmount, onMounted } from 'vue'
import { lockBodyScroll, unlockBodyScroll } from '../scrollLock'

/**
 * Dialog lifecycle boilerplate: a capture-phase keydown handler on window plus
 * the reference-counted page-scroll lock, both torn down on unmount.
 *
 * Capture listeners on window fire in registration order, so a dialog opened
 * later (nested inside another) receives keys *after* its parent — the parent
 * must dispatch (or ask the child via an exposed handle) instead of both
 * acting on the same Escape.
 */
export function useDialogShell(onKey: (e: KeyboardEvent) => void, lockScroll = true) {
  onMounted(() => {
    window.addEventListener('keydown', onKey, true)
    if (lockScroll) lockBodyScroll()
  })
  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKey, true)
    if (lockScroll) unlockBodyScroll()
  })
}
