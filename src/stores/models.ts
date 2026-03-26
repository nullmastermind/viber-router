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

  async function createModel(name: string) {
    const { data } = await api.post<Model>('/api/admin/models', { name });
    return data;
  }

  async function deleteModel(id: string) {
    await api.delete(`/api/admin/models/${id}`);
  }

  return { fetchModels, createModel, deleteModel };
});
