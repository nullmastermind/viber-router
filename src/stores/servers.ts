import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from 'boot/axios';

export interface Server {
  id: string;
  short_id: number;
  name: string;
  base_url: string | null;
  api_key: string | null;
  password_hash: string | null;
  system_prompt: string | null;
  remove_thinking: boolean;
  created_at: string;
  updated_at: string;
}

interface PaginatedResponse {
  data: Server[];
  total: number;
  page: number;
  total_pages: number;
}

interface UnlockResult {
  base_url: string;
  api_key: string | null;
}

export const useServersStore = defineStore('servers', () => {
  const servers = ref<Server[]>([]);
  const total = ref(0);
  const loading = ref(false);
  const protectedServerIds = ref(new Set<string>());
  const unlockedServers = ref(new Set<string>());

  async function fetchServers(params?: { page?: number; limit?: number; search?: string }) {
    loading.value = true;
    try {
      const { data } = await api.get<PaginatedResponse>('/api/admin/servers', { params });
      servers.value = data.data;
      total.value = data.total;
      // Populate protected server IDs
      protectedServerIds.value = new Set(
        data.data.filter((s) => s.password_hash != null).map((s) => s.id),
      );
      return data;
    } finally {
      loading.value = false;
    }
  }

  async function createServer(input: {
    name: string;
    base_url: string;
    api_key?: string;
    password?: string;
    system_prompt?: string | null;
    remove_thinking?: boolean;
  }) {
    const { data } = await api.post<Server>('/api/admin/servers', input);
    return data;
  }

  async function updateServer(
    id: string,
    input: {
      name?: string;
      base_url?: string;
      api_key?: string | null;
      password?: string | null;
      system_prompt?: string | null;
      remove_thinking?: boolean;
    },
  ) {
    const { data } = await api.put<Server>(`/api/admin/servers/${id}`, input);
    return data;
  }

  async function deleteServer(id: string) {
    await api.delete(`/api/admin/servers/${id}`);
  }

  function isProtected(id: string): boolean {
    return protectedServerIds.value.has(id);
  }

  function isUnlocked(id: string): boolean {
    return unlockedServers.value.has(id);
  }

  async function unlockServer(id: string, password: string): Promise<UnlockResult> {
    try {
      const { data } = await api.post<UnlockResult>(`/api/admin/servers/${id}/verify-password`, {
        password,
      });
      unlockedServers.value.add(id);
      return data;
    } catch (e: unknown) {
      const status = (e as { response?: { status?: number } })?.response?.status;
      if (status === 401) {
        throw new Error('Incorrect password');
      }
      throw e;
    }
  }

  function lockServer(id: string): void {
    unlockedServers.value.delete(id);
  }

  return {
    servers,
    total,
    loading,
    protectedServerIds,
    unlockedServers,
    fetchServers,
    createServer,
    updateServer,
    deleteServer,
    isProtected,
    isUnlocked,
    unlockServer,
    lockServer,
  };
});
