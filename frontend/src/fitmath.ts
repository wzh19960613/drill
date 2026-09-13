export function fitDisplayMath(root: ParentNode = document, opts?: { forceFit?: boolean }) {
  root.querySelectorAll<HTMLElement>('.q-math').forEach((box) => {
    if (opts?.forceFit) box.dataset.mathMode = 'fit'
    if (!box.dataset.mathInit) {
      box.dataset.mathInit = '1'
      wirePanAndZoom(box)
    }
    apply(box)
  })
}

function wirePanAndZoom(box: HTMLElement) {
  let panned = false
  box.addEventListener('pointerdown', (e) => {

    if (
      (e as PointerEvent).pointerType !== 'mouse' ||
      (e as PointerEvent).button !== 0 ||
      box.dataset.mathMode !== 'scroll'
    ) {
      return
    }
    const startX = (e as PointerEvent).clientX
    const startScroll = box.scrollLeft
    const onMove = (ev: PointerEvent) => {
      const dx = ev.clientX - startX
      if (!panned && Math.abs(dx) < 4) return
      panned = true
      box.dataset.mathPanning = '1'
      box.scrollLeft = startScroll - dx
    }
    const onUp = () => {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerup', onUp)
      if (panned) {

        window.setTimeout(() => {
          panned = false
          delete box.dataset.mathPanning
        }, 0)
      }
    }
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)

    e.preventDefault()
  })
  box.addEventListener('click', () => {
    if (panned) return
    if (!box.dataset.mathWide) return
    box.dataset.mathMode = box.dataset.mathMode === 'scroll' ? 'fit' : 'scroll'
    apply(box)
  })
}

function apply(box: HTMLElement) {

  const disps = [...box.children].filter(
    (el): el is HTMLElement =>
      el instanceof HTMLElement && el.classList.contains('katex-display'),
  )
  if (!disps.length) return
  const avail = box.clientWidth
  if (avail <= 0) return
  const scroll = box.dataset.mathMode === 'scroll'
  let anyZoomable = false
  for (const disp of disps) {
    const overflow = disp.offsetWidth - avail

    const zoomable = overflow > Math.max(12, avail * 0.04)
    if (zoomable) anyZoomable = true

    if (scroll || (overflow <= 0 && !zoomable)) {
      disp.style.transform = ''
      disp.style.marginBottom = ''
      continue
    }
    const k = avail / disp.offsetWidth
    disp.style.transformOrigin = 'top left'
    disp.style.transform = `scale(${k})`
    disp.style.marginBottom = `-${Math.round(disp.offsetHeight * (1 - k))}px`
    box.dataset.mathReady = '1'
  }
  if (!anyZoomable) {

    box.removeAttribute('data-math-wide')
    box.dataset.mathMode = 'fit'
    box.title = ''
    return
  }
  box.dataset.mathWide = '1'
  box.title = '点击切换：原始大小（可横向滚动）/ 适应宽度'
}
