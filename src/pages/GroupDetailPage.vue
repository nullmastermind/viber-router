<template>
  <q-page padding>
    <div v-if="group" class="q-gutter-md">
      <div class="row items-center">
        <q-btn flat icon="arrow_back" @click="$router.push('/groups')" />
        <div class="text-h5 q-ml-sm">{{ group.name }}</div>
        <q-space />
        <q-toggle v-model="group.is_active" label="Active" @update:model-value="saveGroup" />
      </div>

      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">Properties</div>
          <q-input v-model="group.name" label="Name" outlined dense class="q-mb-sm" />
          <q-input v-model="failoverCodesStr" label="Failover Status Codes (comma-separated)" outlined dense class="q-mb-sm" />
          <q-input v-model="ttftTimeoutStr" label="TTFT Timeout (ms, empty = disabled)" outlined dense type="number" class="q-mb-sm" hint="Auto-switch to next server if first token takes longer" />
          <q-btn color="primary" label="Save" @click="saveGroup" />
        </q-card-section>
      </q-card>

      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">Count Tokens Default Server</div>
          <q-select
            v-model="group.count_tokens_server_id"
            :options="allServers"
            label="Default server for /v1/messages/count_tokens"
            outlined
            dense
            emit-value
            map-options
            clearable
            class="q-mb-sm"
            @update:model-value="saveGroup"
          />
          <div class="text-subtitle2 q-mb-xs q-mt-md">Count Tokens Model Mappings</div>
          <div v-for="(entry, idx) in ctMappingEntries" :key="idx" class="row q-gutter-sm q-mb-sm">
            <q-input v-model="entry.from" label="From model" outlined dense style="flex:1" />
            <q-input v-model="entry.to" label="To model" outlined dense style="flex:1" />
            <q-btn flat dense icon="close" @click="ctMappingEntries.splice(idx, 1)" />
          </div>
          <div class="row q-gutter-sm">
            <q-btn flat dense icon="add" label="Add mapping" @click="ctMappingEntries.push({ from: '', to: '' })" />
            <q-btn color="primary" label="Save Mappings" dense @click="saveCtMappings" />
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">API Key</div>
          <div class="row items-center q-gutter-sm">
            <code class="text-body1">{{ group.api_key }}</code>
            <q-btn flat dense icon="content_copy" @click="copyKey" />
            <q-btn flat dense color="warning" label="Regenerate" @click="onRegenerate" />
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered>
        <q-card-section>
          <div class="row items-center q-mb-sm">
            <div class="text-subtitle1">Key Builder</div>
            <q-space />
            <q-toggle v-model="showAllKeyBuilderServers" label="Show all servers" dense />
          </div>
          <div v-for="entry in visibleKeyBuilderEntries" :key="entry.server_id" class="row items-center q-gutter-sm q-mb-sm">
            <span class="text-body2" style="min-width: 160px">{{ entry.server_name }} (#{{ entry.short_id }})</span>
            <q-input v-model="entry.key" :placeholder="entry.defaultKey || 'API Key'" outlined dense style="flex: 1" />
          </div>
          <div v-if="visibleKeyBuilderEntries.length === 0" class="text-grey q-mb-sm">No servers without predefined key</div>
          <div v-if="builtKey" class="q-mt-sm">
            <div class="text-caption q-mb-xs">Dynamic Key</div>
            <div class="row items-center q-gutter-sm">
              <code class="text-body2" style="word-break: break-all">{{ builtKey }}</code>
              <q-btn flat dense icon="content_copy" @click="copyText(builtKey)" />
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered>
        <q-card-section>
          <div class="row items-center q-mb-sm">
            <div class="text-subtitle1">Servers (priority order)</div>
            <q-space />
            <q-btn flat dense icon="add" label="Add Server" @click="showAddServer = true" />
          </div>
          <q-list bordered separator>
            <q-item v-for="(s, idx) in servers" :key="s.server_id">
              <q-item-section avatar>
                <div class="column items-center">
                  <q-btn flat dense icon="arrow_upward" :disable="idx === 0" @click="moveServer(idx, -1)" />
                  <span class="text-caption">{{ s.priority }}</span>
                  <q-btn flat dense icon="arrow_downward" :disable="idx === servers.length - 1" @click="moveServer(idx, 1)" />
                </div>
              </q-item-section>
              <q-item-section>
                <q-item-label>
                  {{ s.server_name }}
                  <q-badge outline class="q-ml-sm">
                    #{{ s.short_id }}
                    <q-btn flat dense size="xs" icon="content_copy" class="q-ml-xs" @click.stop="copyShortId(s.short_id)" />
                  </q-badge>
                </q-item-label>
                <q-item-label caption>{{ s.base_url }}</q-item-label>
              </q-item-section>
              <q-item-section side>
                <div class="row q-gutter-xs">
                  <q-btn flat dense icon="edit" @click="openEditServer(s)" />
                  <q-btn flat dense icon="tune" @click="editMappings(s)" />
                  <q-btn flat dense icon="delete" color="negative" @click="onRemoveServer(s)" />
                </div>
              </q-item-section>
            </q-item>
            <q-item v-if="servers.length === 0">
              <q-item-section class="text-grey">No servers assigned</q-item-section>
            </q-item>
          </q-list>
        </q-card-section>
      </q-card>

      <q-card flat bordered>
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">TTFT (Time to First Token) — Last Hour</div>
          <div v-if="ttftLoading && !ttftStats" class="flex flex-center q-pa-lg">
            <q-spinner size="md" />
          </div>
          <q-banner v-else-if="ttftError" class="bg-negative text-white q-mb-sm" rounded>
            {{ ttftError }}
          </q-banner>
          <div v-else-if="ttftChartData" style="height: 300px">
            <Line :data="ttftChartData" :options="ttftChartOptions" />
          </div>
          <div v-else class="text-grey text-center q-pa-lg">
            No TTFT data in the last hour
          </div>
          <div v-if="ttftStats && ttftStats.servers.length > 0" class="q-mt-md">
            <q-markup-table flat bordered dense>
              <thead>
                <tr>
                  <th class="text-left">Server</th>
                  <th class="text-right">Avg</th>
                  <th class="text-right">P50</th>
                  <th class="text-right">P95</th>
                  <th class="text-right">Timeouts</th>
                  <th class="text-right">Total</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="s in ttftStats.servers" :key="s.server_id">
                  <td>{{ s.server_name }}</td>
                  <td class="text-right">{{ s.avg_ttft_ms != null ? `${Math.round(s.avg_ttft_ms)}ms` : '—' }}</td>
                  <td class="text-right">{{ s.p50_ttft_ms != null ? `${Math.round(s.p50_ttft_ms)}ms` : '—' }}</td>
                  <td class="text-right">{{ s.p95_ttft_ms != null ? `${Math.round(s.p95_ttft_ms)}ms` : '—' }}</td>
                  <td class="text-right">{{ s.timeout_count }}</td>
                  <td class="text-right">{{ s.total_count }}</td>
                </tr>
              </tbody>
            </q-markup-table>
          </div>
        </q-card-section>
      </q-card>

      <q-dialog v-model="showAddServer" @hide="resetAddForm">
        <q-card style="width: 400px">
          <q-card-section><div class="text-h6">Add Server</div></q-card-section>
          <q-card-section>
            <q-select
              v-model="addForm.server_id"
              :options="addServerOptions"
              label="Server"
              outlined
              emit-value
              map-options
              class="q-mb-sm"
            />
            <template v-if="isCreatingNew">
              <q-input v-model="newServerForm.name" label="Name" outlined class="q-mb-sm" />
              <q-input v-model="newServerForm.base_url" label="Base URL" outlined class="q-mb-sm" />
              <q-input v-model="newServerForm.api_key" label="API Key (optional)" outlined class="q-mb-sm" />
            </template>
            <q-input v-model.number="addForm.priority" label="Priority" type="number" outlined />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="Cancel" v-close-popup />
            <q-btn color="primary" label="Add" :loading="addingServer" @click="onAddServer" />
          </q-card-actions>
        </q-card>
      </q-dialog>

      <q-dialog v-model="showMappings">
        <q-card style="width: 500px">
          <q-card-section><div class="text-h6">Model Mappings — {{ editingMapping?.server_name }}</div></q-card-section>
          <q-card-section>
            <div v-for="(entry, idx) in mappingEntries" :key="idx" class="row q-gutter-sm q-mb-sm">
              <q-input v-model="entry.from" label="From model" outlined dense style="flex:1" />
              <q-input v-model="entry.to" label="To model" outlined dense style="flex:1" />
              <q-btn flat dense icon="close" @click="mappingEntries.splice(idx, 1)" />
            </div>
            <q-btn flat dense icon="add" label="Add mapping" @click="mappingEntries.push({ from: '', to: '' })" />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="Cancel" v-close-popup />
            <q-btn color="primary" label="Save" @click="saveMappings" />
          </q-card-actions>
        </q-card>
      </q-dialog>

      <q-dialog v-model="showEditServer">
        <q-card style="width: 400px">
          <q-card-section><div class="text-h6">Edit Server</div></q-card-section>
          <q-card-section>
            <q-input v-model="editServerForm.name" label="Name" outlined class="q-mb-sm" />
            <q-input v-model="editServerForm.base_url" label="Base URL" outlined class="q-mb-sm" />
            <q-input v-model="editServerForm.api_key" label="API Key (optional)" outlined />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="Cancel" v-close-popup />
            <q-btn color="primary" label="Save" :loading="savingServer" @click="onSaveEditServer" />
          </q-card-actions>
        </q-card>
      </q-dialog>
    </div>
    <div v-else class="flex flex-center" style="min-height: 200px">
      <q-spinner size="lg" />
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useRoute } from 'vue-router';
import { useQuasar, copyToClipboard } from 'quasar';
import { useGroupsStore, type GroupWithServers, type GroupServerDetail, type TtftStatsResponse } from 'stores/groups';
import { useServersStore } from 'stores/servers';
import { Line } from 'vue-chartjs';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  TimeScale,
} from 'chart.js';

import type { TooltipItem } from 'chart.js';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, TimeScale);

const CHART_COLORS = ['#1976D2', '#26A69A', '#FF6F00', '#AB47BC', '#EF5350', '#66BB6A', '#42A5F5', '#FFA726'];

const $q = useQuasar();
const route = useRoute();
const groupsStore = useGroupsStore();
const serversStore = useServersStore();

const group = ref<GroupWithServers | null>(null);
const servers = ref<GroupServerDetail[]>([]);
const failoverCodesStr = ref('');

const CREATE_NEW = '__create_new__';
const showAddServer = ref(false);
const addForm = ref({ server_id: '', priority: 1 });
const newServerForm = ref({ name: '', base_url: '', api_key: '' });
const addingServer = ref(false);
const isCreatingNew = computed(() => addForm.value.server_id === CREATE_NEW);
const addServerOptions = computed(() => [
  ...availableServers.value,
  { label: '＋ Create new server...', value: CREATE_NEW },
]);
const showMappings = ref(false);
const editingMapping = ref<GroupServerDetail | null>(null);
const mappingEntries = ref<{ from: string; to: string }[]>([]);

const showEditServer = ref(false);
const editServerId = ref('');
const editServerForm = ref({ name: '', base_url: '', api_key: '' });
const savingServer = ref(false);

const keyBuilderEntries = ref<{ server_id: string; server_name: string; short_id: number; key: string; defaultKey: string }[]>([]);
const showAllKeyBuilderServers = ref(false);

const ttftTimeoutStr = ref('');
const ttftStats = ref<TtftStatsResponse | null>(null);
const ttftLoading = ref(false);
const ttftError = ref('');
let ttftRefreshTimer: ReturnType<typeof setInterval> | null = null;
const ctMappingEntries = ref<{ from: string; to: string }[]>([]);
const visibleKeyBuilderEntries = computed(() =>
  showAllKeyBuilderServers.value
    ? keyBuilderEntries.value
    : keyBuilderEntries.value.filter((e) => !e.defaultKey)
);
const builtKey = computed(() => {
  if (!group.value) return '';
  const parts = [group.value.api_key];
  for (const e of keyBuilderEntries.value) {
    if (e.key) parts.push(`-rsv-${e.short_id}-${e.key}`);
  }
  return parts.length > 1 ? parts.join('') : '';
});

const allServers = ref<{ label: string; value: string }[]>([]);
const availableServers = computed(() =>
  allServers.value.filter((s) => !servers.value.some((gs) => gs.server_id === s.value))
);

onMounted(async () => {
  await loadGroup();
  const sData = await serversStore.fetchServers({ limit: 100 });
  if (sData) {
    allServers.value = sData.data.map((s) => ({ label: `${s.name} (#${s.short_id})`, value: s.id }));
  }
  loadTtftStats();
  ttftRefreshTimer = setInterval(loadTtftStats, 30_000);
});

onUnmounted(() => {
  if (ttftRefreshTimer) {
    clearInterval(ttftRefreshTimer);
    ttftRefreshTimer = null;
  }
});

async function loadGroup() {
  const id = route.params.id as string;
  const data = await groupsStore.getGroup(id);
  group.value = data;
  servers.value = data.servers;
  failoverCodesStr.value = (data.failover_status_codes || []).join(', ');
  ttftTimeoutStr.value = data.ttft_timeout_ms != null ? String(data.ttft_timeout_ms) : '';
  const ctm = data.count_tokens_model_mappings || {};
  ctMappingEntries.value = Object.entries(ctm).map(([from, to]) => ({ from, to }));
  keyBuilderEntries.value = data.servers.map((s) => ({
    server_id: s.server_id,
    server_name: s.server_name,
    short_id: s.short_id,
    key: '',
    defaultKey: s.api_key || '',
  }));
}

async function saveGroup() {
  if (!group.value) return;
  const codes = failoverCodesStr.value
    .split(',')
    .map((s) => parseInt(s.trim(), 10))
    .filter((n) => !Number.isNaN(n));
  const ttftVal = ttftTimeoutStr.value.trim();
  const ttft_timeout_ms = ttftVal === '' ? null : parseInt(ttftVal, 10);
  await groupsStore.updateGroup(group.value.id, {
    name: group.value.name,
    failover_status_codes: codes,
    is_active: group.value.is_active,
    ttft_timeout_ms: Number.isNaN(ttft_timeout_ms as number) ? null : ttft_timeout_ms,
    count_tokens_server_id: group.value.count_tokens_server_id,
  });
  $q.notify({ type: 'positive', message: 'Saved' });
}

async function saveCtMappings() {
  if (!group.value) return;
  const mappings: Record<string, string> = {};
  for (const e of ctMappingEntries.value) {
    if (e.from && e.to) mappings[e.from] = e.to;
  }
  await groupsStore.updateGroup(group.value.id, {
    count_tokens_model_mappings: mappings,
  });
  group.value.count_tokens_model_mappings = mappings;
  $q.notify({ type: 'positive', message: 'Count tokens mappings saved' });
}

function copyKey() {
  if (group.value) {
    copyToClipboard(group.value.api_key).then(() =>
      $q.notify({ message: 'Copied', type: 'positive' })
    );
  }
}

function copyText(text: string) {
  copyToClipboard(text).then(() =>
    $q.notify({ message: 'Copied', type: 'positive' })
  );
}

function copyShortId(shortId: number) {
  copyToClipboard(String(shortId)).then(() =>
    $q.notify({ message: 'Copied', type: 'positive' })
  );
}

async function onRegenerate() {
  if (!group.value) return;
  $q.dialog({ title: 'Regenerate Key', message: 'This will invalidate the current key.', cancel: true })
    .onOk(async () => {
      if (!group.value) return;
      const updated = await groupsStore.regenerateKey(group.value.id);
      group.value.api_key = updated.api_key;
      $q.notify({ type: 'positive', message: 'Key regenerated' });
    });
}

function resetAddForm() {
  addForm.value = { server_id: '', priority: 1 };
  newServerForm.value = { name: '', base_url: '', api_key: '' };
}

async function onAddServer() {
  if (!group.value) return;
  addingServer.value = true;
  try {
    let serverId = addForm.value.server_id;
    if (isCreatingNew.value) {
      const input: { name: string; base_url: string; api_key?: string } = {
        name: newServerForm.value.name,
        base_url: newServerForm.value.base_url,
      };
      if (newServerForm.value.api_key) input.api_key = newServerForm.value.api_key;
      const created = await serversStore.createServer(input);
      serverId = created.id;
      // refresh server list for future use
      const sData = await serversStore.fetchServers({ limit: 100 });
      if (sData) {
        allServers.value = sData.data.map((s) => ({ label: `${s.name} (#${s.short_id})`, value: s.id }));
      }
    }
    await groupsStore.assignServer(group.value.id, { server_id: serverId, priority: addForm.value.priority });
    showAddServer.value = false;
    loadGroup();
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to add server';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    addingServer.value = false;
  }
}

async function onRemoveServer(s: GroupServerDetail) {
  if (!group.value) return;
  $q.dialog({ title: 'Remove Server', message: `Remove "${s.server_name}"?`, cancel: true })
    .onOk(async () => {
      if (!group.value) return;
      await groupsStore.removeServer(group.value.id, s.server_id);
      loadGroup();
    });
}

async function moveServer(idx: number, direction: number) {
  if (!group.value) return;
  const arr = [...servers.value];
  const target = idx + direction;
  const a = arr[idx];
  const b = arr[target];
  if (!a || !b) return;
  arr[idx] = b;
  arr[target] = a;
  await groupsStore.reorderServers(group.value.id, arr.map((s) => s.server_id));
  loadGroup();
}

function editMappings(s: GroupServerDetail) {
  editingMapping.value = s;
  const m = s.model_mappings || {};
  mappingEntries.value = Object.entries(m).map(([from, to]) => ({ from, to }));
  showMappings.value = true;
}

async function saveMappings() {
  if (!group.value || !editingMapping.value) return;
  const mappings: Record<string, string> = {};
  for (const e of mappingEntries.value) {
    if (e.from && e.to) mappings[e.from] = e.to;
  }
  await groupsStore.updateAssignment(group.value.id, editingMapping.value.server_id, { model_mappings: mappings });
  showMappings.value = false;
  loadGroup();
}

function openEditServer(s: GroupServerDetail) {
  editServerId.value = s.server_id;
  editServerForm.value = { name: s.server_name, base_url: s.base_url, api_key: s.api_key || '' };
  showEditServer.value = true;
}

async function onSaveEditServer() {
  savingServer.value = true;
  try {
    await serversStore.updateServer(editServerId.value, {
      name: editServerForm.value.name,
      base_url: editServerForm.value.base_url,
      api_key: editServerForm.value.api_key || null,
    });
    showEditServer.value = false;
    loadGroup();
    // refresh server list for add dialog
    const sData = await serversStore.fetchServers({ limit: 100 });
    if (sData) {
      allServers.value = sData.data.map((s) => ({ label: `${s.name} (#${s.short_id})`, value: s.id }));
    }
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to save server';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    savingServer.value = false;
  }
}

async function loadTtftStats() {
  if (!group.value) return;
  ttftLoading.value = true;
  ttftError.value = '';
  try {
    ttftStats.value = await groupsStore.fetchTtftStats(group.value.id);
  } catch {
    ttftError.value = 'Failed to load TTFT data';
  } finally {
    ttftLoading.value = false;
  }
}

const ttftChartData = computed(() => {
  if (!ttftStats.value || ttftStats.value.servers.length === 0) return null;

  // Build shared label array from all data points
  const allTimes = ttftStats.value.servers
    .flatMap((s) => s.data_points.map((p) => p.created_at))
    .sort();
  const uniqueTimes = [...new Set(allTimes)];

  if (uniqueTimes.length === 0) return null;

  // Build a lookup map: created_at -> data point for each server
  const datasets = ttftStats.value.servers.flatMap((server, idx) => {
    const color = CHART_COLORS[idx % CHART_COLORS.length] as string;

    // Build maps for quick lookup
    const normalMap = new Map<string, number | null>();
    const timeoutSet = new Set<string>();
    for (const p of server.data_points) {
      if (p.timed_out) {
        timeoutSet.add(p.created_at);
      } else if (p.ttft_ms != null) {
        normalMap.set(p.created_at, p.ttft_ms);
      }
    }

    const result: {
      label: string;
      data: (number | null)[];
      borderColor: string;
      backgroundColor: string;
      pointRadius: number;
      tension?: number;
      fill?: boolean;
      pointStyle?: string;
      showLine?: boolean;
      spanGaps?: boolean;
    }[] = [
      {
        label: server.server_name,
        data: uniqueTimes.map((t) => normalMap.get(t) ?? null),
        borderColor: color,
        backgroundColor: color,
        pointRadius: 3,
        tension: 0.2,
        fill: false,
        spanGaps: true,
      },
    ];

    if (timeoutSet.size > 0) {
      result.push({
        label: `${server.server_name} (timeout)`,
        data: uniqueTimes.map((t) => (timeoutSet.has(t) ? 0 : null)),
        borderColor: 'transparent',
        backgroundColor: '#EF5350',
        pointRadius: 6,
        pointStyle: 'crossRot',
        showLine: false,
      });
    }

    return result;
  });

  return { labels: uniqueTimes, datasets };
});

const ttftChartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    x: {
      type: 'category' as const,
      ticks: { maxTicksLimit: 10, callback: function (this: unknown, val: string | number) {
        if (typeof val === 'number' && ttftChartData.value?.labels) {
          const label = ttftChartData.value.labels[val];
          if (label) {
            const d = new Date(label);
            return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
          }
        }
        return val;
      }},
    },
    y: { beginAtZero: true, title: { display: true, text: 'TTFT (ms)' } },
  },
  plugins: {
    legend: { position: 'top' as const },
    tooltip: {
      callbacks: {
        label: function (this: unknown, ctx: TooltipItem<'line'>) {
          const label = ctx.dataset?.label || '';
          if (label.includes('timeout')) return `${label}`;
          return `${label}: ${ctx.parsed?.y}ms`;
        },
      },
    },
  },
};
</script>
