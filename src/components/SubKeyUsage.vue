<template>
  <div class="q-pa-sm">
    <div v-if="loading" class="flex flex-center q-pa-sm"><q-spinner size="sm" /></div>
    <q-table
      v-else-if="rows.length"
      flat bordered dense
      :rows="rows"
      :columns="columns"
      row-key="__key"
      :pagination="{ rowsPerPage: 0 }"
      hide-pagination
    />
    <div v-else class="text-grey text-caption">No usage data</div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useGroupsStore, type ServerTokenUsage } from 'stores/groups';

const props = defineProps<{ groupId: string; groupKeyId: string }>();
const groupsStore = useGroupsStore();
const loading = ref(true);
const rows = ref<(ServerTokenUsage & { __key: string })[]>([]);

const columns = [
  { name: 'server', label: 'Server', field: 'server_name', align: 'left' as const },
  { name: 'model', label: 'Model', field: 'model', align: 'left' as const, format: (v: string | null) => v || '\u2014' },
  { name: 'input', label: 'Input', field: 'total_input_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'output', label: 'Output', field: 'total_output_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cache_creation', label: 'Cache Creation', field: 'total_cache_creation_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cache_read', label: 'Cache Read', field: 'total_cache_read_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'requests', label: 'Requests', field: 'request_count', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cost', label: 'Cost ($)', field: 'cost_usd', align: 'right' as const, format: (v: number | null) => v != null ? `$${v.toFixed(4)}` : '\u2014' },
];

onMounted(async () => {
  try {
    const data = await groupsStore.fetchKeyUsage(props.groupId, props.groupKeyId, { period: '30d' });
    rows.value = data.servers.map((r, i) => ({ ...r, __key: `${r.server_id}-${r.model}-${i}` }));
  } catch {
    // silently fail
  } finally {
    loading.value = false;
  }
});
</script>
