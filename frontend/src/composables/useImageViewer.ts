import { ref } from 'vue'

export interface ViewerTarget {
  url: string

  rect: { x: number; y: number; w: number; h: number } | null
}

const target = ref<ViewerTarget | null>(null)

export function openImageViewer(url: string, originEl?: Element | null) {
  let rect: ViewerTarget['rect'] = null
  if (originEl) {
    const r = originEl.getBoundingClientRect()
    rect = { x: r.x, y: r.y, w: r.width, h: r.height }
  }
  target.value = { url, rect }
}

export function closeImageViewer() {
  target.value = null
}

export function useImageViewerState() {
  return target
}
