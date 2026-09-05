import { onBeforeUnmount, ref } from 'vue'

export function useStudyTimer(canRun: () => boolean) {
  const sessionMs = ref(0)
  const questionMs = ref(0)
  const paused = ref(false)
  let lastTick = performance.now()
  let timerId: number | undefined

  function tick() {
    const now = performance.now()
    const dt = now - lastTick
    lastTick = now
    if (canRun() && !paused.value && !document.hidden) {
      sessionMs.value += dt
      questionMs.value += dt
    }
  }

  function start() {
    stop()
    lastTick = performance.now()
    timerId = window.setInterval(tick, 250)
  }

  function stop() {
    if (timerId !== undefined) {
      window.clearInterval(timerId)
      timerId = undefined
    }
  }

  function resetQuestion() {
    questionMs.value = 0
  }

  function resetAll() {
    sessionMs.value = 0
    questionMs.value = 0
  }

  onBeforeUnmount(stop)
  return { sessionMs, questionMs, paused, start, stop, resetQuestion, resetAll }
}
