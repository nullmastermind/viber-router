import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from 'boot/axios';

export interface Server {
  id: string;
  name: string;
  base_url: string;
  api_key: string;
  created_at: string;
  updated_at: string;
}

interface PaginatedResponse {
  data: Server[];
  total: number;
  page: number;
  total_pages: number;
}

export const useServersStore = defineStore('servers', () => {
  const servers = ref<Server[]>([]);
  const total = ref(0);
  const loading = ref(false);

  async function fetchServers(params?: { page?: number; limit?: number; search?: string }) {
    loading.value = true;
    try {
      const { data } = await api.get<PaginatedResponse>('/api/admin/servers', { params });
      servers.value = data.data;
      total.value = data.total;
      return data;
    } finally {
      loading.value = false;
    }
  }

  async function createServer(input: { name: string; base_url: string; api_key: string }) {
    const { data } = await api.post<Server>('/api/admin/servers', input);
    return data;
  }

  async function updateServer(id: string, input: { name?: string; base_url?: string; api_key?: string }) {
    const { data } = await api.put<Server>(`/api/admin/servers/${id}`, input);
    return data;
  }

  async function deleteServer(id: string) {
    await api.delete(`/api/admin/servers/${id}`);
  }

  return { servers, total, loading, fetchServers, createServer, updateServer, deleteServer };
});
