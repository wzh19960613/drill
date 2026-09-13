<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RotateCcw, X } from 'lucide-vue-next'
import { closeImageViewer, useImageViewerState } from '../composables/useImageViewer'


const target = useImageViewerState()
const src = computed(() => target.value?.url)
const open = computed(() => !!target.value)


const scale = ref(1)

const pullShrink = ref(1)
const fit = ref(1)
const tx = ref(0)
const ty = ref(0)
const stageEl = ref<HTMLElement | null>(null)
const imgEl = ref<HTMLImageElement | null>(null)

const animating = ref(false)

const closing = ref(false)

const bgOpacity = ref(1)

const dragging = ref(false)

const ready = ref(false)

const failed = ref(false)

let pullProgress = 0

let pullPassed = false

let dragMoved = false
const PULL_THRESHOLD = 165

const displayScale = computed(() => fit.value * scale.value * pullShrink.value)


const altered = computed(() => open.value && (scale.value > 1.01 || scale.value < 0.99))

const pointers = new Map<number, { x: number; y: number }>()
let pinch: { dist: number; scale: number } | null = null
let drag: { x: number; y: number; tx: number; ty: number } | null = null

function reset() {
  scale.value = 1
  pullShrink.value = 1
  tx.value = 0
  ty.value = 0
  bgOpacity.value = 1
  pullProgress = 0
  pullPassed = false
  dragging.value = false
  pointers.clear()
  pinch = null
  drag = null
}

watch(open, (v) => {
  if (v) {
    reset()
    ready.value = false
    failed.value = false
    animating.value = false
  }
})


function computeFit() {
  const stage = stageEl.value
  const img = imgEl.value
  if (!stage || !img || !img.naturalWidth || !img.naturalHeight) return
  const w = Math.max(50, stage.offsetWidth - 64)
  const h = Math.max(50, stage.offsetHeight - 112)
  fit.value = Math.min(w / img.naturalWidth, h / img.naturalHeight)
}

function clampScale(v: number) {
  return Math.min(12, Math.max(0.5, v))
}


function zoomAt(cx: number, cy: number, nextMult: number) {
  const el = stageEl.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  const ux = cx - rect.left - rect.width / 2
  const uy = cy - rect.top - rect.height / 2
  const next = clampScale(nextMult)
  const f = next / scale.value
  tx.value = ux - f * (ux - tx.value)
  ty.value = uy - f * (uy - ty.value)
  scale.value = next
}

function dist() {
  const [a, b] = [...pointers.values()]
  if (!a || !b) return 0
  return Math.hypot(a.x - b.x, a.y - b.y)
}

function mid() {
  const [a, b] = [...pointers.values()]
  if (!a || !b) return null
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }
}

function onWheel(e: WheelEvent) {
  if (!open.value) return
  e.preventDefault()
  animating.value = false
  zoomAt(e.clientX, e.clientY, scale.value * Math.exp(-e.deltaY * 0.0036))
}

function onPointerDown(e: PointerEvent) {
  const stage = stageEl.value
  if (!stage) return
  pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
  dragging.value = true
  animating.value = false
  
  
  try {
    stage.setPointerCapture(e.pointerId)
  } catch {
    
  }
  if (pointers.size === 2) {
    const d = dist()
    if (d > 0) pinch = { dist: d, scale: scale.value }
    drag = null
  } else if (pointers.size === 1) {
    drag = { x: e.clientX, y: e.clientY, tx: tx.value, ty: ty.value }
  }
}

function onPointerMove(e: PointerEvent) {
  if (!pointers.has(e.pointerId)) return
  pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
  if (pinch && pointers.size >= 2) {
    const d = dist()
    const m = mid()
    if (d > 0 && m && pinch.dist > 0) {
      zoomAt(m.x, m.y, pinch.scale * (d / pinch.dist))
    }
    return
  }
  if (drag && pointers.size === 1) {
    const dx = e.clientX - drag.x
    const dy = e.clientY - drag.y
    if (scale.value > 1.01) {
      
      tx.value = drag.tx + dx
      ty.value = drag.ty + dy
    } else {
      
      
      const DAMP = 0.3
      dragMoved = dx * dx + dy * dy > 4
      tx.value = drag.tx + dx * DAMP
      ty.value = drag.ty + (dy > 0 ? dy : dy * DAMP)
      const pull = Math.max(0, dy)
      pullPassed = pull > PULL_THRESHOLD
      pullProgress = pullPassed
        ? Math.min(1, (pull - PULL_THRESHOLD) / (PULL_THRESHOLD * 2))
        : 0
      
      pullShrink.value = 1 - 0.15 * pullProgress
      bgOpacity.value = 1 - 0.7 * pullProgress
    }
  }
}

function onPointerUp(e: PointerEvent) {
  pointers.delete(e.pointerId)
  if (pointers.size < 2) pinch = null
  if (pointers.size === 0) {
    dragging.value = false
    if (dragMoved && scale.value <= 1.01 && !pullPassed) {
      
      
      animating.value = true
      tx.value = 0
      ty.value = 0
      pullShrink.value = 1
      bgOpacity.value = 1
      dragMoved = false
    }
    if (pullProgress > 0 || pullPassed) {
      
      if (pullPassed) {
        dismissAnimated()
      } else {
        animating.value = true
        scale.value = 1
        tx.value = 0
        ty.value = 0
        bgOpacity.value = 1
      }
      pullProgress = 0
      pullPassed = false
      dragMoved = false
    } else if (scale.value < 1) {
      
      animating.value = true
      scale.value = 1
      tx.value = 0
      ty.value = 0
    }
    drag = null
  }
}


function dismissAnimated() {
  if (closing.value) return
  animating.value = true
  closing.value = true
  pullShrink.value = 1
  const img = imgEl.value
  const stage = stageEl.value
  const rect = target.value?.rect
  if (rect && img?.naturalWidth && stage) {
    
    const mult = rect.w / img.naturalWidth / fit.value
    const vw = stage.offsetWidth
    const vh = stage.offsetHeight
    scale.value = Math.max(0.05, mult)
    
    tx.value = rect.x + rect.w / 2 - vw / 2
    ty.value = rect.y + rect.h / 2 - vh / 2
  } else {
    scale.value = 1
    tx.value = 0
    ty.value = 0
  }
  bgOpacity.value = 0
  window.setTimeout(() => {
    closing.value = false
    closeImageViewer()
  }, 240)
}

function restore() {
  animating.value = true
  scale.value = 1
  pullShrink.value = 1
  tx.value = 0
  ty.value = 0
}

function onDoubleClick(e: MouseEvent) {
  animating.value = true
  if (scale.value > 1.05) {
    scale.value = 1
    tx.value = 0
    ty.value = 0
  } else {
    zoomAt(e.clientX, e.clientY, 2.5)
  }
}

function onImgLoad() {
  computeFit()
  ready.value = true
}

function onImgError() {
  failed.value = true
}


function ensureFit() {
  const img = imgEl.value
  if (img?.complete && img.naturalWidth) onImgLoad()
}

watch(open, (v) => {
  if (v) void nextTick(ensureFit)
})

function onKey(e: KeyboardEvent) {
  if (e.code === 'Escape' && open.value) {
    e.stopPropagation()
    dismissAnimated()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKey, true)
  window.addEventListener('resize', computeFit)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey, true)
  window.removeEventListener('resize', computeFit)
})
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="ivw-mask" @wheel="onWheel">
      <!-- only this backdrop fades, never the image/controls -->
      <div class="ivw-bg" :class="{ fading: animating }" :style="{ opacity: bgOpacity }"></div>
      <div
        ref="stageEl"
        class="ivw-stage"
        :class="{ animating, zoomed: scale > 1.01, dragging }"
        :style="{ transform: `translate(${tx}px, ${ty}px) scale(${displayScale})` }"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
        @dblclick="onDoubleClick"
      >
        <img
          ref="imgEl"
          :src="src ?? ''"
          alt=""
          draggable="false"
          :class="{ hidden: !ready }"
          @load="onImgLoad"
          @error="onImgError"
        />
        <div v-if="failed" class="ivw-err">图片加载失败</div>
      </div>
      <button class="ivw-close" aria-label="关闭" @click="dismissAnimated()">
        <X style="width: 1.375rem; height: 1.375rem" />
      </button>
      <button v-if="altered" class="ivw-restore" @click="restore()">
        <RotateCcw style="width: 0.9375rem; height: 0.9375rem" /> 恢复原始大小
      </button>
    </div>
  </Teleport>
</template>

<style scoped>
/* attached under body's first level: absolute inset 0 pins to the initial
   containing block (never position: fixed per project rules) */
.ivw-mask {
  position: absolute;
  inset: 0;
  z-index: var(--z-toast);
  touch-action: none;
}

/* only the backdrop layer fades during pull/close */
.ivw-bg {
  position: absolute;
  inset: 0;
  background: var(--card);
}

.ivw-bg.fading {
  transition: opacity 0.24s ease;
}

.ivw-stage {
  position: absolute;
  inset: 0;
  padding: 3.5rem 2rem 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: default;
  touch-action: none;
  user-select: none;
  -webkit-user-select: none;
}

.ivw-stage.zoomed {
  cursor: grab;
}

/* any drag (pull or pan) shows the hand */
.ivw-stage.dragging {
  cursor: grabbing;
}

.ivw-stage.animating {
  transition: transform 0.24s ease;
}

.ivw-stage img {
  /* sized via the stage transform (fit = fill width or height) */
  max-width: none;
  max-height: none;
  pointer-events: none;
}

.ivw-stage img.hidden {
  visibility: hidden;
}

.ivw-err {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--muted);
  font-size: 0.9375rem;
  pointer-events: none;
}

/* bank svg line art is drawn for light backgrounds */
html[data-theme='dark'] .ivw-stage img {
  filter: invert(1);
}

.ivw-close {
  position: absolute;
  top: 1rem;
  right: 1rem;
  width: 3rem;
  height: 3rem;
  border: none;
  border-radius: var(--radius-pill);
  background: var(--hover);
  color: var(--ink);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.ivw-close:hover {
  filter: brightness(0.94);
}

.ivw-restore {
  position: absolute;
  bottom: 1.25rem;
  left: 50%;
  translate: -50% 0;
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  border: none;
  border-radius: var(--radius-pill);
  background: var(--hover);
  color: var(--ink);
  font-size: 0.875rem;
  padding: 0.5rem 1rem;
  cursor: pointer;
}

.ivw-restore:hover {
  filter: brightness(0.94);
}
</style>
