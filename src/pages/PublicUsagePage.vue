<template>
  <div class="public-usage-page flex flex-center">
    <div style="width: 100%; max-width: 800px; padding: 24px 16px">
      <div class="row items-center q-mb-lg">
        <div>
          <div class="text-h5" style="line-height: 1.2">Viber Router</div>
          <div class="text-caption" style="color: var(--vr-text-secondary); margin-top: 2px">Sub-Key Usage</div>
        </div>
        <q-space />
        <q-btn
          flat dense round
          :icon="isDark ? 'light_mode' : 'dark_mode'"
          :aria-label="isDark ? 'Switch to light mode' : 'Switch to dark mode'"
          @click="toggleDark"
        />
      </div>

      <!-- Key input form (when no key in route or not yet loaded) -->
      <q-card v-if="!routeKey && !data" bordered flat class="q-mb-md">
        <q-card-section>
          <q-input
            v-model="keyInput"
            label="Enter your sub-key"
            aria-label="Enter your sub-key"
            outlined
            @keyup.enter="submitKey"
          />
          <div v-if="error" class="text-negative q-mt-sm" style="font-size: 13px">{{ error }}</div>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn label="View Usage" color="primary" :loading="loading" @click="submitKey" />
        </q-card-actions>
      </q-card>

      <!-- Loading -->
      <div v-if="loading" class="flex flex-center q-pa-xl">
        <q-spinner size="lg" color="primary" />
      </div>

      <!-- Error (when we have a route key but got an error) -->
      <q-banner v-else-if="error && routeKey" rounded class="bg-negative text-white q-mb-md">
        {{ error }}
        <template #action>
          <q-btn flat label="Try another key" @click="goToForm" />
        </template>
      </q-banner>

      <!-- Data display -->
      <template v-if="data">
        <!-- Key info header -->
        <div class="text-h6 q-mb-xs">{{ data.key_name }}</div>
        <div class="text-caption q-mb-md" style="color: var(--vr-text-secondary)">Group: {{ data.group_name }}</div>

        <!-- Base URL + API Key -->
        <q-card bordered flat class="q-mb-md">
          <q-card-section class="q-py-sm">
            <div class="row items-center q-mb-xs">
              <span class="text-caption text-weight-medium" style="min-width: 70px">Base URL</span>
              <code class="q-ml-sm" style="font-size: 13px">{{ baseUrl }}</code>
              <q-btn flat dense size="xs" icon="content_copy" aria-label="Copy base URL" class="q-ml-xs" @click="copyText(baseUrl)" />
            </div>
            <div class="row items-center">
              <span class="text-caption text-weight-medium" style="min-width: 70px">API Key</span>
              <code class="q-ml-sm" style="font-size: 13px">{{ data.api_key }}</code>
              <q-btn flat dense size="xs" icon="content_copy" aria-label="Copy API key" class="q-ml-xs" @click="copyText(data.api_key)" />
            </div>
          </q-card-section>
        </q-card>

        <!-- Allowed Models -->
        <template v-if="data.allowed_models.length > 0">
          <div class="text-subtitle1 q-mb-sm">Allowed Models</div>
          <div class="q-mb-lg row q-gutter-xs">
            <q-badge v-for="m in data.allowed_models" :key="m" outline color="primary" :label="m" class="cursor-pointer" @click="copyText(m)" />
          </div>
        </template>

        <!-- Subscriptions -->
        <div class="text-subtitle1 q-mb-sm">Subscriptions</div>
        <div v-if="!data.subscriptions.length" class="text-caption q-mb-lg" style="color: var(--vr-text-secondary)">No subscriptions</div>
        <div v-else class="q-mb-lg">
          <q-card
            v-for="sub in sortedSubscriptions"
            :key="sub.id"
            bordered flat
            class="q-mb-sm"
            :style="sub.status !== 'active' ? 'opacity: 0.6' : ''"
          >
            <q-card-section>
              <div class="row items-center q-mb-xs">
                <span class="text-weight-medium q-mr-sm">{{ sub.sub_type }}</span>
                <q-badge :color="subStatusColor(sub.status)" :label="sub.status" />
                <q-space />
                <span v-if="sub.expires_at" class="text-caption" style="color: var(--vr-text-secondary)">
                  Expires {{ formatDate(sub.expires_at) }}
                </span>
              </div>
              <q-linear-progress
                :value="sub.cost_limit_usd > 0 ? sub.cost_used / sub.cost_limit_usd : 0"
                :color="sub.cost_used / sub.cost_limit_usd > 0.9 ? 'negative' : 'primary'"
                class="q-mb-xs"
                rounded
                size="8px"
                aria-label="Cost usage"
                :aria-valuenow="Math.round((sub.cost_limit_usd > 0 ? sub.cost_used / sub.cost_limit_usd : 0) * 100)"
                aria-valuemin="0"
                aria-valuemax="100"
              />
              <div class="row items-center">
                <span class="text-caption">${{ sub.cost_used.toFixed(2) }} / ${{ sub.cost_limit_usd.toFixed(2) }}</span>
                <q-space />
                <span v-if="sub.window_reset_at" class="text-caption" style="color: var(--vr-text-secondary)">
                  Resets in {{ formatCountdown(sub.window_reset_at) }}
                </span>
              </div>
            </q-card-section>
          </q-card>
        </div>

        <!-- Usage table -->
        <div class="text-subtitle1 q-mb-sm">Usage (Last 30 Days)</div>
        <div v-if="!data.usage.length" class="text-caption" style="color: var(--vr-text-secondary)">No usage data</div>
        <q-table
          v-else
          flat bordered dense
          :rows="data.usage"
          :columns="usageColumns"
          row-key="model"
          :pagination="{ rowsPerPage: 0 }"
          hide-pagination
        />

        <!-- TTFT Section -->
        <div class="q-mt-lg">
          <div class="row items-center q-mb-sm">
            <div class="text-subtitle1">TTFT (Time To First Token)</div>
            <q-space />
            <q-btn-toggle
              v-model="ttftPeriod"
              flat dense no-caps
              toggle-color="primary"
              :options="[
                { label: '1h', value: '1h' },
                { label: '6h', value: '6h' },
                { label: '24h', value: '24h' },
              ]"
            />
          </div>
          <div style="min-height: 340px">
            <div v-if="ttftLoading" class="flex flex-center" style="height: 300px">
              <q-spinner size="md" />
            </div>
            <template v-else-if="ttftData && ttftData.models.length > 0 && ttftChartData">
              <div style="height: 300px">
                <Scatter :key="ttftPeriod" :data="ttftChartData" :options="ttftChartOptions" />
              </div>
            <q-markup-table flat bordered dense class="q-mt-md">
              <thead>
                <tr>
                  <th class="text-left">Model</th>
                  <th class="text-right">Avg</th>
                  <th class="text-right">P50</th>
                  <th class="text-right">P95</th>
                  <th class="text-right">Timeouts</th>
                  <th class="text-right">Total</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="m in ttftData.models" :key="m.model ?? 'unknown'">
                  <td>{{ m.model || '\u2014' }}</td>
                  <td class="text-right">{{ m.avg_ttft_ms != null ? `${Math.round(m.avg_ttft_ms)}ms` : '\u2014' }}</td>
                  <td class="text-right">{{ m.p50_ttft_ms != null ? `${Math.round(m.p50_ttft_ms)}ms` : '\u2014' }}</td>
                  <td class="text-right">{{ m.p95_ttft_ms != null ? `${Math.round(m.p95_ttft_ms)}ms` : '\u2014' }}</td>
                  <td class="text-right">{{ m.timeout_count }}</td>
                  <td class="text-right">{{ m.total_count }}</td>
                </tr>
              </tbody>
            </q-markup-table>
          </template>
          <div v-else class="flex flex-center text-caption" style="height: 300px; color: var(--vr-text-secondary)">No TTFT data in this period</div>
          </div>
        </div>
      </template>

      <!-- Footer -->
      <div class="text-center text-caption q-mt-xl q-pt-md" style="color: var(--vr-text-secondary); border-top: 1px solid var(--vr-border)">
        <a href="https://viber.vn" target="_blank" rel="noopener" style="color: inherit; text-decoration: none">viber.vn</a>
        &middot;
        <a href="https://github.com/nullmastermind/viber-router" target="_blank" rel="noopener" style="color: inherit; text-decoration: none">Source Code</a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuasar, copyToClipboard } from 'quasar';
import { api } from 'boot/axios';
import { Scatter } from 'vue-chartjs';
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
import 'chartjs-adapter-date-fns';
import type { TooltipItem } from 'chart.js';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, TimeScale);

const $q = useQuasar();

// Initialize dark mode (same logic as MainLayout)
const isDark = ref(false);
const savedDark = localStorage.getItem('dark-mode');
if (savedDark !== null) {
  isDark.value = savedDark === 'true';
} else {
  isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches;
}
$q.dark.set(isDark.value);

function toggleDark() {
  isDark.value = !isDark.value;
  $q.dark.set(isDark.value);
  localStorage.setItem('dark-mode', String(isDark.value));
}

interface ModelUsage {
  model: string | null;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cache_creation_tokens: number;
  total_cache_read_tokens: number;
  request_count: number;
  cost_usd: number | null;
}

interface Subscription {
  id: string;
  sub_type: string;
  cost_limit_usd: number;
  status: string;
  cost_used: number;
  window_reset_at: string | null;
  activated_at: string | null;
  expires_at: string | null;
}

interface UsageData {
  key_name: string;
  group_name: string;
  api_key: string;
  allowed_models: string[];
  usage: ModelUsage[];
  subscriptions: Subscription[];
}

interface TtftDataPoint {
  created_at: string;
  ttft_ms: number | null;
  timed_out: boolean;
}

interface ModelTtftStats {
  model: string | null;
  avg_ttft_ms: number | null;
  p50_ttft_ms: number | null;
  p95_ttft_ms: number | null;
  timeout_count: number;
  total_count: number;
  data_points: TtftDataPoint[];
}

interface TtftResponse {
  models: ModelTtftStats[];
}

const route = useRoute();
const router = useRouter();
const keyInput = ref('');
const loading = ref(false);
const error = ref('');
const data = ref<UsageData | null>(null);

const routeKey = computed(() => route.params.key as string | undefined);

const baseUrl = computed(() => window.location.origin);

function copyText(text: string) {
  copyToClipboard(text).then(() =>
    $q.notify({ message: 'Copied', type: 'positive' })
  );
}

const sortedSubscriptions = computed(() => {
  if (!data.value) return [];
  return [...data.value.subscriptions].sort((a, b) => {
    if (a.status === 'active' && b.status !== 'active') return -1;
    if (a.status !== 'active' && b.status === 'active') return 1;
    return 0;
  });
});

const usageColumns = [
  { name: 'model', label: 'Model', field: 'model', align: 'left' as const, format: (v: string | null) => v || '\u2014' },
  { name: 'input', label: 'Input', field: 'total_input_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'output', label: 'Output', field: 'total_output_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cache_creation', label: 'Cache Creation', field: 'total_cache_creation_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cache_read', label: 'Cache Read', field: 'total_cache_read_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'requests', label: 'Requests', field: 'request_count', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cost', label: 'Cost ($)', field: 'cost_usd', align: 'right' as const, format: (v: number | null) => v != null ? `$${v.toFixed(4)}` : '\u2014' },
];

function subStatusColor(status: string) {
  if (status === 'active') return 'positive';
  if (status === 'exhausted') return 'negative';
  return 'grey';
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString();
}

function formatCountdown(iso: string) {
  const diff = new Date(iso).getTime() - Date.now();
  if (diff <= 0) return 'now';
  const hours = Math.floor(diff / 3600000);
  const minutes = Math.floor((diff % 3600000) / 60000);
  return hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
}

const CHART_COLORS = ['#1976D2', '#26A69A', '#FF6F00', '#AB47BC', '#EF5350', '#66BB6A', '#42A5F5', '#FFA726'];

// TTFT state
const ttftPeriod = ref('24h');
const ttftLoading = ref(false);
const ttftData = ref<TtftResponse | null>(null);

async function fetchTtft(key: string) {
  ttftLoading.value = true;
  try {
    const res = await api.get<TtftResponse>('/api/public/ttft', {
      params: { key, period: ttftPeriod.value },
    });
    ttftData.value = res.data;
  } catch {
    ttftData.value = null;
  } finally {
    ttftLoading.value = false;
  }
}

const ttftChartData = computed(() => {
  if (!ttftData.value || ttftData.value.models.length === 0) return null;
  const datasets = ttftData.value.models.map((model, idx) => {
    const color = CHART_COLORS[idx % CHART_COLORS.length] as string;
    const pts: { x: number; y: number }[] = [];
    const bgColors: string[] = [];
    const radii: number[] = [];
    const styles: string[] = [];
    for (const p of model.data_points) {
      const x = new Date(p.created_at).getTime();
      if (p.timed_out) {
        pts.push({ x, y: 0 });
        bgColors.push('#EF5350');
        radii.push(6);
        styles.push('crossRot');
      } else if (p.ttft_ms != null) {
        pts.push({ x, y: p.ttft_ms });
        bgColors.push(color);
        radii.push(4);
        styles.push('circle');
      }
    }
    return {
      label: model.model || 'Unknown',
      data: pts,
      backgroundColor: bgColors,
      pointRadius: radii,
      pointStyle: styles,
      showLine: false,
    };
  });
  return { datasets };
});

const ttftChartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  animation: false as const,
  scales: {
    x: {
      type: 'time' as const,
      time: { unit: 'minute' as const, displayFormats: { minute: 'HH:mm' }, tooltipFormat: 'HH:mm:ss' },
      title: { display: false },
      ticks: { maxTicksLimit: 10 },
    },
    y: { beginAtZero: true, title: { display: true, text: 'TTFT (ms)' } },
  },
  plugins: {
    legend: { position: 'top' as const },
    tooltip: {
      callbacks: {
        label: function (this: unknown, ctx: TooltipItem<'scatter'>) {
          const label = ctx.dataset?.label || '';
          if (ctx.parsed?.y === 0) return `${label} (timeout)`;
          return `${label}: ${ctx.parsed?.y}ms`;
        },
      },
    },
  },
}));

async function fetchUsage(key: string) {
  loading.value = true;
  error.value = '';
  data.value = null;
  try {
    const res = await api.get<UsageData>('/api/public/usage', { params: { key } });
    data.value = res.data;
    fetchTtft(key);
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } }).response?.status;
    if (status === 403) error.value = 'Invalid or inactive key';
    else if (status === 429) error.value = 'Too many requests. Please try again later.';
    else error.value = 'An error occurred. Please try again.';
  } finally {
    loading.value = false;
  }
}

function submitKey() {
  const key = keyInput.value.trim();
  if (!key) return;
  router.push(`/usage/${encodeURIComponent(key)}`);
}

function goToForm() {
  error.value = '';
  data.value = null;
  router.push('/usage');
}

onMounted(() => {
  if (routeKey.value) fetchUsage(routeKey.value);
});

// Watch for route param changes
watch(routeKey, (key) => {
  if (key) fetchUsage(key);
});

// Re-fetch TTFT when period changes
watch(ttftPeriod, () => {
  const key = routeKey.value;
  if (key) fetchTtft(key);
});
</script>

<style scoped lang="scss">
.public-usage-page {
  min-height: 100vh;
  background-color: var(--vr-bg-page);
}
</style>
