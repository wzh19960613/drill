// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'
import { fitDisplayMath } from './fitmath'

function makeMath(avail: number, dispWidth: number, dispHeight = 40) {
  const box = document.createElement('div')
  box.className = 'q-math'
  const disp = document.createElement('div')
  disp.className = 'katex-display'
  Object.defineProperty(box, 'clientWidth', { value: avail, configurable: true })
  Object.defineProperty(disp, 'offsetWidth', { value: dispWidth, configurable: true })
  Object.defineProperty(disp, 'offsetHeight', { value: dispHeight, configurable: true })
  box.appendChild(disp)
  document.body.appendChild(box)
  return { box, disp }
}

describe('fitDisplayMath 的放大光标与切换', () => {
  it('不超宽：不残留 data-math-wide 属性（空串也会命中 CSS 的 zoom-in 光标）', () => {
    const { box } = makeMath(300, 280)
    fitDisplayMath()
    expect(box.hasAttribute('data-math-wide')).toBe(false)
    expect(box.dataset.mathMode).toBe('fit')
    expect(box.title).toBe('')
  })

  it('轻微超宽（未到可感知阈值）：视觉适配缩放，但不提供放大切换', () => {
    const { box, disp } = makeMath(300, 308)
    fitDisplayMath()
    expect(box.hasAttribute('data-math-wide')).toBe(false)
    expect(disp.style.transform).toContain('scale(')
    box.click()
    expect(box.dataset.mathMode).toBe('fit')
  })

  it('明显超宽：标记 wide，点击可在 fit/scroll 间切换', () => {
    const { box } = makeMath(300, 500)
    fitDisplayMath()
    expect(box.dataset.mathWide).toBe('1')

    box.click()
    expect(box.dataset.mathMode).toBe('scroll')
    box.click()
    expect(box.dataset.mathMode).toBe('fit')
  })

  it('scroll 态下容器变宽到放得下：自动回 fit 并清除 wide 标记', () => {
    const { box, disp } = makeMath(300, 500)
    fitDisplayMath()
    box.click()
    expect(box.dataset.mathMode).toBe('scroll')
    Object.defineProperty(box, 'clientWidth', { value: 600 })
    fitDisplayMath()
    expect(box.dataset.mathMode).toBe('fit')
    expect(box.hasAttribute('data-math-wide')).toBe(false)
    expect(disp.style.transform).toBe('')
  })

  function fire(el: EventTarget, type: string, props: { clientX?: number; pointerType?: string } = {}) {
    const e = new MouseEvent(type, {
      bubbles: true,
      cancelable: true,
      clientX: props.clientX ?? 0,
      button: 0,
    })
    Object.defineProperty(e, 'pointerType', { value: props.pointerType ?? 'mouse' })
    el.dispatchEvent(e)
  }

  it('scroll 模式下鼠标左键拖动可平移，拖完的 click 不触发切换', async () => {
    const { box } = makeMath(300, 500)
    fitDisplayMath()
    box.click()
    expect(box.dataset.mathMode).toBe('scroll')

    fire(box, 'pointerdown', { clientX: 100 })
    fire(window, 'pointermove', { clientX: 60 })
    expect(box.scrollLeft).toBe(40)
    expect(box.hasAttribute('data-math-panning')).toBe(true)
    fire(window, 'pointerup')

    box.click()
    expect(box.dataset.mathMode).toBe('scroll')

    await new Promise((r) => setTimeout(r, 0))
    box.click()
    expect(box.dataset.mathMode).toBe('fit')
  })

  it('触屏 pointer 不进入鼠标拖动逻辑（原生滚动处理）', () => {
    const { box } = makeMath(300, 500)
    fitDisplayMath()
    box.click()
    fire(box, 'pointerdown', { clientX: 100, pointerType: 'touch' })
    fire(window, 'pointermove', { clientX: 40, pointerType: 'touch' })
    expect(box.scrollLeft).toBe(0)
    expect(box.hasAttribute('data-math-panning')).toBe(false)
    fire(window, 'pointerup')
    expect(box.dataset.mathMode).toBe('scroll')
  })

  it('合并容器：每个公式各自缩放，容器统一标记 wide/切换', () => {
    const box = document.createElement('div')
    box.className = 'q-math'
    const d1 = document.createElement('div')
    const d2 = document.createElement('div')
    d1.className = 'katex-display'
    d2.className = 'katex-display'
    Object.defineProperty(box, 'clientWidth', { value: 300, configurable: true })
    Object.defineProperty(d1, 'offsetWidth', { value: 200, configurable: true })
    Object.defineProperty(d1, 'offsetHeight', { value: 40, configurable: true })
    Object.defineProperty(d2, 'offsetWidth', { value: 500, configurable: true })
    Object.defineProperty(d2, 'offsetHeight', { value: 40, configurable: true })
    box.append(d1, d2)
    document.body.appendChild(box)
    fitDisplayMath()

    expect(d1.style.transform).toBe('')
    expect(d2.style.transform).toContain('scale(0.6')
    expect(box.dataset.mathWide).toBe('1')
    box.click()
    expect(box.dataset.mathMode).toBe('scroll')
    expect(d2.style.transform).toBe('')
  })
})
