<script setup lang="ts">
import { Pause, Trophy } from 'lucide-vue-next'

defineProps<{
  interrupted: boolean
  judged: number
  total: number
  right: number
  wrong: number
  unanswered: number
  sessionMs: string
}>()

const emit = defineEmits<{
  (e: 'continue'): void
  (e: 'retry-wrong'): void
  (e: 'retry-all'): void
  (e: 'home'): void
}>()
</script>

<template>
  <Teleport to="body">
  <div class="sum">
    <div class="sum-card">
      <Pause v-if="interrupted" style="width: 2.5rem; height: 2.5rem" class="icon" />
      <Trophy v-else style="width: 2.5rem; height: 2.5rem" class="icon trophy" />
      <h2>{{ interrupted ? '本次总结' : '本轮完成' }}</h2>

      <div class="stats">
        <div class="stat">
          <span class="num good-t">{{ right }}</span>
          <span class="lab">做对</span>
        </div>
        <div class="stat">
          <span class="num bad-t">{{ wrong }}</span>
          <span class="lab">做错</span>
        </div>
        <div v-if="!interrupted" class="stat">
          <span class="num">{{ unanswered }}</span>
          <span class="lab">未做</span>
        </div>
        <div class="stat">
          <span class="num">{{ sessionMs }}</span>
          <span class="lab">用时</span>
        </div>
      </div>

      <div v-if="interrupted" class="tip">进度已保存，随时继续</div>

      <div class="actions">
        <button v-if="interrupted" class="btn primary big" @click="emit('continue')">
          继续刷题 <span class="kbd">空格</span>
        </button>
        <template v-else>
          <button class="btn primary big" @click="emit('retry-all')">再刷一轮</button>
          <button class="btn big" :disabled="!wrong" @click="emit('retry-wrong')">
            只刷做错的（{{ wrong }}）
          </button>
        </template>
        <button class="btn ghost big" @click="emit('home')">
          返回首页 <span class="kbd">Esc</span>
        </button>
      </div>
    </div>
  </div>
  </Teleport>
</template>

<style scoped>
.sum {
  position: absolute;
  inset: 0;
  z-index: 60;
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    radial-gradient(120% 90% at 50% 0%, var(--brand-weak) 0%, transparent 55%),
    var(--bg);
  padding: calc(1.5rem + env(safe-area-inset-top)) 1.5rem calc(1.5rem + env(safe-area-inset-bottom));
}

.sum-card {
  width: min(30rem, 100%);
  text-align: center;
  animation: rise 0.35s ease both;
}

@keyframes rise {
  from {
    opacity: 0;
    transform: translateY(0.875rem);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.icon {
  color: var(--brand);
}

.trophy {
  color: var(--warn);
}

h2 {
  margin: 0.75rem 0 1.75rem;
  font-size: 1.375rem;
  letter-spacing: 0.125rem;
}

.stats {
  display: flex;
  justify-content: center;
  gap: 2.25rem;
  margin-bottom: 2.25rem;
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.num {
  font-size: 2rem;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  line-height: 1;
}

.good-t {
  color: var(--good);
}

.bad-t {
  color: var(--bad);
}

.lab {
  font-size: 0.75rem;
  color: var(--muted);
  letter-spacing: 0.125rem;
}

.tip {
  font-size: 0.8125rem;
  color: var(--muted);
  margin-bottom: 1.5rem;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
  max-width: 18rem;
  margin: 0 auto;
}

.btn.big {
  min-height: 3rem;
}

.kbd {
  font-size: 0.7188rem;
  padding:1px 0.4375rem;
  border: 1px solid var(--line);
  border-radius:0.375rem;
  opacity: 0.85;
}
</style>
