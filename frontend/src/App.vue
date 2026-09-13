<script setup lang="ts">
import ImgViewer from './components/ImgViewer.vue'
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { BookOpen, Home, List, Settings } from 'lucide-vue-next'
import { loadAll } from './store'
import SettingsDialog from './components/SettingsDialog.vue'
import SubjectPicker from './components/SubjectPicker.vue'

const route = useRoute()
const fullscreen = computed(() => route.meta.fullscreen === true)
const onSplash = computed(() => route.name === 'splash')
const showNav = computed(() => !fullscreen.value || onSplash.value)
const settingsOpen = ref(false)

onMounted(() => {
  if (!fullscreen.value) loadAll()
})
</script>

<template>
  <div class="app">
    <main :class="{ fullscreen }">
      <RouterView />
    </main>

    <footer v-if="showNav" class="bottombar" :class="{ bare: onSplash }">
      <nav aria-label="主导航">
        <RouterLink to="/" aria-label="首页" title="首页"><Home style="width: 1.625rem; height: 1.625rem" /></RouterLink>
        <RouterLink to="/list" aria-label="题库" title="题库"><List style="width: 1.625rem; height: 1.625rem" /></RouterLink>
        <RouterLink to="/book" aria-label="题本" title="题本"><BookOpen style="width: 1.625rem; height: 1.625rem" /></RouterLink>
      </nav>
      <div class="bb-right">
        <SubjectPicker />
        <button class="icon-btn" aria-label="设置" title="设置" @click="settingsOpen = true">
          <Settings style="width: 1.625rem; height: 1.625rem" />
        </button>
      </div>
    </footer>

    <SettingsDialog v-if="settingsOpen" @close="settingsOpen = false" />
  </div>
  <ImgViewer />
</template>

<style scoped>
.bottombar {
  position: sticky;
  bottom: var(--bnav-gap);
  z-index: var(--z-bar);
  display: flex;
  align-items: stretch;
  gap: 1rem;
  height: var(--bnav-h);
  max-width: 74.5rem;
  margin-top: 2rem;
  margin-inline: max(var(--bnav-mx), calc((100% - 74.5rem) / 2));
  pointer-events: none;
}

.bottombar nav,
.bottombar .bb-right {
  pointer-events: auto;
}

.bottombar nav,
.bb-right {
  flex: none;
  width: 14rem;
  border-radius: var(--radius-pill);
  display: flex;
  align-items: center;
  padding-inline: calc((var(--bnav-h) - 2.75rem - 2px) / 2);
}

@container (min-width: 49.125rem) {
  .bottombar nav,
  .bb-right {
    width: min(14rem, calc((100% - 22rem) / 2));
  }
}

.bottombar:not(.bare) nav,
.bottombar:not(.bare) .bb-right {
  border:1px solid var(--line);
  background: var(--glass-bg);
  -webkit-backdrop-filter: blur(0.4375rem);
  backdrop-filter: blur(0.4375rem);
  box-shadow: var(--shadow-float);
}

.bottombar.bare nav,
.bottombar.bare .bb-right {
  border: 1px solid transparent;
  background: none;
}

.bottombar nav {
  justify-content: space-evenly;
  gap:4px;
}

/* All four icon buttons share one spec: 44px circles with 26px icons */
.bottombar nav a {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.75rem;
  height: 2.75rem;
  border-radius: 50%;
  color: var(--muted);
}

.bottombar nav a:hover {
  background: var(--hover);
  color: var(--ink);
}

.bottombar nav a.router-link-exact-active {
  color: var(--brand);
  background: var(--brand-weak);
}

.bb-right {
  margin-left: auto;
  justify-content: flex-end;
  gap: 0.5rem;
}

.bb-right .icon-btn {
  width: 2.75rem;
  height: 2.75rem;
  border-radius: 50%;
}

.bb-right :deep(.subj-sel) {
  flex: 1;
  min-width: 0;
  max-width: none;
  background: transparent;
}


/* Narrow screens: merge into one whole capsule — background/border/shadow/rounding are carried by
   the bar body (not two joined segments); the inner nav segment and right cluster are fully
   transparent; button spacing tightens and the subject dropdown gets a width cap */
@container (max-width: 40rem) {
  .bottombar {
    gap: 0.375rem;
    padding:calc((var(--bnav-h) - 2.75rem - 2px) / 2);
    pointer-events: auto;
  }

  .bottombar:not(.bare) {
    border:1px solid var(--line);
    border-radius: var(--radius-pill);
    background: var(--glass-bg);
    -webkit-backdrop-filter: blur(0.4375rem);
    backdrop-filter: blur(0.4375rem);
    box-shadow: var(--shadow-float);
  }

  .bottombar:not(.bare) nav,
  .bottombar:not(.bare) .bb-right {
    background: none;
    border: none;
    box-shadow: none;
    -webkit-backdrop-filter: none;
    backdrop-filter: none;
  }

  .bottombar nav,
  .bb-right {
    width: auto;
    flex: 0 1 auto;
    padding: 0;
  }

  .bottombar nav {
    gap:2px;
  }

  .bb-right {
    gap:4px;
  }

  .bb-right :deep(.subj-sel) {
    flex: none;
    max-width: 6.5rem;
  }
}

@container (max-width: 22.5rem) {
  .bb-right :deep(.subj-sel) {
    display: none;
  }
}

/* Very narrow (≤360px): round buttons uniformly shrink to 40px so both sides still fit */
@container (max-width: 22.5rem) {
  .bottombar nav a,
  .bb-right .icon-btn {
    width: 2.5rem;
    height: 2.5rem;
  }
}

@container (max-height: 35rem) and (min-width: 40.0625rem) {
  .bottombar.bare nav,
  .bottombar.bare .bb-right {
    border:1px solid var(--line);
    background: var(--glass-bg);
    -webkit-backdrop-filter: blur(0.4375rem);
    backdrop-filter: blur(0.4375rem);
    box-shadow: var(--shadow-float);
  }
}

@container (max-height: 35rem) and (max-width: 40rem) {
  .bottombar.bare {
    border:1px solid var(--line);
    border-radius: var(--radius-pill);
    background: var(--glass-bg);
    -webkit-backdrop-filter: blur(0.4375rem);
    backdrop-filter: blur(0.4375rem);
    box-shadow: var(--shadow-float);
  }
}
</style>
