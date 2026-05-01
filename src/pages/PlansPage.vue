<template>
  <q-page padding>
    <div class="row items-center q-mb-md">
      <div class="text-h5">Subscription Plans</div>
      <q-space />
      <q-btn color="primary" label="Add Plan" @click="openCreate" />
    </div>

    <div v-if="loading && !plans.length" class="flex flex-center q-pa-lg"><q-spinner size="md" /></div>
    <q-banner v-else-if="!plans.length" class="q-mb-sm" rounded>No subscription plans created</q-banner>
    <q-table
      v-else
      flat bordered dense
      :rows="plans"
      :columns="columns"
      row-key="id"
      :pagination="{ rowsPerPage: 50 }"
    >
      <template #body-cell-model_limits="props">
        <q-td :props="props">
          <span v-if="Object.keys(props.row.model_limits || {}).length === 0">&mdash;</span>
          <span v-else>
            <q-chip v-for="(val, key) in props.row.model_limits" :key="key" dense size="sm">
              {{ key }}: ${{ Number(val).toFixed(2) }}
            </q-chip>
          </span>
        </q-td>
      </template>
      <template #body-cell-model_request_costs="props">
        <q-td :props="props">
          <span v-if="props.row.sub_type !== 'pay_per_request'">&mdash;</span>
          <span v-else-if="Object.keys(props.row.model_request_costs || {}).length === 0">&mdash;</span>
          <span v-else>
            <q-chip v-for="(val, key) in props.row.model_request_costs" :key="key" dense size="sm">
              {{ key }}: ${{ Number(val).toFixed(4) }}
            </q-chip>
          </span>
        </q-td>
      </template>
      <template #body-cell-is_active="props">
        <q-td :props="props">
          <q-toggle :model-value="props.row.is_active" @update:model-value="toggleActive(props.row, $event)" />
        </q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn flat dense icon="edit" @click="openEdit(props.row)" />
          <q-btn flat dense icon="sync" @click="onSyncRpm(props.row)">
            <q-tooltip>Sync RPM to active subscriptions</q-tooltip>
          </q-btn>
          <q-btn flat dense icon="speed" @click="onSyncTpm(props.row)">
            <q-tooltip>Sync TPM to active subscriptions</q-tooltip>
          </q-btn>
          <q-btn flat dense icon="delete" color="negative" @click="onDelete(props.row)" />
        </q-td>
      </template>
    </q-table>
<!-- PLACEHOLDER_DIALOG -->
    <q-dialog v-model="showDialog" @hide="resetForm">
      <q-card style="width: 500px">
        <q-card-section><div class="text-h6">{{ editingId ? 'Edit Plan' : 'Create Plan' }}</div></q-card-section>
        <q-card-section class="q-gutter-sm">
          <q-input v-model="form.name" label="Name" outlined dense />
          <q-select v-model="form.sub_type" :options="getSubTypeOptions()" label="Type" outlined dense emit-value map-options>
            <template #option="scope">
              <q-item v-bind="scope.itemProps">
                <q-item-section>
                  <q-item-label>{{ scope.opt.label }}</q-item-label>
                  <q-item-label caption>{{ scope.opt.tooltip }}</q-item-label>
                </q-item-section>
              </q-item>
            </template>
          </q-select>
          <q-input v-model.number="form.cost_limit_usd" label="Cost Limit ($)" outlined dense type="number" :min="0" />
          <q-input v-model.number="form.weekly_cost_limit_usd" label="Weekly Cost Limit ($)" outlined dense type="number" :min="0" :step="0.01" hint="Empty = unlimited" clearable @clear="form.weekly_cost_limit_usd = null" />
          <q-input v-model.number="form.duration_days" label="Duration (days)" outlined dense type="number" :min="1" />
          <q-input v-if="form.sub_type === 'hourly_reset' || form.sub_type === 'pay_per_request'" v-model.number="form.reset_hours" label="Reset Hours" outlined dense type="number" :min="1" />
          <q-input v-model.number="form.rpm_limit" label="RPM Limit" outlined dense type="number" :min="0" :step="0.1" hint="Requests per minute (empty = unlimited)" clearable @clear="form.rpm_limit = null" />
          <q-input v-model.number="form.tpm_limit" label="TPM Limit" outlined dense type="number" :min="0" :step="1" hint="Tokens per minute (empty = unlimited)" clearable @clear="form.tpm_limit = null" />
          <div class="text-subtitle2 q-mt-sm">Model Limits</div>
          <div v-for="(entry, idx) in modelLimitEntries" :key="idx" class="row q-gutter-sm q-mb-xs">
            <q-select v-model="entry.model" :options="availableModels" label="Model" outlined dense emit-value map-options style="flex:2" />
            <q-input v-model.number="entry.limit" label="$ Limit" outlined dense type="number" :min="0" style="flex:1" />
            <q-btn flat dense icon="close" @click="modelLimitEntries.splice(idx, 1)" />
          </div>
          <q-btn flat dense icon="add" label="Add model limit" @click="modelLimitEntries.push({ model: '', limit: 0 })" />
          <template v-if="form.sub_type === 'pay_per_request'">
            <div class="text-subtitle2 q-mt-sm">Model Request Costs</div>
            <div v-for="(entry, idx) in modelRequestCostEntries" :key="idx" class="row q-gutter-sm q-mb-xs">
              <q-select v-model="entry.model" :options="availableModels" label="Model" outlined dense emit-value map-options style="flex:2" />
              <q-input v-model.number="entry.cost" label="$ per request" outlined dense type="number" :min="0" :step="0.01" style="flex:1" />
              <q-btn flat dense icon="close" @click="modelRequestCostEntries.splice(idx, 1)" />
            </div>
            <q-btn flat dense icon="add" label="Add model cost" @click="modelRequestCostEntries.push({ model: '', cost: 0 })" />
          </template>
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
import { ref, reactive, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { api } from 'boot/axios';
import { useModelsStore } from 'stores/models';
import { getSubTypeLabel, getSubTypeOptions } from 'src/composables/useSubscriptionType';

interface Plan {
  id: string;
  name: string;
  sub_type: string;
  cost_limit_usd: number;
  weekly_cost_limit_usd: number | null;
  model_limits: Record<string, number>;
  model_request_costs: Record<string, number>;
  reset_hours: number | null;
  rpm_limit: number | null;
  tpm_limit: number | null;
  duration_days: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

const $q = useQuasar();
const modelsStore = useModelsStore();

const plans = ref<Plan[]>([]);
const loading = ref(false);
const showDialog = ref(false);
const editingId = ref<string | null>(null);
const saving = ref(false);
const availableModels = ref<string[]>([]);
const modelLimitEntries = ref<{ model: string; limit: number }[]>([]);
const modelRequestCostEntries = ref<{ model: string; cost: number }[]>([]);

const form = reactive({
  name: '',
  sub_type: 'fixed' as string,
  cost_limit_usd: 0,
  weekly_cost_limit_usd: null as number | null,
  duration_days: 30,
  reset_hours: null as number | null,
  rpm_limit: null as number | null,
  tpm_limit: null as number | null,
});

const columns = [
  { name: 'name', label: 'Name', field: 'name', align: 'left' as const },
  { name: 'sub_type', label: 'Type', field: 'sub_type', align: 'left' as const, format: (v: string) => getSubTypeLabel(v) },
  { name: 'cost_limit_usd', label: 'Cost Limit', field: 'cost_limit_usd', align: 'right' as const, format: (v: number) => `$${v.toFixed(2)}` },
  { name: 'weekly_cost_limit_usd', label: 'Weekly Cost Limit', field: 'weekly_cost_limit_usd', align: 'right' as const, format: (v: number | null) => v != null ? `$${v.toFixed(2)}` : 'Unlimited' },
  { name: 'rpm_limit', label: 'RPM', field: 'rpm_limit', align: 'right' as const, format: (v: number | null) => v != null ? String(v) : '\u2014' },
  { name: 'tpm_limit', label: 'TPM Limit', field: 'tpm_limit', align: 'right' as const, format: (v: number | null) => v != null ? String(v) : '\u2014' },
  { name: 'model_limits', label: 'Model Limits', field: 'model_limits', align: 'left' as const },
  { name: 'model_request_costs', label: 'Request Costs', field: 'model_request_costs', align: 'left' as const },
  { name: 'reset_hours', label: 'Reset Hours', field: 'reset_hours', align: 'right' as const, format: (v: number | null) => v != null ? String(v) : '\u2014' },
  { name: 'duration_days', label: 'Duration (days)', field: 'duration_days', align: 'right' as const },
  { name: 'is_active', label: 'Active', field: 'is_active', align: 'center' as const },
  { name: 'actions', label: 'Actions', field: 'id', align: 'right' as const },
];

// PLACEHOLDER_FUNCTIONS

async function load() {
  loading.value = true;
  try {
    const { data } = await api.get<Plan[]>('/api/admin/subscription-plans');
    plans.value = data;
  } catch {
    $q.notify({ type: 'negative', message: 'Failed to load plans' });
  } finally {
    loading.value = false;
  }
}

async function loadModels() {
  try {
    const result = await modelsStore.fetchModels({ limit: 1000 });
    availableModels.value = result.data.map((m) => m.name);
  } catch { /* ignore */ }
}

function openCreate() {
  editingId.value = null;
  resetForm();
  showDialog.value = true;
}

function openEdit(row: Plan) {
  editingId.value = row.id;
  form.name = row.name;
  form.sub_type = row.sub_type;
  form.cost_limit_usd = row.cost_limit_usd;
  form.weekly_cost_limit_usd = row.weekly_cost_limit_usd;
  form.duration_days = row.duration_days;
  form.reset_hours = row.reset_hours;
  form.rpm_limit = row.rpm_limit;
  form.tpm_limit = row.tpm_limit;
  modelLimitEntries.value = Object.entries(row.model_limits || {}).map(([model, limit]) => ({ model, limit: limit as number }));
  modelRequestCostEntries.value = Object.entries(row.model_request_costs || {}).map(([model, cost]) => ({ model, cost: cost as number }));
  showDialog.value = true;
}

function resetForm() {
  form.name = '';
  form.sub_type = 'fixed';
  form.cost_limit_usd = 0;
  form.weekly_cost_limit_usd = null;
  form.duration_days = 30;
  form.reset_hours = null;
  form.rpm_limit = null;
  form.tpm_limit = null;
  modelLimitEntries.value = [];
  modelRequestCostEntries.value = [];
}

function buildModelLimits(): Record<string, number> {
  const limits: Record<string, number> = {};
  for (const e of modelLimitEntries.value) {
    if (e.model) limits[e.model] = e.limit;
  }
  return limits;
}

function buildModelRequestCosts(): Record<string, number> {
  const costs: Record<string, number> = {};
  for (const e of modelRequestCostEntries.value) {
    if (e.model) costs[e.model] = e.cost;
  }
  return costs;
}

async function onSave() {
  if (!form.name.trim()) {
    $q.notify({ type: 'negative', message: 'Name is required' });
    return;
  }
  if (form.sub_type === 'pay_per_request') {
    const costs = buildModelRequestCosts();
    if (Object.keys(costs).length === 0) {
      $q.notify({ type: 'negative', message: 'At least one model request cost is required for Pay Per Request plans' });
      return;
    }
  }
  saving.value = true;
  try {
    const payload = {
      name: form.name,
      sub_type: form.sub_type,
      cost_limit_usd: form.cost_limit_usd,
      weekly_cost_limit_usd: form.weekly_cost_limit_usd != null && Number.isFinite(form.weekly_cost_limit_usd) ? form.weekly_cost_limit_usd : null,
      duration_days: form.duration_days,
      reset_hours: (form.sub_type === 'hourly_reset' || form.sub_type === 'pay_per_request') ? form.reset_hours : null,
      rpm_limit: form.rpm_limit && Number.isFinite(form.rpm_limit) ? form.rpm_limit : null,
      tpm_limit: form.tpm_limit && Number.isFinite(form.tpm_limit) ? form.tpm_limit : null,
      model_limits: buildModelLimits(),
      model_request_costs: buildModelRequestCosts(),
    };
    if (editingId.value) {
      await api.patch(`/api/admin/subscription-plans/${editingId.value}`, payload);
    } else {
      await api.post('/api/admin/subscription-plans', payload);
    }
    showDialog.value = false;
    load();
    $q.notify({ type: 'positive', message: editingId.value ? 'Plan updated' : 'Plan created' });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to save';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    saving.value = false;
  }
}

async function toggleActive(row: Plan, val: boolean) {
  try {
    await api.patch(`/api/admin/subscription-plans/${row.id}`, { is_active: val });
    row.is_active = val;
  } catch {
    $q.notify({ type: 'negative', message: 'Failed to update' });
  }
}

async function onDelete(row: Plan) {
  $q.dialog({ title: 'Delete Plan', message: `Delete "${row.name}"?`, cancel: true })
    .onOk(async () => {
      try {
        await api.delete(`/api/admin/subscription-plans/${row.id}`);
        load();
        $q.notify({ type: 'positive', message: 'Plan deleted' });
      } catch (e: unknown) {
        const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to delete';
        $q.notify({ type: 'negative', message: msg });
      }
    });
}

async function onSyncRpm(row: Plan) {
  $q.dialog({
    title: 'Sync RPM',
    message: `Sync RPM limit (${row.rpm_limit != null ? `${row.rpm_limit} RPM` : 'unlimited'}) to all active subscriptions of "${row.name}"?`,
    cancel: true,
  }).onOk(async () => {
    try {
      const { data } = await api.post<{ updated: number }>(`/api/admin/subscription-plans/${row.id}/sync-rpm`);
      $q.notify({ type: 'positive', message: `RPM synced to ${data.updated} subscription(s)` });
    } catch (e: unknown) {
      const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to sync';
      $q.notify({ type: 'negative', message: msg });
    }
  });
}

async function onSyncTpm(row: Plan) {
  $q.dialog({
    title: 'Sync TPM',
    message: `Sync TPM limit (${row.tpm_limit != null ? `${row.tpm_limit} TPM` : 'unlimited'}) to all active subscriptions of "${row.name}"?`,
    cancel: true,
  }).onOk(async () => {
    try {
      const { data } = await api.post<{ updated: number }>(`/api/admin/subscription-plans/${row.id}/sync-tpm`);
      await load();
      $q.notify({ type: 'positive', message: `TPM synced to ${data.updated} subscription(s)` });
    } catch (e: unknown) {
      const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to sync';
      $q.notify({ type: 'negative', message: msg });
    }
  });
}

onMounted(() => {
  load();
  loadModels();
});
</script>
