import { defineStore } from 'pinia';
import { api } from 'boot/axios';
import type { Model } from 'stores/groups';

interface PaginatedModels {
  data: Model[];
  total: number;
  page: number;
  total_pages: number;
}

export const useModelsStore = defineStore('models', () => {
  async function fetchModels(params?: { page?: number; limit?: number; search?: string }) {
    const { data } = await api.get<PaginatedModels>('/api/admin/models', { params });
    return data;
  }

  async function createModel(input: {
    name: string;
    input_1m_usd?: number | null;
    output_1m_usd?: number | null;
    cache_write_1m_usd?: number | null;
    cache_read_1m_usd?: number | null;
  }) {
    const { data } = await api.post<Model>('/api/admin/models', input);
    return data;
  }

  async function updateModel(
    id: string,
    input: {
      name?: string;
      input_1m_usd?: number | null;
      output_1m_usd?: number | null;
      cache_write_1m_usd?: number | null;
      cache_read_1m_usd?: number | null;
    },
  ) {
    const { data } = await api.put<Model>(`/api/admin/models/${id}`, input);
    return data;
  }

  async function deleteModel(id: string) {
    await api.delete(`/api/admin/models/${id}`);
  }

  return { fetchModels, createModel, updateModel, deleteModel };
});
