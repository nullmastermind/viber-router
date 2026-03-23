<template>
  <q-layout view="lHh Lpr lFf">
    <q-header elevated>
      <q-toolbar>
        <q-btn flat dense round icon="menu" aria-label="Menu" @click="toggleLeftDrawer" />
        <q-toolbar-title>Viber Router</q-toolbar-title>
        <q-btn flat label="Logout" icon="logout" @click="logout" />
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
      </q-list>
    </q-drawer>

    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();
const leftDrawerOpen = ref(false);

function toggleLeftDrawer() {
  leftDrawerOpen.value = !leftDrawerOpen.value;
}

function logout() {
  localStorage.removeItem('admin_token');
  router.push('/login');
}
</script>
