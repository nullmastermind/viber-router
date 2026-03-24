<template>
  <q-page padding>
    <div class="text-h5 q-mb-md">Proxy Error Logs</div>

    <div class="row q-col-gutter-sm q-mb-md">
      <div class="col-auto">
        <q-select
          v-model="filters.status_code"
          :options="statusOptions"
          label="Status Code"
          outlined dense clearable emit-value map-options
          style="min-width: 140px"
          @update:model-value="onFilterChange"
        />
      </div>
      <div class="col-auto">
        <q-select
          v-model="filters.group_id"
          :options="groupOptions"
          label="Group"
          outlined dense clearable emit-value map-options
          style="min-width: 180px"
          @update:model-value="onFilterChange"
        />
      </div>
      <div class="col-auto">
        <q-select
          v-model="filters.server_id"
          :options="serverOptions"
          label="Server"
          outlined dense clearable emit-value map-options
          style="min-width: 180px"
          @update:model-value="onFilterChange"
        />
      </div>
      <div class="col-auto">
        <q-select
          v-model="filters.error_type"
          :options="errorTypeOptions"
          label="Error Type"
          outlined dense clearable emit-value map-options
          style="min-width: 180px"
          @update:model-value="onFilterChange"
        />
      </div>
      <div class="col-auto">
        <q-input
          :model-value="fromDate"
          label="From"
          outlined dense readonly
          style="min-width: 180px; max-width: 180px"
        >
          <template #append>
            <q-icon v-if="fromDate" name="cancel" class="cursor-pointer text-grey-5" @click="clearFrom" />
            <q-icon name="event" class="cursor-pointer">
              <q-popup-proxy ref="fromPopup" transition-show="scale" transition-hide="scale">
                <q-date :model-value="fromDateModel" @update:model-value="onFromDatePick" />
              </q-popup-proxy>
            </q-icon>
          </template>
        </q-input>
      </div>
      <div v-if="fromDate" class="col-auto">
        <q-input
          v-model="fromTimeInput"
          label="Time"
          outlined dense
          mask="##:##"
          placeholder="00:00"
          style="min-width: 80px; max-width: 80px"
          @update:model-value="onFromTimeChange"
        />
      </div>
      <div class="col-auto">
        <q-input
          :model-value="toDate"
          label="To"
          outlined dense readonly
          style="min-width: 180px; max-width: 180px"
        >
          <template #append>
            <q-icon v-if="toDate" name="cancel" class="cursor-pointer text-grey-5" @click="clearTo" />
            <q-icon name="event" class="cursor-pointer">
              <q-popup-proxy ref="toPopup" transition-show="scale" transition-hide="scale">
                <q-date :model-value="toDateModel" @update:model-value="onToDatePick" />
              </q-popup-proxy>
            </q-icon>
          </template>
        </q-input>
      </div>
      <div v-if="toDate" class="col-auto">
        <q-input
          v-model="toTimeInput"
          label="Time"
          outlined dense
          mask="##:##"
          placeholder="23:59"
          style="min-width: 80px; max-width: 80px"
          @update:model-value="onToTimeChange"
        />
      </div>
      <div class="col-auto">
        <q-input
          v-model="apiKeySearch"
          label="API Key"
          outlined dense clearable
          style="min-width: 200px"
          @keyup.enter="onApiKeySearch"
          @clear="onApiKeySearch"
        >
          <template #append>
            <q-icon name="search" class="cursor-pointer" @click="onApiKeySearch" />
          </template>
        </q-input>
      </div>
      <div v-if="hasActiveFilters" class="col-auto self-center">
        <q-btn flat dense no-caps color="grey" icon="clear_all" label="Clear all" @click="clearAllFilters" />
      </div>
    </div>

    <q-banner v-if="error" class="bg-negative text-white q-mb-md">
      Failed to load logs
      <template #action>
        <q-btn flat label="Retry" @click="() => fetchLogs(pagination.page, pagination.rowsPerPage)" />
      </template>
    </q-banner>

    <q-table
      :rows="logs"
      :columns="columns"
      row-key="id"
      :loading="loading"
      flat bordered
      v-model:pagination="pagination"
      @request="onRequest"
    >
      <template #body="props">
        <q-tr :props="props" class="cursor-pointer" @click="props.expand = !props.expand">
          <q-td v-for="col in props.cols" :key="col.name" :props="props">
            <template v-if="col.name === 'status_code'">
              <span v-if="props.row.failover_chain.length > 1" class="status-chain">
                <template v-for="(attempt, i) in props.row.failover_chain" :key="i">
                  <q-badge :color="statusColor(attempt.status)" :label="attempt.status === 0 ? 'TTFT' : String(attempt.status)" />
                  <q-icon v-if="Number(i) < props.row.failover_chain.length - 1" name="arrow_forward" size="xs" class="q-mx-xs text-grey" />
                </template>
              </span>
              <q-badge v-else :color="statusColor(props.row.status_code)" :label="String(props.row.status_code)" />
            </template>
            <template v-else-if="col.name === 'error_type'">
              <q-badge
                :color="errorTypeBadge(props.row.error_type).color"
                :label="errorTypeBadge(props.row.error_type).label"
                outline
              />
            </template>
            <template v-else-if="col.name === 'server_name'">
              <span v-if="props.row.failover_chain.length > 1" class="server-chain">
                <template v-for="(attempt, i) in props.row.failover_chain" :key="i">
                  <span :class="attempt.status >= 200 && attempt.status < 400 ? 'text-positive' : 'text-negative'">{{ attempt.server_name }}</span>
                  <q-icon v-if="Number(i) < props.row.failover_chain.length - 1" name="arrow_forward" size="xs" class="q-mx-xs text-grey" />
                </template>
              </span>
              <span v-else>{{ props.row.server_name }}</span>
            </template>
            <template v-else-if="col.name === 'latency_ms'">
              {{ props.row.latency_ms }}ms
            </template>
            <template v-else-if="col.name === 'created_at'">
              {{ formatDate(props.row.created_at) }}
            </template>
            <template v-else>
              {{ col.value }}
            </template>
          </q-td>
        </q-tr>
        <q-tr v-show="props.expand" :props="props">
          <q-td colspan="100%">
            <div class="q-pa-md">
              <div class="row q-col-gutter-md">
                <div class="col-12 col-md-6">
                  <div class="text-subtitle2 q-mb-sm">Request Details</div>
                  <div><span class="text-weight-medium">Path:</span> {{ props.row.request_method }} {{ props.row.request_path }}</div>
                  <div><span class="text-weight-medium">API Key:</span> <code>{{ props.row.group_api_key }}</code></div>
                  <div><span class="text-weight-medium">Model:</span> {{ props.row.request_model || 'N/A' }}</div>
                  <div><span class="text-weight-medium">Error Type:</span> {{ props.row.error_type }}</div>
                  <q-btn
                    v-if="props.row.failover_chain.length === 1 && props.row.failover_chain[0].upstream_url"
                    flat dense no-caps
                    icon="download"
                    label="Download cURL"
                    color="primary"
                    class="q-mt-sm"
                    @click.stop="downloadCurl(props.row.failover_chain[0], props.row)"
                  />
                </div>
                <div class="col-12 col-md-6">
                  <div class="text-subtitle2 q-mb-sm">Failover Chain</div>
                  <div v-if="props.row.failover_chain.length === 0" class="text-grey">No failover data</div>
                  <div v-else class="failover-timeline">
                    <div v-for="(attempt, i) in props.row.failover_chain" :key="i" class="failover-step">
                      <div class="failover-dot" :class="attemptClass(attempt)" />
                      <div class="failover-line" v-if="Number(i) < props.row.failover_chain.length - 1" />
                      <div class="failover-info">
                        <div class="text-weight-medium">
                          {{ attempt.server_name }}
                          <q-badge
                            :color="attempt.status === 0 ? 'grey' : attempt.status >= 200 && attempt.status < 400 ? 'positive' : 'negative'"
                            :label="attempt.status === 0 ? 'TTFT skip' : String(attempt.status)"
                            class="q-ml-sm"
                          />
                          <q-icon v-if="attempt.status >= 200 && attempt.status < 400" name="check_circle" color="positive" size="xs" class="q-ml-xs" />
                          <q-btn
                            v-if="attempt.upstream_url && !(attempt.status >= 200 && attempt.status < 400)"
                            flat dense no-caps size="sm"
                            icon="download"
                            label="cURL"
                            color="primary"
                            class="q-ml-sm"
                            @click.stop="downloadCurl(attempt, props.row)"
                          />
                        </div>
                        <div class="text-caption text-grey">{{ attempt.latency_ms }}ms</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </q-td>
        </q-tr>
      </template>

      <template #no-data>
        <div class="full-width text-center q-pa-lg text-grey">
          No logs matching filters. Try adjusting your filter criteria.
        </div>
      </template>
    </q-table>

  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue';
import type { QPopupProxy } from 'quasar';
import { api } from 'boot/axios';

interface FailoverAttempt {
  server_id: string;
  server_name: string;
  status: number;
  latency_ms: number;
  upstream_url?: string;
  request_headers?: Record<string, string>;
  request_body?: Record<string, unknown>;
}

interface ProxyLog {
  id: string;
  created_at: string;
  group_id: string;
  group_api_key: string;
  server_id: string;
  server_name: string;
  request_path: string;
  request_method: string;
  status_code: number;
  error_type: string;
  latency_ms: number;
  failover_chain: FailoverAttempt[];
  request_model: string | null;
  request_body: Record<string, unknown> | null;
  request_headers: Record<string, string> | null;
  upstream_url: string | null;
}

interface LogListResponse {
  data: ProxyLog[];
  total: number;
}

const logs = ref<ProxyLog[]>([]);
const loading = ref(false);
const error = ref(false);
const apiKeySearch = ref('');
const fromPopup = ref<InstanceType<typeof QPopupProxy>>();
const toPopup = ref<InstanceType<typeof QPopupProxy>>();

const fromDateModel = computed(() =>
  filters.from ? filters.from.slice(0, 10).replace(/-/g, '/') : null,
);
const toDateModel = computed(() =>
  filters.to ? filters.to.slice(0, 10).replace(/-/g, '/') : null,
);
const fromDate = computed(() =>
  filters.from ? filters.from.slice(0, 10) : '',
);
const fromTimeInput = ref('');
const toDate = computed(() =>
  filters.to ? filters.to.slice(0, 10) : '',
);
const toTimeInput = ref('');

const hasActiveFilters = computed(() =>
  filters.status_code !== null ||
  filters.group_id !== null ||
  filters.server_id !== null ||
  filters.error_type !== null ||
  filters.from !== null ||
  filters.to !== null ||
  apiKeySearch.value !== '',
);

const pagination = ref({
  page: 1,
  rowsPerPage: 10,
  rowsNumber: 0,
});

const filters = reactive({
  status_code: null as number | null,
  group_id: null as string | null,
  server_id: null as string | null,
  error_type: null as string | null,
  from: null as string | null,
  to: null as string | null,
});

const groupOptions = ref<{ label: string; value: string }[]>([]);
const serverOptions = ref<{ label: string; value: string }[]>([]);

const statusOptions = [
  { label: '400', value: 400 },
  { label: '401', value: 401 },
  { label: '403', value: 403 },
  { label: '429', value: 429 },
  { label: '500', value: 500 },
  { label: '502', value: 502 },
  { label: '503', value: 503 },
];

const errorTypeOptions = [
  { label: 'Upstream Error', value: 'upstream_error' },
  { label: 'Failover Success', value: 'failover_success' },
  { label: 'All Servers Exhausted', value: 'all_servers_exhausted' },
  { label: 'Connection Error', value: 'connection_error' },
];

const columns = [
  { name: 'created_at', label: 'Time', field: 'created_at', align: 'left' as const },
  { name: 'status_code', label: 'Status', field: 'status_code', align: 'left' as const },
  { name: 'error_type', label: 'Type', field: 'error_type', align: 'left' as const },
  { name: 'server_name', label: 'Server', field: (row: ProxyLog) => {
    if (row.failover_chain.length > 1) {
      return row.failover_chain.map(a => a.server_name).join(' → ');
    }
    return row.server_name;
  }, align: 'left' as const },
  { name: 'request_model', label: 'Model', field: (row: ProxyLog) => row.request_model || '-', align: 'left' as const },
  { name: 'latency_ms', label: 'Latency', field: 'latency_ms', align: 'right' as const },
];

function statusColor(code: number): string {
  if (code === 0) return 'negative';
  if (code >= 500) return 'negative';
  if (code >= 400) return 'warning';
  return 'positive';
}

function errorTypeBadge(type: string): { color: string; label: string } {
  switch (type) {
    case 'failover_success': return { color: 'amber', label: 'Failover ✓' };
    case 'all_servers_exhausted': return { color: 'negative', label: 'All Failed' };
    case 'connection_error': return { color: 'grey', label: 'Conn Error' };
    default: return { color: 'warning', label: 'Upstream Error' };
  }
}

function attemptClass(attempt: FailoverAttempt): string {
  if (attempt.status === 0) return 'bg-grey';
  if (attempt.status >= 200 && attempt.status < 400) return 'bg-positive';
  return 'bg-negative';
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString();
}

function buildParams(page: number, rowsPerPage: number) {
  const params: Record<string, string | number> = {};
  if (filters.status_code) params.status_code = filters.status_code;
  if (filters.group_id) params.group_id = filters.group_id;
  if (filters.server_id) params.server_id = filters.server_id;
  if (filters.error_type) params.error_type = filters.error_type;
  if (filters.from) params.from = new Date(filters.from).toISOString();
  if (filters.to) params.to = new Date(filters.to).toISOString();
  if (apiKeySearch.value) params.api_key = apiKeySearch.value;
  params.page = page;
  params.page_size = rowsPerPage;
  return params;
}

async function fetchLogs(page = 1, rowsPerPage = 10) {
  loading.value = true;
  error.value = false;
  try {
    const { data } = await api.get<LogListResponse>('/api/admin/logs', {
      params: buildParams(page, rowsPerPage),
    });
    logs.value = data.data;
    pagination.value.rowsNumber = data.total;
    pagination.value.page = page;
    pagination.value.rowsPerPage = rowsPerPage;
  } catch {
    error.value = true;
  } finally {
    loading.value = false;
  }
}

function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  fetchLogs(props.pagination.page, props.pagination.rowsPerPage);
}

function onFilterChange() {
  fetchLogs(1, pagination.value.rowsPerPage);
}

function onFromDatePick(val: string) {
  const date = val.replace(/\//g, '-');
  if (!fromTimeInput.value) fromTimeInput.value = '00:00';
  filters.from = `${date}T${fromTimeInput.value}`;
  fromPopup.value?.hide();
  onFilterChange();
}

function onToDatePick(val: string) {
  const date = val.replace(/\//g, '-');
  if (!toTimeInput.value) toTimeInput.value = '23:59';
  filters.to = `${date}T${toTimeInput.value}`;
  toPopup.value?.hide();
  onFilterChange();
}

function onFromTimeChange(val: string | number | null) {
  if (typeof val === 'string' && val.length === 5 && filters.from) {
    filters.from = `${fromDate.value}T${val}`;
    onFilterChange();
  }
}

function onToTimeChange(val: string | number | null) {
  if (typeof val === 'string' && val.length === 5 && filters.to) {
    filters.to = `${toDate.value}T${val}`;
    onFilterChange();
  }
}

function clearFrom() {
  filters.from = null;
  fromTimeInput.value = '';
  onFilterChange();
}

function clearTo() {
  filters.to = null;
  toTimeInput.value = '';
  onFilterChange();
}

function clearAllFilters() {
  filters.status_code = null;
  filters.group_id = null;
  filters.server_id = null;
  filters.error_type = null;
  filters.from = null;
  filters.to = null;
  fromTimeInput.value = '';
  toTimeInput.value = '';
  apiKeySearch.value = '';
  onFilterChange();
}

function onApiKeySearch() {
  fetchLogs(1, pagination.value.rowsPerPage);
}

function shellEscape(s: string): string {
  return `'${s.replace(/'/g, "'\\''")}'`;
}

function generateCurl(attempt: FailoverAttempt, method: string): string {
  const parts: string[] = ['curl'];

  if (method !== 'GET') {
    parts.push(`-X ${method}`);
  }

  parts.push(shellEscape(attempt.upstream_url ?? ''));

  if (attempt.request_headers) {
    for (const [name, value] of Object.entries(attempt.request_headers)) {
      parts.push(`-H ${shellEscape(`${name}: ${value}`)}`);
    }
  }

  if (attempt.request_body) {
    parts.push(`-d ${shellEscape(JSON.stringify(attempt.request_body))}`);
  }

  return parts.join(' \\\n  ');
}

function downloadCurl(attempt: FailoverAttempt, log: ProxyLog) {
  const curl = generateCurl(attempt, log.request_method);
  const blob = new Blob([`${curl}\n`], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `curl-${attempt.server_name}-${log.id.slice(0, 8)}.txt`;
  a.click();
  URL.revokeObjectURL(url);
}

async function loadFilterOptions() {
  try {
    const [groups, servers] = await Promise.all([
      api.get('/api/admin/groups', { params: { limit: 100 } }),
      api.get('/api/admin/servers', { params: { limit: 100 } }),
    ]);
    groupOptions.value = groups.data.data.map((g: { id: string; name: string }) => ({
      label: g.name,
      value: g.id,
    }));
    serverOptions.value = servers.data.data.map((s: { id: string; name: string }) => ({
      label: s.name,
      value: s.id,
    }));
  } catch {
    // Filter options are non-critical
  }
}

onMounted(() => {
  fetchLogs();
  loadFilterOptions();
});
</script>

<style scoped>
.failover-timeline {
  display: flex;
  flex-direction: column;
  gap: 0;
}
.failover-step {
  display: flex;
  align-items: flex-start;
  position: relative;
  padding-bottom: 8px;
}
.failover-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  margin-top: 5px;
  margin-right: 10px;
  flex-shrink: 0;
}
.failover-line {
  position: absolute;
  left: 4px;
  top: 15px;
  bottom: 0;
  width: 2px;
  background: #ccc;
}
.server-chain {
  display: inline-flex;
  align-items: center;
}
</style>
