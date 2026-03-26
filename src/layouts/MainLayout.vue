<template>
  <q-layout view="lHh Lpr lFf">
    <q-header>
      <q-toolbar>
        <q-btn flat dense round icon="menu" aria-label="Menu" @click="toggleLeftDrawer" />
        <q-toolbar-title>Viber Router</q-toolbar-title>
        <q-btn
          flat dense round
          :icon="isDark ? 'light_mode' : 'dark_mode'"
          :aria-label="isDark ? 'Switch to light mode' : 'Switch to dark mode'"
          @click="toggleDark"
        />
        <q-btn flat dense icon="logout" aria-label="Logout" @click="logout" />
      </q-toolbar>
    </q-header>

    <q-drawer v-model="leftDrawerOpen" show-if-above bordered>
      <q-list>
        <q-item-label header>Admin</q-item-label>
        <q-item clickable :to="'/servers'" :active="$route.path === '/servers'">
          <q-item-section avatar><q-icon name="dns" /></q-item-section>
          <q-item-section>Servers</q-item-section>
        </q-item>
        <q-item clickable :to="'/groups'" :active="$route.path.startsWith('/groups')">
          <q-item-section avatar><q-icon name="group_work" /></q-item-section>
          <q-item-section>Groups</q-item-section>
        </q-item>
        <q-item clickable :to="'/models'" :active="$route.path === '/models'">
          <q-item-section avatar><q-icon name="smart_toy" /></q-item-section>
          <q-item-section>Models</q-item-section>
        </q-item>
        <q-item clickable :to="'/plans'" :active="$route.path === '/plans'">
          <q-item-section avatar><q-icon name="card_membership" /></q-item-section>
          <q-item-section>Plans</q-item-section>
        </q-item>
        <q-item clickable :to="'/logs'" :active="$route.path === '/logs'">
          <q-item-section avatar><q-icon name="list_alt" /></q-item-section>
          <q-item-section>Logs</q-item-section>
        </q-item>
        <q-item clickable :to="'/settings'" :active="$route.path === '/settings'">
          <q-item-section avatar><q-icon name="settings" /></q-item-section>
          <q-item-section>Settings</q-item-section>
        </q-item>
      </q-list>
    </q-drawer>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useQuasar } from 'quasar';

const $q = useQuasar();
const router = useRouter();
const leftDrawerOpen = ref(false);
const isDark = ref(false);

onMounted(() => {
  const saved = localStorage.getItem('dark-mode');
  if (saved !== null) {
    isDark.value = saved === 'true';
  } else {
    isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  $q.dark.set(isDark.value);
});

function toggleDark() {
  isDark.value = !isDark.value;
  $q.dark.set(isDark.value);
  localStorage.setItem('dark-mode', String(isDark.value));
}

function toggleLeftDrawer() {
  leftDrawerOpen.value = !leftDrawerOpen.value;
}

function logout() {
  localStorage.removeItem('admin_token');
  router.push('/login');
}
</script>
