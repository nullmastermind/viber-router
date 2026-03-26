import type { RouteRecordRaw } from 'vue-router';

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    component: () => import('pages/LoginPage.vue'),
  },
  {
    path: '/',
    component: () => import('layouts/MainLayout.vue'),
    children: [
      { path: '', redirect: '/servers' },
      { path: 'servers', component: () => import('pages/ServersPage.vue') },
      { path: 'groups', component: () => import('pages/GroupsPage.vue') },
      { path: 'groups/:id', component: () => import('pages/GroupDetailPage.vue') },
      { path: 'models', component: () => import('pages/ModelsPage.vue') },
      { path: 'plans', component: () => import('pages/PlansPage.vue') },
      { path: 'logs', component: () => import('pages/LogsPage.vue') },
      { path: 'settings', component: () => import('pages/SettingsPage.vue') },
    ],
  },
  {
    path: '/:catchAll(.*)*',
    component: () => import('pages/ErrorNotFound.vue'),
  },
];

export default routes;
