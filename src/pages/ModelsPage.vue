<template>
  <q-page padding>
    <div class="row items-center q-mb-md">
      <div class="text-h5">Models</div>
      <q-space />
      <q-input v-model="search" placeholder="Search models" outlined dense clearable style="max-width: 250px" class="q-mr-sm" @update:model-value="onSearch" />
      <q-btn color="primary" label="Add Model" @click="openCreate" />
    </div>

    <div v-if="loading && !models.length" class="flex flex-center q-pa-lg"><q-spinner size="md" /></div>
    <q-banner v-else-if="error" class="bg-negative text-white q-mb-sm" rounded>
      {{ error }}
      <template #action><q-btn flat label="Retry" @click="load" /></template>
    </q-banner>
    <q-banner v-else-if="!models.length" class="q-mb-sm" rounded>No models configured</q-banner>
    <q-table
      v-else
      flat bordered dense
      :rows="models"
      :columns="columns"
      row-key="id"
      v-model:pagination="pagination"
      @request="onRequest"
    >
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn flat dense icon="edit" :aria-label="`Edit model ${props.row.name}`" @click="openEdit(props.row)" />
          <q-btn flat dense icon="delete" color="negative" :aria-label="`Delete model ${props.row.name}`" @click="onDelete(props.row)" />
        </q-td>
      </template>
    </q-table>

    <q-dialog v-model="showDialog" @hide="resetForm">
      <q-card style="width: 450px">
        <q-card-section><div class="text-h6">{{ editingId ? 'Edit Model' : 'Create Model' }}</div></q-card-section>
        <q-card-section class="q-gutter-sm">
          <q-input v-model="form.name" label="Name" outlined dense />
          <q-input v-model.number="form.input_1m_usd" label="Input ($/1MTok)" outlined dense type="number" :min="0" :rules="[nonNegativeRule]" clearable @clear="form.input_1m_usd = null" />
          <q-input v-model.number="form.output_1m_usd" label="Output ($/1MTok)" outlined dense type="number" :min="0" :rules="[nonNegativeRule]" clearable @clear="form.output_1m_usd = null" />
          <q-input v-model.number="form.cache_write_1m_usd" label="Cache Write ($/1MTok)" outlined dense type="number" :min="0" :rules="[nonNegativeRule]" clearable @clear="form.cache_write_1m_usd = null" />
          <q-input v-model.number="form.cache_read_1m_usd" label="Cache Read ($/1MTok)" outlined dense type="number" :min="0" :rules="[nonNegativeRule]" clearable @clear="form.cache_read_1m_usd = null" />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup />
          <q-btn color="primary" :label="editingId ? 'Save' : 'Create'" :loading="saving" @click="onSave" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useQuasar } from 'quasar';
import { useModelsStore } from 'stores/models';
import type { Model } from 'stores/groups';

const $q = useQuasar();
const modelsStore = useModelsStore();

const models = ref<Model[]>([]);
const loading = ref(false);
const error = ref('');
const search = ref('');
const pagination = ref({ page: 1, rowsPerPage: 20, rowsNumber: 0, sortBy: '', descending: false });

const showDialog = ref(false);
const editingId = ref<string | null>(null);
const saving = ref(false);
const form = reactive({
  name: '',
  input_1m_usd: null as number | null,
  output_1m_usd: null as number | null,
  cache_write_1m_usd: null as number | null,
  cache_read_1m_usd: null as number | null,
});

const nonNegativeRule = (v: number | null) => v === null || v === undefined || v >= 0 || 'Must be non-negative';

const fmtPrice = (v: number | null) => (v != null ? `$${v.toFixed(4)}` : '\u2014');

const columns = [
  { name: 'name', label: 'Name', field: 'name', align: 'left' as const, sortable: true },
  { name: 'input_1m_usd', label: 'Input ($/1MTok)', field: 'input_1m_usd', align: 'right' as const, format: fmtPrice },
  { name: 'output_1m_usd', label: 'Output ($/1MTok)', field: 'output_1m_usd', align: 'right' as const, format: fmtPrice },
  { name: 'cache_write_1m_usd', label: 'Cache Write ($/1MTok)', field: 'cache_write_1m_usd', align: 'right' as const, format: fmtPrice },
  { name: 'cache_read_1m_usd', label: 'Cache Read ($/1MTok)', field: 'cache_read_1m_usd', align: 'right' as const, format: fmtPrice },
  { name: 'actions', label: 'Actions', field: 'id', align: 'right' as const },
];

async function load() {
  loading.value = true;
  error.value = '';
  try {
    const params: { page?: number; limit?: number; search?: string } = {
      page: pagination.value.page,
      limit: pagination.value.rowsPerPage,
    };
    if (search.value) params.search = search.value;
    const result = await modelsStore.fetchModels(params);
    models.value = result.data;
    pagination.value.rowsNumber = result.total;
  } catch {
    error.value = 'Failed to load models';
  } finally {
    loading.value = false;
  }
}

function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  load();
}

function onSearch() {
  pagination.value.page = 1;
  load();
}

function openCreate() {
  editingId.value = null;
  resetForm();
  showDialog.value = true;
}

function openEdit(row: Model) {
  editingId.value = row.id;
  form.name = row.name;
  form.input_1m_usd = row.input_1m_usd;
  form.output_1m_usd = row.output_1m_usd;
  form.cache_write_1m_usd = row.cache_write_1m_usd;
  form.cache_read_1m_usd = row.cache_read_1m_usd;
  showDialog.value = true;
}

function resetForm() {
  form.name = '';
  form.input_1m_usd = null;
  form.output_1m_usd = null;
  form.cache_write_1m_usd = null;
  form.cache_read_1m_usd = null;
}

async function onSave() {
  if (!form.name.trim()) {
    $q.notify({ type: 'negative', message: 'Name is required' });
    return;
  }
  for (const field of ['input_1m_usd', 'output_1m_usd', 'cache_write_1m_usd', 'cache_read_1m_usd'] as const) {
    if (form[field] !== null && form[field] !== undefined && (form[field] as number) < 0) {
      $q.notify({ type: 'negative', message: `${field} must be non-negative` });
      return;
    }
  }
  saving.value = true;
  try {
    if (editingId.value) {
      await modelsStore.updateModel(editingId.value, {
        name: form.name,
        input_1m_usd: form.input_1m_usd,
        output_1m_usd: form.output_1m_usd,
        cache_write_1m_usd: form.cache_write_1m_usd,
        cache_read_1m_usd: form.cache_read_1m_usd,
      });
    } else {
      await modelsStore.createModel({
        name: form.name,
        input_1m_usd: form.input_1m_usd,
        output_1m_usd: form.output_1m_usd,
        cache_write_1m_usd: form.cache_write_1m_usd,
        cache_read_1m_usd: form.cache_read_1m_usd,
      });
    }
    showDialog.value = false;
    load();
    $q.notify({ type: 'positive', message: editingId.value ? 'Model updated' : 'Model created' });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to save model';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    saving.value = false;
  }
}

async function onDelete(row: Model) {
  $q.dialog({ title: 'Delete Model', message: `Delete "${row.name}"?`, cancel: true })
    .onOk(async () => {
      try {
        await modelsStore.deleteModel(row.id);
        load();
        $q.notify({ type: 'positive', message: 'Model deleted' });
      } catch (e: unknown) {
        const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to delete model';
        $q.notify({ type: 'negative', message: msg });
      }
    });
}

load();
</script>
