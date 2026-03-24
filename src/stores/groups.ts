import { defineStore } from 'pinia';
import { ref } from 'vue';
import { api } from 'boot/axios';

export interface Group {
  id: string;
  name: string;
  api_key: string;
  failover_status_codes: number[];
  is_active: boolean;
  ttft_timeout_ms: number | null;
  count_tokens_server_id: string | null;
  count_tokens_model_mappings: Record<string, string>;
  created_at: string;
  updated_at: string;
}

export interface GroupServerDetail {
  server_id: string;
  short_id: number;
  server_name: string;
  base_url: string;
  api_key: string | null;
  priority: number;
  model_mappings: Record<string, string>;
}

export interface GroupWithServers extends Group {
  servers: GroupServerDetail[];
}

interface PaginatedResponse {
  data: Group[];
  total: number;
  page: number;
  total_pages: number;
}

export interface TtftDataPoint {
  created_at: string;
  ttft_ms: number | null;
  timed_out: boolean;
}

export interface ServerTtftStats {
  server_id: string;
  server_name: string;
  avg_ttft_ms: number | null;
  p50_ttft_ms: number | null;
  p95_ttft_ms: number | null;
  timeout_count: number;
  total_count: number;
  data_points: TtftDataPoint[];
}

export interface TtftStatsResponse {
  servers: ServerTtftStats[];
}

export const useGroupsStore = defineStore('groups', () => {
  const groups = ref<Group[]>([]);
  const total = ref(0);
  const totalPages = ref(0);
  const loading = ref(false);

  async function fetchGroups(params?: {
    page?: number; limit?: number; search?: string;
    is_active?: boolean; server_id?: string;
  }) {
    loading.value = true;
    try {
      const { data } = await api.get<PaginatedResponse>('/api/admin/groups', { params });
      groups.value = data.data;
      total.value = data.total;
      totalPages.value = data.total_pages;
      return data;
    } finally {
      loading.value = false;
    }
  }

  async function getGroup(id: string) {
    const { data } = await api.get<GroupWithServers>(`/api/admin/groups/${id}`);
    return data;
  }

  async function createGroup(input: { name: string; failover_status_codes?: number[] }) {
    const { data } = await api.post<Group>('/api/admin/groups', input);
    return data;
  }

  async function updateGroup(id: string, input: { name?: string; failover_status_codes?: number[]; is_active?: boolean; ttft_timeout_ms?: number | null; count_tokens_server_id?: string | null; count_tokens_model_mappings?: Record<string, string> }) {
    const { data } = await api.put<Group>(`/api/admin/groups/${id}`, input);
    return data;
  }

  async function deleteGroup(id: string) {
    await api.delete(`/api/admin/groups/${id}`);
  }

  async function regenerateKey(id: string) {
    const { data } = await api.post<Group>(`/api/admin/groups/${id}/regenerate-key`);
    return data;
  }

  async function bulkActivate(ids: string[]) {
    await api.post('/api/admin/groups/bulk/activate', { ids });
  }

  async function bulkDeactivate(ids: string[]) {
    await api.post('/api/admin/groups/bulk/deactivate', { ids });
  }

  async function bulkDelete(ids: string[]) {
    await api.post('/api/admin/groups/bulk/delete', { ids });
  }

  async function bulkAssignServer(group_ids: string[], server_id: string, priority: number, model_mappings?: Record<string, string>) {
    await api.post('/api/admin/groups/bulk/assign-server', { group_ids, server_id, priority, model_mappings });
  }

  // Group-server assignment actions
  async function assignServer(groupId: string, input: { server_id: string; priority: number; model_mappings?: Record<string, string> }) {
    await api.post(`/api/admin/groups/${groupId}/servers`, input);
  }

  async function updateAssignment(groupId: string, serverId: string, input: { priority?: number; model_mappings?: Record<string, string> }) {
    await api.put(`/api/admin/groups/${groupId}/servers/${serverId}`, input);
  }

  async function removeServer(groupId: string, serverId: string) {
    await api.delete(`/api/admin/groups/${groupId}/servers/${serverId}`);
  }

  async function reorderServers(groupId: string, serverIds: string[]) {
    await api.put(`/api/admin/groups/${groupId}/servers/reorder`, { server_ids: serverIds });
  }

  async function fetchTtftStats(groupId: string) {
    const { data } = await api.get<TtftStatsResponse>('/api/admin/ttft-stats', {
      params: { group_id: groupId, period: '1h' },
    });
    return data;
  }

  return {
    groups, total, totalPages, loading,
    fetchGroups, getGroup, createGroup, updateGroup, deleteGroup, regenerateKey,
    bulkActivate, bulkDeactivate, bulkDelete, bulkAssignServer,
    assignServer, updateAssignment, removeServer, reorderServers,
    fetchTtftStats,
  };
});
