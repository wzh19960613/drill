export function fitDisplayMath(root: ParentNode = document, opts?: { forceFit?: boolean }) {
  root.querySelectorAll<HTMLElement>('.q-math').forEach((box) => {
    if (opts?.forceFit) box.dataset.mathMode = 'fit'
    if (!box.dataset.mathInit) {
      box.dataset.mathInit = '1'
      box.addEventListener('click', () => {
        if (!box.dataset.mathWide) return
        box.dataset.mathMode = box.dataset.mathMode === 'scroll' ? 'fit' : 'scroll'
        apply(box)
      })
    }
    apply(box)
  })
}

function apply(box: HTMLElement) {
  const disp = box.firstElementChild as HTMLElement | null
  if (!disp?.classList.contains('katex-display')) return
  const avail = box.clientWidth
  if (avail <= 0) return
  const wide = disp.offsetWidth > avail + 1
  if (!wide) {
    box.dataset.mathWide = ''
    box.dataset.mathMode = 'fit'
    box.title = ''
    disp.style.transform = ''
    disp.style.marginBottom = ''
    return
  }
  box.dataset.mathWide = '1'
  box.title = '点击切换：原始大小（可横向滚动）/ 适应宽度'
  if (box.dataset.mathMode === 'scroll') {
    disp.style.transform = ''
    disp.style.marginBottom = ''
    return
  }
  const k = avail / disp.offsetWidth
  disp.style.transformOrigin = 'top left'
  disp.style.transform = `scale(${k})`
  disp.style.marginBottom = `-${Math.round(disp.offsetHeight * (1 - k))}px`
  box.dataset.mathReady = '1'
}
