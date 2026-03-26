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

export interface Model {
  id: string;
  name: string;
  input_1m_usd: number | null;
  output_1m_usd: number | null;
  cache_write_1m_usd: number | null;
  cache_read_1m_usd: number | null;
  created_at: string;
}

export interface GroupServerDetail {
  server_id: string;
  short_id: number;
  server_name: string;
  base_url: string;
  api_key: string | null;
  priority: number;
  model_mappings: Record<string, string>;
  is_enabled: boolean;
  cb_max_failures: number | null;
  cb_window_seconds: number | null;
  cb_cooldown_seconds: number | null;
  rate_input: number | null;
  rate_output: number | null;
  rate_cache_write: number | null;
  rate_cache_read: number | null;
}

export interface GroupWithServers extends Group {
  servers: GroupServerDetail[];
  allowed_models: Model[];
}

interface PaginatedResponse {
  data: Group[];
  total: number;
  page: number;
  total_pages: number;
}

export interface CircuitStatus {
  server_id: string;
  is_open: boolean;
  remaining_seconds: number;
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

export interface ServerTokenUsage {
  server_id: string;
  server_name: string;
  model: string | null;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cache_creation_tokens: number;
  total_cache_read_tokens: number;
  request_count: number;
  cost_usd: number | null;
}

export interface TokenUsageStats {
  servers: ServerTokenUsage[];
}

export interface GroupKey {
  id: string;
  group_id: string;
  api_key: string;
  name: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

interface PaginatedGroupKeys {
  data: GroupKey[];
  total: number;
  page: number;
  total_pages: number;
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

  async function updateAssignment(
    groupId: string,
    serverId: string,
    input: {
      priority?: number;
      model_mappings?: Record<string, string>;
      is_enabled?: boolean;
      cb_max_failures?: number | null;
      cb_window_seconds?: number | null;
      cb_cooldown_seconds?: number | null;
      rate_input?: number | null;
      rate_output?: number | null;
      rate_cache_write?: number | null;
      rate_cache_read?: number | null;
    },
  ) {
    await api.put(`/api/admin/groups/${groupId}/servers/${serverId}`, input);
  }

  async function removeServer(groupId: string, serverId: string) {
    await api.delete(`/api/admin/groups/${groupId}/servers/${serverId}`);
  }

  async function reorderServers(groupId: string, serverIds: string[]) {
    await api.put(`/api/admin/groups/${groupId}/servers/reorder`, { server_ids: serverIds });
  }

  async function fetchTtftStats(groupId: string, range?: { start: string; end: string }, groupKeyId?: string) {
    const params: Record<string, string> = { group_id: groupId };
    if (range) {
      params.start = range.start;
      params.end = range.end;
    } else {
      params.period = '1h';
    }
    if (groupKeyId) {
      params.group_key_id = groupKeyId;
    }
    const { data } = await api.get<TtftStatsResponse>('/api/admin/ttft-stats', { params });
    return data;
  }

  async function fetchCircuitStatus(groupId: string) {
    const { data } = await api.get<CircuitStatus[]>(
      `/api/admin/groups/${groupId}/circuit-status`,
    );
    return data;
  }

  async function fetchTokenUsageStats(
    groupId: string,
    params?: { start?: string; end?: string; period?: string; is_dynamic_key?: boolean; key_hash?: string; group_key_id?: string },
  ) {
    const qp: Record<string, string> = { group_id: groupId };
    if (params?.start) qp.start = params.start;
    if (params?.end) qp.end = params.end;
    if (params?.period) qp.period = params.period;
    if (params?.is_dynamic_key !== undefined) qp.is_dynamic_key = String(params.is_dynamic_key);
    if (params?.key_hash) qp.key_hash = params.key_hash;
    if (params?.group_key_id) qp.group_key_id = params.group_key_id;
    const { data } = await api.get<TokenUsageStats>('/api/admin/token-usage', { params: qp });
    return data;
  }

  async function fetchGroupKeys(
    groupId: string,
    params?: { page?: number; limit?: number; search?: string },
  ) {
    const { data } = await api.get<PaginatedGroupKeys>(`/api/admin/groups/${groupId}/keys`, { params });
    return data;
  }

  async function createGroupKey(groupId: string, name: string) {
    const { data } = await api.post<GroupKey>(`/api/admin/groups/${groupId}/keys`, { name });
    return data;
  }

  async function updateGroupKey(groupId: string, keyId: string, input: { name?: string; is_active?: boolean }) {
    const { data } = await api.patch<GroupKey>(`/api/admin/groups/${groupId}/keys/${keyId}`, input);
    return data;
  }

  async function regenerateGroupKey(groupId: string, keyId: string) {
    const { data } = await api.post<GroupKey>(`/api/admin/groups/${groupId}/keys/${keyId}/regenerate`);
    return data;
  }

  async function fetchKeyUsage(
    groupId: string,
    groupKeyId: string,
    params?: { period?: string; start?: string; end?: string },
  ) {
    return fetchTokenUsageStats(groupId, { ...params, group_key_id: groupKeyId });
  }

  // Group allowed models
  async function fetchGroupAllowedModels(groupId: string) {
    const { data } = await api.get<Model[]>(`/api/admin/groups/${groupId}/allowed-models`);
    return data;
  }

  async function addGroupAllowedModel(groupId: string, input: { model_id?: string; name?: string }) {
    const { data } = await api.post<Model>(`/api/admin/groups/${groupId}/allowed-models`, input);
    return data;
  }

  async function removeGroupAllowedModel(groupId: string, modelId: string) {
    await api.delete(`/api/admin/groups/${groupId}/allowed-models/${modelId}`);
  }

  // Key allowed models
  async function fetchKeyAllowedModels(groupId: string, keyId: string) {
    const { data } = await api.get<Model[]>(`/api/admin/groups/${groupId}/keys/${keyId}/allowed-models`);
    return data;
  }

  async function addKeyAllowedModel(groupId: string, keyId: string, modelId: string) {
    const { data } = await api.post<Model>(`/api/admin/groups/${groupId}/keys/${keyId}/allowed-models`, { model_id: modelId });
    return data;
  }

  async function removeKeyAllowedModel(groupId: string, keyId: string, modelId: string) {
    await api.delete(`/api/admin/groups/${groupId}/keys/${keyId}/allowed-models/${modelId}`);
  }

  return {
    groups, total, totalPages, loading,
    fetchGroups, getGroup, createGroup, updateGroup, deleteGroup, regenerateKey,
    bulkActivate, bulkDeactivate, bulkDelete, bulkAssignServer,
    assignServer, updateAssignment, removeServer, reorderServers,
    fetchTtftStats, fetchCircuitStatus, fetchTokenUsageStats,
    fetchGroupKeys, createGroupKey, updateGroupKey, regenerateGroupKey, fetchKeyUsage,
    fetchGroupAllowedModels, addGroupAllowedModel, removeGroupAllowedModel,
    fetchKeyAllowedModels, addKeyAllowedModel, removeKeyAllowedModel,
  };
});
