import { onBeforeUnmount, onMounted, type Ref } from 'vue'
import { fitDisplayMath } from '../fitmath'

export function useMathFit(rootEl: Ref<HTMLElement | null>) {
  let mo: MutationObserver | null = null
  let timer: number | undefined
  let disposed = false

  function scheduleFit() {
    if (disposed) return
    window.clearTimeout(timer)
    timer = window.setTimeout(() => fitDisplayMath(rootEl.value ?? document), 0)
  }

  onMounted(() => {
    scheduleFit()
    mo = new MutationObserver(scheduleFit)
    if (rootEl.value) mo.observe(rootEl.value, { childList: true, subtree: true })
    window.addEventListener('resize', scheduleFit)

    document.fonts?.ready.then(scheduleFit)
  })
  onBeforeUnmount(() => {
    disposed = true
    window.clearTimeout(timer)
    mo?.disconnect()
    window.removeEventListener('resize', scheduleFit)
  })
}
