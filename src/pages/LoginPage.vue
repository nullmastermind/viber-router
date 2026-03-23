<template>
  <q-page class="flex flex-center">
    <q-card style="width: 400px">
      <q-card-section>
        <div class="text-h5 text-center">Viber Router Admin</div>
      </q-card-section>
      <q-card-section>
        <q-input
          v-model="token"
          label="Admin Token"
          type="password"
          outlined
          @keyup.enter="login"
        />
        <div v-if="error" class="text-negative q-mt-sm">{{ error }}</div>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn label="Login" color="primary" :loading="loading" @click="login" />
      </q-card-actions>
    </q-card>
  </q-page>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { api } from 'boot/axios';

const router = useRouter();
const token = ref('');
const error = ref('');
const loading = ref(false);

async function login() {
  error.value = '';
  loading.value = true;
  try {
    await api.post('/api/admin/login', { token: token.value });
    localStorage.setItem('admin_token', token.value);
    router.push('/servers');
  } catch {
    error.value = 'Invalid admin token';
  } finally {
    loading.value = false;
  }
}
</script>
