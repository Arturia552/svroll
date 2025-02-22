import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';

const defaultRouterList: Array<RouteRecordRaw> = [
  {
    path: '',
    name: 'Home',
    component: () => import('@/pages/home.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes: defaultRouterList,
  scrollBehavior() {
    return {
      el: '#app',
      top: 0,
      behavior: 'smooth',
    };
  },
});

export default router;