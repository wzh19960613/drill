import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'splash', component: () => import('./views/SplashView.vue'), meta: { fullscreen: true } },
    { path: '/study', name: 'study', component: () => import('./views/StudyView.vue'), meta: { fullscreen: true } },
    { path: '/list', name: 'list', component: () => import('./views/ListView.vue') },
    { path: '/book', name: 'book', component: () => import('./views/BookHomeView.vue') },
    { path: '/book/study', name: 'bookStudy', component: () => import('./views/StudyView.vue'), meta: { fullscreen: true } },
  ],
})

export default router
