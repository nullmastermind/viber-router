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
      <template #body-cell-short_id="props">
        <q-td :props="props">
          <code>{{ props.row.short_id }}</code>
          <q-btn flat dense size="sm" icon="content_copy" @click.stop="copyShortId(props.row.short_id)" />
        </q-td>
      </template>
      <template #body-cell-base_url="props">
        <q-td :props="props">
          <template v-if="store.isProtected(props.row.id) && !store.isUnlocked(props.row.id)">
            <span>🔒 {{ props.row.name }}</span>
          </template>
          <code v-else>{{ props.row.base_url }}</code>
        </q-td>
      </template>
      <template #body-cell-api_key="props">
        <q-td :props="props">
          <template v-if="store.isProtected(props.row.id) && !store.isUnlocked(props.row.id)">
            <span>🔒 {{ props.row.name }}</span>
          </template>
          <template v-else-if="props.row.api_key">
            <code>{{ props.row.api_key.substring(0, 20) }}...</code>
          </template>
          <span v-else class="text-grey">None</span>
        </q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn
            v-if="store.isProtected(props.row.id) && !store.isUnlocked(props.row.id)"
            flat
            dense
            icon="lock_open"
            @click.stop="unlockDialog(props.row.id)"
          />
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
          <q-input v-model="form.api_key" label="API Key (optional)" outlined class="q-mb-sm" />
          <q-input v-model="form.password" label="Protect Password (optional)" type="password" outlined class="q-mb-sm" />
          <q-input v-model="form.system_prompt" label="System Prompt (optional)" type="textarea" outlined />
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
import { useQuasar, copyToClipboard } from 'quasar';
import { useServersStore, type Server } from 'stores/servers';

const $q = useQuasar();
const store = useServersStore();
const search = ref('');
const showDialog = ref(false);
const editingServer = ref<Server | null>(null);
const form = ref({ name: '', base_url: '', api_key: '', password: '', system_prompt: '' });

const columns = [
  { name: 'short_id', label: 'Short ID', field: 'short_id', align: 'left' as const, sortable: true },
  { name: 'name', label: 'Name', field: 'name', align: 'left' as const, sortable: true },
  { name: 'base_url', label: 'Base URL', field: 'base_url', align: 'left' as const },
  { name: 'api_key', label: 'API Key', field: 'api_key', align: 'left' as const },
  { name: 'actions', label: 'Actions', field: 'actions', align: 'center' as const },
];

onMounted(() => store.fetchServers());

function onSearch() {
  store.fetchServers(search.value ? { search: search.value } : {});
}

async function unlockDialog(serverId: string): Promise<void> {
  await new Promise<void>((resolve) => {
    $q.dialog({
      title: 'Unlock Server',
      message: 'Enter the server password:',
      prompt: { model: '', type: 'password' as const },
      cancel: true,
    }).onOk(async (pw: string) => {
      try {
        await store.unlockServer(serverId, pw);
        // Re-fetch so API returns real credentials (backend session is now unlocked)
        await store.fetchServers(search.value ? { search: search.value } : {});
        $q.notify({ type: 'positive', message: 'Server unlocked' });
        resolve();
      } catch (e: unknown) {
        const msg = (e as Error).message || 'Failed to unlock';
        $q.notify({ type: 'negative', message: msg });
        resolve();
      }
    }).onCancel(() => resolve());
  });
}

async function openDialog(server?: Server) {
  if (server && store.isProtected(server.id) && !store.isUnlocked(server.id)) {
    let cancelled = true;
    await new Promise<void>((resolve) => {
      $q.dialog({
        title: 'Unlock Server',
        message: 'Enter the server password to edit:',
        prompt: { model: '', type: 'password' as const },
        cancel: true,
      }).onOk(async (pw: string) => {
        try {
          await store.unlockServer(server.id, pw);
          // Re-fetch so API returns real credentials
          await store.fetchServers(search.value ? { search: search.value } : {});
          cancelled = false;
          resolve();
        } catch (e: unknown) {
          const msg = (e as Error).message || 'Failed to unlock';
          $q.notify({ type: 'negative', message: msg });
          resolve();
        }
      }).onCancel(() => resolve());
    });
    if (cancelled) return;
    // Get fresh server data with real credentials from store
    const fresh = store.servers.find((s) => s.id === server.id);
    if (!fresh) return;
    editingServer.value = fresh;
    form.value = { name: fresh.name, base_url: fresh.base_url || '', api_key: fresh.api_key || '', password: '', system_prompt: fresh.system_prompt || '' };
    showDialog.value = true;
    return;
  }

  editingServer.value = server || null;
  form.value = server
    ? { name: server.name, base_url: server.base_url || '', api_key: server.api_key || '', password: '', system_prompt: server.system_prompt || '' }
    : { name: '', base_url: '', api_key: '', password: '', system_prompt: '' };
  showDialog.value = true;
}

async function saveServer() {
  try {
    const apiKey = form.value.api_key || null;
    const systemPrompt = form.value.system_prompt || null;
    if (editingServer.value) {
      const input: { name: string; base_url: string; api_key: string | null; password?: string | null; system_prompt?: string | null } = {
        name: form.value.name,
        base_url: form.value.base_url,
        api_key: apiKey,
      };
      if (form.value.password) {
        input.password = form.value.password;
      }
      if (systemPrompt !== null) {
        input.system_prompt = systemPrompt;
      }
      await store.updateServer(editingServer.value.id, input);
    } else {
      const input: { name: string; base_url: string; api_key?: string; password?: string; system_prompt?: string } = {
        name: form.value.name,
        base_url: form.value.base_url,
      };
      if (form.value.api_key) input.api_key = form.value.api_key;
      if (form.value.password) input.password = form.value.password;
      if (systemPrompt) input.system_prompt = systemPrompt;
      await store.createServer(input);
    }
    showDialog.value = false;
    store.fetchServers(search.value ? { search: search.value } : {});
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to save';
    $q.notify({ type: 'negative', message: msg });
  }
}

function copyShortId(shortId: number) {
  copyToClipboard(String(shortId)).then(() =>
    $q.notify({ message: 'Copied', type: 'positive' })
  );
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
