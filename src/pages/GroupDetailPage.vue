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
          <q-btn color="primary" label="Save" @click="saveGroup" />
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
                <q-item-label>{{ s.server_name }}</q-item-label>
                <q-item-label caption>{{ s.base_url }}</q-item-label>
              </q-item-section>
              <q-item-section side>
                <div class="row q-gutter-xs">
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

      <q-dialog v-model="showAddServer">
        <q-card style="width: 400px">
          <q-card-section><div class="text-h6">Add Server</div></q-card-section>
          <q-card-section>
            <q-select v-model="addForm.server_id" :options="availableServers" label="Server" outlined emit-value map-options class="q-mb-sm" />
            <q-input v-model.number="addForm.priority" label="Priority" type="number" outlined />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="Cancel" v-close-popup />
            <q-btn color="primary" label="Add" @click="onAddServer" />
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
    </div>
    <div v-else class="flex flex-center" style="min-height: 200px">
      <q-spinner size="lg" />
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useQuasar, copyToClipboard } from 'quasar';
import { useGroupsStore, type GroupWithServers, type GroupServerDetail } from 'stores/groups';
import { useServersStore } from 'stores/servers';

const $q = useQuasar();
const route = useRoute();
const groupsStore = useGroupsStore();
const serversStore = useServersStore();

const group = ref<GroupWithServers | null>(null);
const servers = ref<GroupServerDetail[]>([]);
const failoverCodesStr = ref('');

const showAddServer = ref(false);
const addForm = ref({ server_id: '', priority: 1 });
const showMappings = ref(false);
const editingMapping = ref<GroupServerDetail | null>(null);
const mappingEntries = ref<{ from: string; to: string }[]>([]);

const allServers = ref<{ label: string; value: string }[]>([]);
const availableServers = computed(() =>
  allServers.value.filter((s) => !servers.value.some((gs) => gs.server_id === s.value))
);

onMounted(async () => {
  await loadGroup();
  const sData = await serversStore.fetchServers({ limit: 100 });
  if (sData) {
    allServers.value = sData.data.map((s) => ({ label: s.name, value: s.id }));
  }
});

async function loadGroup() {
  const id = route.params.id as string;
  const data = await groupsStore.getGroup(id);
  group.value = data;
  servers.value = data.servers;
  failoverCodesStr.value = (data.failover_status_codes || []).join(', ');
}

async function saveGroup() {
  if (!group.value) return;
  const codes = failoverCodesStr.value
    .split(',')
    .map((s) => parseInt(s.trim(), 10))
    .filter((n) => !Number.isNaN(n));
  await groupsStore.updateGroup(group.value.id, {
    name: group.value.name,
    failover_status_codes: codes,
    is_active: group.value.is_active,
  });
  $q.notify({ type: 'positive', message: 'Saved' });
}

function copyKey() {
  if (group.value) {
    copyToClipboard(group.value.api_key).then(() =>
      $q.notify({ message: 'Copied', type: 'positive' })
    );
  }
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

async function onAddServer() {
  if (!group.value) return;
  await groupsStore.assignServer(group.value.id, addForm.value);
  showAddServer.value = false;
  loadGroup();
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
</script>
