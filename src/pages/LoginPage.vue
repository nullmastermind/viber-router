<template>
  <div class="login-page flex flex-center">
    <q-card bordered style="width: 400px">
      <q-card-section>
        <div class="text-h5 text-center">Viber Router</div>
        <div class="text-center login-subtitle">Admin Dashboard</div>
      </q-card-section>
      <q-card-section>
        <q-input
          v-model="token"
          label="Admin Token"
          type="password"
          outlined
          @keyup.enter="login"
        />
        <div v-if="error" class="text-negative q-mt-sm" style="font-size: 13px">{{ error }}</div>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn label="Login" color="primary" :loading="loading" @click="login" />
      </q-card-actions>
    </q-card>
  </div>
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

<style scoped lang="scss">
.login-page {
  min-height: 100vh;
  background-color: var(--vr-bg-page);
}

.login-subtitle {
  font-size: 13px;
  color: var(--vr-text-secondary);
  margin-top: 4px;
}
</style>
