<template>
  <q-page padding>
    <div class="row items-center q-mb-md">
      <div class="text-h5">Servers</div>
      <q-space />
      <q-btn color="primary" label="Add Server" icon="add" @click="openDialog()" />
    </div>

    <q-input v-model="search" label="Search servers..." outlined dense clearable class="q-mb-md" @update:model-value="onSearch" />

    <q-table
      :rows="store.servers"
      :columns="columns"
      row-key="id"
      :loading="store.loading"
      flat bordered
    >
      <template #body-cell-api_key="props">
        <q-td :props="props">
          <code>{{ props.row.api_key.substring(0, 20) }}...</code>
        </q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn flat dense icon="edit" @click="openDialog(props.row)" />
          <q-btn flat dense icon="delete" color="negative" @click="confirmDelete(props.row)" />
        </q-td>
      </template>
    </q-table>

    <q-dialog v-model="showDialog">
      <q-card style="width: 500px">
        <q-card-section>
          <div class="text-h6">{{ editingServer ? 'Edit Server' : 'Add Server' }}</div>
        </q-card-section>
        <q-card-section>
          <q-input v-model="form.name" label="Name" outlined class="q-mb-sm" />
          <q-input v-model="form.base_url" label="Base URL" outlined class="q-mb-sm" />
          <q-input v-model="form.api_key" label="API Key" outlined />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup />
          <q-btn color="primary" label="Save" @click="saveServer" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useServersStore, type Server } from 'stores/servers';

const $q = useQuasar();
const store = useServersStore();
const search = ref('');
const showDialog = ref(false);
const editingServer = ref<Server | null>(null);
const form = ref({ name: '', base_url: '', api_key: '' });

const columns = [
  { name: 'name', label: 'Name', field: 'name', align: 'left' as const, sortable: true },
  { name: 'base_url', label: 'Base URL', field: 'base_url', align: 'left' as const },
  { name: 'api_key', label: 'API Key', field: 'api_key', align: 'left' as const },
  { name: 'actions', label: 'Actions', field: 'actions', align: 'center' as const },
];

onMounted(() => store.fetchServers());

function onSearch() {
  store.fetchServers(search.value ? { search: search.value } : {});
}

function openDialog(server?: Server) {
  editingServer.value = server || null;
  form.value = server
    ? { name: server.name, base_url: server.base_url, api_key: server.api_key }
    : { name: '', base_url: '', api_key: '' };
  showDialog.value = true;
}

async function saveServer() {
  try {
    if (editingServer.value) {
      await store.updateServer(editingServer.value.id, form.value);
    } else {
      await store.createServer(form.value);
    }
    showDialog.value = false;
    store.fetchServers(search.value ? { search: search.value } : {});
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to save';
    $q.notify({ type: 'negative', message: msg });
  }
}

async function confirmDelete(server: Server) {
  $q.dialog({
    title: 'Delete Server',
    message: `Delete "${server.name}"?`,
    cancel: true,
  }).onOk(async () => {
    try {
      await store.deleteServer(server.id);
      store.fetchServers(search.value ? { search: search.value } : {});
    } catch (e: unknown) {
      const data = (e as { response?: { data?: { error?: string; groups?: string[] } } })?.response?.data;
      const msg = data?.groups
        ? `Server is assigned to groups: ${data.groups.join(', ')}`
        : data?.error || 'Failed to delete';
      $q.notify({ type: 'negative', message: msg });
    }
  });
}
</script>
