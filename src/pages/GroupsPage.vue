<template>
  <q-page padding>
    <div class="row items-center q-mb-md">
      <div class="text-h5">Groups</div>
      <q-space />
      <q-btn color="primary" label="Add Group" icon="add" @click="openCreateDialog" />
    </div>

    <div class="row q-gutter-sm q-mb-md">
      <q-input v-model="search" label="Search..." outlined dense clearable style="width: 250px" @update:model-value="reload" />
      <q-select v-model="statusFilter" :options="statusOptions" label="Status" outlined dense clearable emit-value map-options style="width: 150px" @update:model-value="reload" />
      <q-select v-model="serverFilter" :options="serverOptions" label="Server" outlined dense clearable emit-value map-options style="width: 200px" @update:model-value="reload" />
      <q-space />
      <template v-if="selected.length > 0">
        <q-btn flat color="positive" label="Activate" @click="onBulkActivate" />
        <q-btn flat color="warning" label="Deactivate" @click="onBulkDeactivate" />
        <q-btn flat color="negative" label="Delete" @click="onBulkDelete" />
        <q-btn flat color="info" label="Assign Server" @click="showBulkAssign = true" />
      </template>
    </div>

    <q-table
      v-model:pagination="pagination"
      :rows="store.groups"
      :columns="columns"
      row-key="id"
      :loading="store.loading"
      selection="multiple"
      v-model:selected="selected"
      flat bordered
      @request="onRequest"
    >
      <template #body-cell-api_key="props">
        <q-td :props="props">
          <code>{{ props.row.api_key }}</code>
          <q-btn flat dense size="sm" icon="content_copy" @click.stop="copyKey(props.row.api_key)" />
        </q-td>
      </template>
      <template #body-cell-is_active="props">
        <q-td :props="props">
          <q-badge :color="props.row.is_active ? 'positive' : 'grey'">
            {{ props.row.is_active ? 'Active' : 'Inactive' }}
          </q-badge>
        </q-td>
      </template>
      <template #body-cell-name="props">
        <q-td :props="props">
          <a class="cursor-pointer text-primary" @click="$router.push(`/groups/${props.row.id}`)">{{ props.row.name }}</a>
        </q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn flat dense icon="delete" color="negative" @click.stop="confirmDelete(props.row)" />
        </q-td>
      </template>
    </q-table>

    <q-dialog v-model="showCreate">
      <q-card style="width: 400px">
        <q-card-section><div class="text-h6">Create Group</div></q-card-section>
        <q-card-section>
          <q-input v-model="createForm.name" label="Name" outlined />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup />
          <q-btn color="primary" label="Create" @click="onCreate" />
        </q-card-actions>
      </q-card>
    </q-dialog>
    <q-dialog v-model="showBulkAssign">
      <q-card style="width: 400px">
        <q-card-section><div class="text-h6">Assign Server to Selected Groups</div></q-card-section>
        <q-card-section>
          <q-select v-model="bulkAssignForm.server_id" :options="serverOptions" label="Server" outlined emit-value map-options class="q-mb-sm" />
          <q-input v-model.number="bulkAssignForm.priority" label="Priority" type="number" outlined />
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup />
          <q-btn color="primary" label="Assign" @click="onBulkAssign" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuasar } from 'quasar';
import { useGroupsStore, type Group } from 'stores/groups';
import { useServersStore } from 'stores/servers';
import { copyToClipboard } from 'quasar';

const $q = useQuasar();
const store = useGroupsStore();
const serversStore = useServersStore();

const search = ref('');
const statusFilter = ref<boolean | null>(null);
const serverFilter = ref<string | null>(null);
const selected = ref<Group[]>([]);
const showCreate = ref(false);
const createForm = ref({ name: '' });
const showBulkAssign = ref(false);
const bulkAssignForm = ref({ server_id: '', priority: 99 });

const statusOptions = [
  { label: 'Active', value: true },
  { label: 'Inactive', value: false },
];
const serverOptions = ref<{ label: string; value: string }[]>([]);

const pagination = ref({ page: 1, rowsPerPage: 20, rowsNumber: 0 });

const columns = [
  { name: 'name', label: 'Name', field: 'name', align: 'left' as const, sortable: true },
  { name: 'api_key', label: 'API Key', field: 'api_key', align: 'left' as const },
  { name: 'is_active', label: 'Status', field: 'is_active', align: 'center' as const },
  { name: 'servers_count', label: 'Servers', field: 'servers_count', align: 'center' as const },
  { name: 'created_at', label: 'Created', field: 'created_at', align: 'left' as const, format: (v: string) => new Date(v).toLocaleDateString() },
  { name: 'actions', label: '', field: 'actions', align: 'center' as const },
];

onMounted(async () => {
  await reload();
  const sData = await serversStore.fetchServers({ limit: 100 });
  if (sData) {
    serverOptions.value = sData.data.map((s) => ({ label: s.name, value: s.id }));
  }
});

async function reload() {
  const params: { page: number; limit: number; search?: string; is_active?: boolean; server_id?: string } = {
    page: pagination.value.page,
    limit: pagination.value.rowsPerPage,
  };
  if (search.value) params.search = search.value;
  if (statusFilter.value != null) params.is_active = statusFilter.value;
  if (serverFilter.value != null) params.server_id = serverFilter.value;
  const data = await store.fetchGroups(params);
  if (data) pagination.value.rowsNumber = data.total;
}

async function onRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  pagination.value.page = props.pagination.page;
  pagination.value.rowsPerPage = props.pagination.rowsPerPage;
  await reload();
}

function copyKey(key: string) {
  copyToClipboard(key).then(() => $q.notify({ message: 'Copied', type: 'positive' }));
}

function openCreateDialog() {
  createForm.value = { name: '' };
  showCreate.value = true;
}

async function onCreate() {
  await store.createGroup(createForm.value);
  showCreate.value = false;
  reload();
}

async function confirmDelete(group: Group) {
  $q.dialog({ title: 'Delete Group', message: `Delete "${group.name}"?`, cancel: true })
    .onOk(async () => {
      await store.deleteGroup(group.id);
      reload();
    });
}

async function onBulkActivate() {
  await store.bulkActivate(selected.value.map((g) => g.id));
  selected.value = [];
  reload();
}

async function onBulkDeactivate() {
  await store.bulkDeactivate(selected.value.map((g) => g.id));
  selected.value = [];
  reload();
}

async function onBulkDelete() {
  $q.dialog({ title: 'Bulk Delete', message: `Delete ${selected.value.length} groups?`, cancel: true })
    .onOk(async () => {
      await store.bulkDelete(selected.value.map((g) => g.id));
      selected.value = [];
      reload();
    });
}

async function onBulkAssign() {
  await store.bulkAssignServer(
    selected.value.map((g) => g.id),
    bulkAssignForm.value.server_id,
    bulkAssignForm.value.priority,
  );
  showBulkAssign.value = false;
  selected.value = [];
  reload();
}
</script>
