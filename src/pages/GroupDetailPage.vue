<template>
  <q-page padding>
    <div v-if="group" class="q-gutter-md">
      <div class="row items-center">
        <q-btn flat icon="arrow_back" @click="$router.push('/groups')" />
        <div class="text-h5 q-ml-sm">{{ group.name }}</div>
        <q-space />
        <q-toggle v-model="group.is_active" label="Active" @update:model-value="saveGroup" />
      </div>

      <q-tabs v-model="activeTab" dense align="left" class="text-primary" active-color="primary" indicator-color="primary">
        <q-tab name="properties" label="Properties" />
        <q-tab name="servers" label="Servers" />
        <q-tab name="keys" label="Keys" />
        <q-tab name="allowed-models" label="Allowed Models" />
        <q-tab name="ttft" label="TTFT" />
        <q-tab name="token-usage" label="Token Usage" />
      </q-tabs>
      <q-separator />

      <q-tab-panels v-model="activeTab" animated>
        <!-- Properties Tab -->
        <q-tab-panel name="properties" class="q-pa-none q-gutter-md">
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
              <q-select v-model="group.count_tokens_server_id" :options="allServers" label="Default server for /v1/messages/count_tokens" outlined dense emit-value map-options clearable class="q-mb-sm" @update:model-value="saveGroup" />
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
        </q-tab-panel>


        <!-- Servers Tab -->
        <q-tab-panel name="servers" class="q-pa-none">
          <q-card flat bordered>
            <q-card-section>
              <div class="row items-center q-mb-sm">
                <div class="text-subtitle1">Servers (priority order)</div>
                <q-space />
                <q-btn flat dense icon="add" label="Add Server" @click="showAddServer = true" />
              </div>
              <q-list bordered separator>
                <q-item v-for="(s, idx) in servers" :key="s.server_id" :class="{ 'disabled-server': !s.is_enabled }">
                  <q-item-section avatar>
                    <div class="column items-center">
                      <q-btn flat dense icon="arrow_upward" :disable="idx === 0" @click="moveServer(idx, -1)" />
                      <span class="text-caption">{{ s.priority }}</span>
                      <q-btn flat dense icon="arrow_downward" :disable="idx === servers.length - 1" @click="moveServer(idx, 1)" />
                    </div>
                  </q-item-section>
                  <q-item-section>
                    <q-item-label :class="{ 'text-strike': !s.is_enabled }">
                      {{ s.server_name }}
                      <q-badge outline class="q-ml-sm">
                        #{{ s.short_id }}
                        <q-btn flat dense size="xs" icon="content_copy" class="q-ml-xs" @click.stop="copyShortId(s.short_id)" />
                      </q-badge>
                      <q-badge v-if="getCircuitStatus(s.server_id)?.is_open" color="negative" class="q-ml-sm">
                        Circuit Open ({{ formatCircuitRemaining(getCircuitStatus(s.server_id)?.remaining_seconds ?? 0) }})
                      </q-badge>
                      <q-badge v-if="s.max_requests != null && s.rate_window_seconds != null" outline color="purple" class="q-ml-sm" :aria-label="`Rate limit: ${s.max_requests} requests per ${s.rate_window_seconds} seconds`">
                        {{ s.max_requests }}/{{ s.rate_window_seconds }}s
                      </q-badge>
                    </q-item-label>
                    <q-item-label caption>{{ s.base_url }}</q-item-label>
                  </q-item-section>
                  <q-item-section side>
                    <div class="row q-gutter-xs items-center">
                      <q-badge
                        outline
                        :color="hasNonDefaultRate(s) ? 'orange' : 'grey'"
                        class="cursor-pointer q-pa-xs"
                        tabindex="0"
                        role="button"
                        :aria-label="`Edit cost rates for ${s.server_name}`"
                        @click.stop="openRateModal(s)"
                        @keydown.enter.stop="openRateModal(s)"
                        @keydown.space.stop="openRateModal(s)"
                      >
                        x{{ displayRate(s) }}
                      </q-badge>
                      <q-toggle v-model="s.is_enabled" dense :aria-label="`${s.server_name} enabled`" @update:model-value="toggleServerEnabled(s)" />
                      <q-btn flat dense icon="edit" :aria-label="`Edit server ${s.server_name}`" @click="openEditServer(s)" />
                      <q-btn flat dense icon="tune" :aria-label="`Edit model mappings for ${s.server_name}`" @click="editMappings(s)" />
                      <q-btn flat dense icon="delete" color="negative" :aria-label="`Remove server ${s.server_name}`" @click="onRemoveServer(s)" />
                    </div>
                  </q-item-section>
                </q-item>
                <q-item v-if="servers.length === 0">
                  <q-item-section class="text-grey">No servers assigned</q-item-section>
                </q-item>
              </q-list>
            </q-card-section>
          </q-card>
        </q-tab-panel>


        <!-- Keys Tab -->
        <q-tab-panel name="keys" class="q-pa-none">
          <q-card flat bordered>
            <q-card-section>
              <div class="row items-center q-mb-sm">
                <div class="text-subtitle1">Sub-Keys</div>
                <q-space />
                <q-input v-model="subKeySearch" placeholder="Search by name or key" outlined dense clearable style="max-width: 250px" class="q-mr-sm" @update:model-value="loadSubKeys" />
                <q-btn color="primary" dense label="Create Key" @click="showCreateKey = true" />
              </div>
              <div v-if="subKeysLoading && !subKeys.length" class="flex flex-center q-pa-lg"><q-spinner size="md" /></div>
              <q-banner v-else-if="subKeyError" class="bg-negative text-white q-mb-sm" rounded>
                {{ subKeyError }}
                <template #action>
                  <q-btn flat label="Retry" @click="loadSubKeys" />
                </template>
              </q-banner>
              <q-table
                v-else
                flat bordered dense
                :rows="subKeys"
                :columns="subKeyColumns"
                row-key="id"
                :pagination="subKeyPagination"
                @request="onSubKeyRequest"
              >
                <template #body="props">
                  <q-tr :props="props" @click="onExpandSubKey(props)" @keydown.enter="onExpandSubKey(props)" tabindex="0" role="button" :aria-expanded="props.expand" class="cursor-pointer">
                    <q-td key="name" :props="props">{{ props.row.name }}</q-td>
                    <q-td key="api_key" :props="props">
                      <code>{{ maskKey(props.row.api_key) }}</code>
                      <q-btn flat dense size="xs" icon="content_copy" aria-label="Copy key" @click.stop="copyText(props.row.api_key)" />
                    </q-td>
                    <q-td key="is_active" :props="props">
                      <q-toggle :model-value="props.row.is_active" dense :aria-label="`${props.row.name} active`" @update:model-value="toggleSubKey(props.row, $event)" @click.stop />
                    </q-td>
                    <q-td key="actions" :props="props">
                      <q-btn flat dense size="sm" label="Regenerate" @click.stop="onRegenerateSubKey(props.row)" />
                    </q-td>
                  </q-tr>
                  <q-tr v-if="props.expand" :props="props">
                    <q-td colspan="4" class="q-pa-sm">
                      <div v-if="allowedModels.length > 0" class="q-mb-md">
                        <div class="text-subtitle2 q-mb-xs">Allowed Models</div>
                        <div class="row q-gutter-xs q-mb-sm items-center">
                          <q-chip
                            v-for="km in getKeyAllowedModels(props.row.id)"
                            :key="km.id"
                            removable
                            dense
                            color="primary"
                            text-color="white"
                            :disable="!!keyModelsLoading[props.row.id]"
                            @remove="onRemoveKeyModel(props.row.id, km.id)"
                          >
                            {{ km.name }}
                          </q-chip>
                          <q-spinner v-if="keyModelsLoading[props.row.id]" size="xs" class="q-ml-xs" />
                          <span v-else-if="getKeyAllowedModels(props.row.id).length === 0" class="text-caption text-grey">Inherits all group models</span>
                        </div>
                        <q-select
                          :model-value="null"
                          :options="keyModelOptions(props.row.id)"
                          label="Add model restriction"
                          outlined dense
                          emit-value map-options
                          :loading="!!keyModelsLoading[props.row.id]"
                          :disable="!!keyModelsLoading[props.row.id]"
                          style="max-width: 300px"
                          @update:model-value="(v: string) => onAddKeyModel(props.row.id, v)"
                        />
                      </div>
                      <div class="q-mb-md">
                        <div class="row items-center q-mb-xs">
                          <div class="text-subtitle2">Subscriptions</div>
                          <q-space />
                          <q-btn v-if="activePlans.length > 0" flat dense icon="add" label="Add Subscription" color="primary">
                            <q-menu>
                              <q-list dense style="min-width: 200px">
                                <q-item v-for="plan in activePlans" :key="plan.id" clickable v-close-popup @click="onAssignSubscription(props.row.id, plan.id)">
                                  <q-item-section>
                                    <q-item-label>{{ plan.name }}</q-item-label>
                                    <q-item-label caption>{{ plan.sub_type }} &middot; ${{ plan.cost_limit_usd.toFixed(2) }}</q-item-label>
                                  </q-item-section>
                                </q-item>
                              </q-list>
                            </q-menu>
                          </q-btn>
                          <span v-else class="text-caption text-grey q-ml-sm">No active plans available</span>
                        </div>
                        <div v-if="!keySubscriptions[props.row.id] || (keySubscriptions[props.row.id] ?? []).length === 0" class="text-caption text-grey q-mb-sm">
                          No subscriptions &mdash; unlimited usage
                        </div>
                        <q-table
                          v-else
                          flat bordered dense
                          :rows="keySubscriptions[props.row.id] ?? []"
                          :columns="subColumns"
                          row-key="id"
                          :pagination="{ rowsPerPage: 10 }"
                          hide-pagination
                        >
                          <template #body-cell-status="sProps">
                            <q-td :props="sProps">
                              <q-badge :color="subStatusColor(sProps.row.status)" :label="sProps.row.status" />
                            </q-td>
                          </template>
                          <template #body-cell-cost_used="sProps">
                            <q-td :props="sProps">
                              ${{ sProps.row.cost_used.toFixed(2) }} / ${{ sProps.row.cost_limit_usd.toFixed(2) }}{{ sProps.row.sub_type === 'hourly_reset' ? ' (window)' : '' }}
                            </q-td>
                          </template>
                          <template #body-cell-actions="sProps">
                            <q-td :props="sProps">
                              <q-btn v-if="sProps.row.status === 'active'" flat dense size="sm" label="Cancel" color="negative" @click.stop="onCancelSubscription(props.row.id, sProps.row.id)" />
                            </q-td>
                          </template>
                        </q-table>
                      </div>
                      <SubKeyUsage :group-id="group?.id ?? ''" :group-key-id="props.row.id" />
                      <div class="q-mt-sm">
                        <TtftChart :group-id="group?.id ?? ''" :group-key-id="props.row.id" />
                      </div>
                    </q-td>
                  </q-tr>
                </template>
                <template #no-data>
                  <div class="text-grey text-center q-pa-md">No sub-keys created. Click "Create Key" to add one.</div>
                </template>
              </q-table>
            </q-card-section>
          </q-card>
        </q-tab-panel>

        <!-- Allowed Models Tab -->
        <q-tab-panel name="allowed-models" class="q-pa-none">
          <q-card flat bordered>
            <q-card-section>
              <div class="row items-center q-mb-sm">
                <div class="text-subtitle1">Allowed Models</div>
                <q-space />
                <div class="text-caption text-grey q-mr-md">{{ allowedModels.length === 0 ? 'No restrictions — all models allowed' : `${allowedModels.length} model(s) allowed` }}</div>
              </div>
              <div class="row q-gutter-sm q-mb-md">
                <q-select
                  v-model="modelToAdd"
                  :options="filteredModelOptions"
                  label="Add model"
                  outlined dense
                  use-input
                  input-debounce="200"
                  emit-value map-options
                  clearable
                  :loading="allowedModelsLoading"
                  style="min-width: 300px"
                  new-value-mode="add-unique"
                  @filter="onModelFilter"
                  @new-value="onNewModelValue"
                  @update:model-value="onAddAllowedModel"
                >
                  <template #no-option>
                    <q-item>
                      <q-item-section class="text-grey">
                        {{ modelSearchText ? `Press Enter to create "${modelSearchText}"` : 'Type to search or create' }}
                      </q-item-section>
                    </q-item>
                  </template>
                </q-select>
              </div>
              <q-list v-if="allowedModels.length > 0" bordered separator>
                <q-item v-for="m in allowedModels" :key="m.id">
                  <q-item-section>
                    <q-item-label>{{ m.name }}</q-item-label>
                  </q-item-section>
                  <q-item-section side>
                    <q-btn flat dense icon="close" color="negative" :loading="allowedModelsLoading" :aria-label="`Remove ${m.name}`" @click="onRemoveAllowedModel(m)" />
                  </q-item-section>
                </q-item>
              </q-list>
              <div v-else class="text-grey q-pa-md text-center">
                No model restrictions configured. Add models to restrict which models this group can use.
              </div>
            </q-card-section>
          </q-card>
        </q-tab-panel>

        <!-- TTFT Tab -->
        <q-tab-panel name="ttft" class="q-pa-none">
          <TtftChart :group-id="group?.id ?? ''" :group-key-id="ttftKeyFilter || undefined">
            <template #filter>
              <q-select
                v-model="ttftKeyFilter"
                :options="ttftKeyOptions"
                emit-value map-options
                outlined dense
                style="min-width: 180px"
                class="q-mr-sm"
              />
            </template>
          </TtftChart>
        </q-tab-panel>

        <!-- Token Usage Tab -->
        <q-tab-panel name="token-usage" class="q-pa-none">
          <q-card flat bordered>
            <q-card-section>
          <div class="row items-center q-mb-sm">
            <div class="text-subtitle1">Token Usage</div>
            <q-space />
            <q-btn-toggle
              v-model="tokenUsagePeriod"
              flat dense no-caps
              toggle-color="primary"
              :options="[
                { label: '1h', value: '1h' },
                { label: '6h', value: '6h' },
                { label: '24h', value: '24h' },
                { label: '7d', value: '7d' },
                { label: '30d', value: '30d' },
              ]"
            />
          </div>
          <div class="row q-gutter-sm q-mb-sm">
            <q-select
              v-model="tokenUsageServerFilter"
              :options="tokenUsageServerOptions"
              label="Server"
              outlined dense
              emit-value map-options clearable
              style="min-width: 200px"
            />
            <q-toggle v-model="tokenUsageDynamicKeyFilter" label="Dynamic keys only" dense />
            <q-input v-model="tokenUsageKeyHashFilter" label="Key hash" outlined dense clearable style="max-width: 200px" />
            <q-btn flat dense icon="refresh" @click="loadTokenUsage" />
          </div>
          <div v-if="tokenUsageLoading && !tokenUsageStats" class="flex flex-center q-pa-lg">
            <q-spinner size="md" />
          </div>
          <q-banner v-else-if="tokenUsageError" class="bg-negative text-white q-mb-sm" rounded>
            {{ tokenUsageError }}
            <template #action>
              <q-btn flat label="Retry" @click="loadTokenUsage" />
            </template>
          </q-banner>
          <q-banner v-else-if="tokenUsageStats && tokenUsageStats.servers.length === 0" class="q-mb-sm" rounded>
            No token usage data in this period
          </q-banner>
          <q-table
            v-else-if="tokenUsageStats && tokenUsageStats.servers.length > 0"
            flat bordered dense
            :rows="filteredTokenUsageRows"
            :columns="tokenUsageColumns"
            row-key="__key"
            :pagination="{ rowsPerPage: 0 }"
            hide-pagination
          >
            <template #bottom-row>
              <q-tr class="text-weight-bold">
                <q-td>Total</q-td>
                <q-td />
                <q-td class="text-right">{{ totalTokenUsage.input.toLocaleString() }}</q-td>
                <q-td class="text-right">{{ totalTokenUsage.output.toLocaleString() }}</q-td>
                <q-td class="text-right">{{ totalTokenUsage.cacheCreation.toLocaleString() }}</q-td>
                <q-td class="text-right">{{ totalTokenUsage.cacheRead.toLocaleString() }}</q-td>
                <q-td class="text-right">{{ totalTokenUsage.requests.toLocaleString() }}</q-td>
                <q-td class="text-right">{{ totalTokenUsage.cost != null ? `$${totalTokenUsage.cost.toFixed(4)}` : '\u2014' }}</q-td>
              </q-tr>
            </template>
          </q-table>
            </q-card-section>
          </q-card>
        </q-tab-panel>
      </q-tab-panels>

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
            <q-input v-model="editServerForm.api_key" label="API Key (optional)" outlined class="q-mb-sm" />
            <div class="text-subtitle2 q-mt-md q-mb-xs">Circuit Breaker</div>
            <div class="text-caption text-grey q-mb-sm">
              Auto-shutdown the server when errors exceed a threshold, then auto-restart after a cooldown period. Leave all fields empty to disable.
            </div>
            <q-input
              v-model.number="editServerCbForm.cb_max_failures"
              label="Max Failures"
              type="number"
              :min="1"
              outlined
              dense
              clearable
              class="q-mb-sm"
              @clear="onCbFieldClear('cb_max_failures')"
            />
            <q-input
              v-model.number="editServerCbForm.cb_window_seconds"
              label="Failure Window (seconds)"
              type="number"
              :min="1"
              outlined
              dense
              clearable
              class="q-mb-sm"
              @clear="onCbFieldClear('cb_window_seconds')"
            />
            <q-input
              v-model.number="editServerCbForm.cb_cooldown_seconds"
              label="Cooldown (seconds)"
              type="number"
              :min="1"
              outlined
              dense
              clearable
              class="q-mb-sm"
              @clear="onCbFieldClear('cb_cooldown_seconds')"
            />
            <div class="text-subtitle2 q-mt-md q-mb-xs">Rate Limit</div>
            <div class="text-caption text-grey q-mb-sm">
              Limit how many requests this server receives within a time window. Leave both fields empty to disable.
            </div>
            <q-input
              v-model.number="editServerRlForm.max_requests"
              label="Max Requests"
              type="number"
              :min="1"
              outlined
              dense
              clearable
              class="q-mb-sm"
              @clear="onRlFieldClear()"
            />
            <q-input
              v-model.number="editServerRlForm.rate_window_seconds"
              label="Window (seconds)"
              type="number"
              :min="1"
              outlined
              dense
              clearable
              class="q-mb-sm"
              @clear="onRlFieldClear()"
            />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="Cancel" v-close-popup />
            <q-btn color="primary" label="Save" :loading="savingServer" @click="onSaveEditServer" />
          </q-card-actions>
        </q-card>
      </q-dialog>

      <q-dialog v-model="showCreateKey" @hide="newKeyName = ''">
        <q-card style="width: 400px">
          <q-card-section><div class="text-h6">Create Sub-Key</div></q-card-section>
          <q-card-section>
            <q-input v-model="newKeyName" label="Name" outlined maxlength="100" />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="Cancel" v-close-popup />
            <q-btn color="primary" label="Create" :loading="creatingKey" @click="onCreateKey" />
          </q-card-actions>
        </q-card>
      </q-dialog>

      <q-dialog v-model="showRateModal">
        <q-card style="width: 400px">
          <q-card-section><div class="text-h6">Cost Rates — {{ rateEditServer?.server_name }}</div></q-card-section>
          <q-card-section class="q-gutter-sm">
            <q-input v-model.number="rateForm.rate_input" label="Input Rate" outlined dense type="number" :min="0" placeholder="1.0" clearable @clear="rateForm.rate_input = null" />
            <q-input v-model.number="rateForm.rate_output" label="Output Rate" outlined dense type="number" :min="0" placeholder="1.0" clearable @clear="rateForm.rate_output = null" />
            <q-input v-model.number="rateForm.rate_cache_write" label="Cache Write Rate" outlined dense type="number" :min="0" placeholder="1.0" clearable @clear="rateForm.rate_cache_write = null" />
            <q-input v-model.number="rateForm.rate_cache_read" label="Cache Read Rate" outlined dense type="number" :min="0" placeholder="1.0" clearable @clear="rateForm.rate_cache_read = null" />
          </q-card-section>
          <q-card-actions align="right">
            <q-btn flat label="Cancel" v-close-popup />
            <q-btn color="primary" label="Save" :loading="savingRate" @click="onSaveRates" />
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
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useQuasar, copyToClipboard } from 'quasar';
import { useGroupsStore, type GroupWithServers, type GroupServerDetail, type CircuitStatus, type TokenUsageStats, type GroupKey, type Model } from 'stores/groups';
import { useServersStore } from 'stores/servers';
import { useModelsStore } from 'stores/models';
import { api } from 'boot/axios';
import SubKeyUsage from 'components/SubKeyUsage.vue';
import TtftChart from 'components/TtftChart.vue';

const $q = useQuasar();
const route = useRoute();
const router = useRouter();
const groupsStore = useGroupsStore();
const serversStore = useServersStore();
const modelsStore = useModelsStore();

const group = ref<GroupWithServers | null>(null);
const servers = ref<GroupServerDetail[]>([]);
const failoverCodesStr = ref('');
const validTabs = ['properties', 'servers', 'keys', 'allowed-models', 'ttft', 'token-usage'];
const initialTab = validTabs.includes(route.query.tab as string) ? (route.query.tab as string) : 'properties';
const activeTab = ref(initialTab);

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
const editServerCbForm = ref({
  cb_max_failures: null as number | null,
  cb_window_seconds: null as number | null,
  cb_cooldown_seconds: null as number | null,
});
const editServerRlForm = ref({
  max_requests: null as number | null,
  rate_window_seconds: null as number | null,
});
const savingServer = ref(false);

const keyBuilderEntries = ref<{ server_id: string; server_name: string; short_id: number; key: string; defaultKey: string }[]>([]);
const showAllKeyBuilderServers = ref(false);

const ttftTimeoutStr = ref('');
const ttftKeyFilter = ref<string | null>(null);
const ctMappingEntries = ref<{ from: string; to: string }[]>([]);
const circuitStatuses = ref<CircuitStatus[]>([]);
let circuitPollTimer: ReturnType<typeof setInterval> | null = null;

// Token usage state
const tokenUsageStats = ref<TokenUsageStats | null>(null);
const tokenUsageLoading = ref(false);
const tokenUsageError = ref('');
const tokenUsagePeriod = ref('24h');
const tokenUsageServerFilter = ref<string | null>(null);
const tokenUsageDynamicKeyFilter = ref(false);
const tokenUsageKeyHashFilter = ref('');

// Sub-keys state
const subKeys = ref<GroupKey[]>([]);
const subKeysLoading = ref(false);
const subKeySearch = ref('');
const subKeyError = ref('');
const subKeyPagination = ref({ page: 1, rowsPerPage: 50, rowsNumber: 0, sortBy: '', descending: false });
const showCreateKey = ref(false);
const newKeyName = ref('');
const creatingKey = ref(false);

// Subscription state
interface KeySubscription {
  id: string;
  group_key_id: string;
  plan_id: string | null;
  sub_type: string;
  cost_limit_usd: number;
  model_limits: Record<string, number>;
  reset_hours: number | null;
  duration_days: number;
  status: string;
  activated_at: string | null;
  expires_at: string | null;
  created_at: string;
  cost_used: number;
}
interface SubscriptionPlan {
  id: string;
  name: string;
  sub_type: string;
  cost_limit_usd: number;
  is_active: boolean;
}
const keySubscriptions = ref<Record<string, KeySubscription[]>>({});
const activePlans = ref<SubscriptionPlan[]>([]);

const subColumns = [
  { name: 'sub_type', label: 'Type', field: 'sub_type', align: 'left' as const },
  { name: 'cost_limit_usd', label: 'Budget', field: 'cost_limit_usd', align: 'right' as const, format: (v: number) => `$${v.toFixed(2)}` },
  { name: 'cost_used', label: 'Used', field: 'cost_used', align: 'right' as const },
  { name: 'status', label: 'Status', field: 'status', align: 'center' as const },
  { name: 'duration_days', label: 'Duration', field: 'duration_days', align: 'right' as const, format: (v: number) => `${v}d` },
  { name: 'actions', label: '', field: 'id', align: 'right' as const },
];

function subStatusColor(status: string): string {
  switch (status) {
    case 'active': return 'green';
    case 'exhausted': return 'red';
    case 'expired': return 'orange';
    case 'cancelled': return 'grey';
    default: return 'grey';
  }
}

// Rate modal state
const showRateModal = ref(false);
const rateEditServer = ref<GroupServerDetail | null>(null);
const savingRate = ref(false);
const rateForm = ref({
  rate_input: null as number | null,
  rate_output: null as number | null,
  rate_cache_write: null as number | null,
  rate_cache_read: null as number | null,
});

// Allowed models state
const allowedModels = ref<Model[]>([]);
const modelToAdd = ref<string | null>(null);
const modelSearchText = ref('');
const allModelsOptions = ref<{ label: string; value: string }[]>([]);
const filteredModelOptions = ref<{ label: string; value: string }[]>([]);
const allowedModelsLoading = ref(false);

// Key allowed models state
const keyAllowedModelsMap = ref<Record<string, Model[]>>({});
const keyModelsLoading = ref<Record<string, boolean>>({});

const ttftKeyOptions = computed(() => [
  { label: 'All keys', value: null },
  ...subKeys.value.map((k) => ({ label: k.name, value: k.id })),
]);

const subKeyColumns = [
  { name: 'name', label: 'Name', field: 'name', align: 'left' as const },
  { name: 'api_key', label: 'Key', field: 'api_key', align: 'left' as const },
  { name: 'is_active', label: 'Status', field: 'is_active', align: 'center' as const },
  { name: 'actions', label: 'Actions', field: 'id', align: 'right' as const },
];

const tokenUsageServerOptions = computed(() =>
  servers.value.map((s) => ({ label: s.server_name, value: s.server_id })),
);

const tokenUsageColumns = [
  { name: 'server', label: 'Server', field: 'server_name', align: 'left' as const },
  { name: 'model', label: 'Model', field: 'model', align: 'left' as const, format: (v: string | null) => v || '\u2014' },
  { name: 'input', label: 'Input Tokens', field: 'total_input_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'output', label: 'Output Tokens', field: 'total_output_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cache_creation', label: 'Cache Creation', field: 'total_cache_creation_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cache_read', label: 'Cache Read', field: 'total_cache_read_tokens', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'requests', label: 'Requests', field: 'request_count', align: 'right' as const, format: (v: number) => v.toLocaleString() },
  { name: 'cost', label: 'Cost ($)', field: 'cost_usd', align: 'right' as const, format: (v: number | null) => v != null ? `$${v.toFixed(4)}` : '\u2014' },
];

const filteredTokenUsageRows = computed(() => {
  if (!tokenUsageStats.value) return [];
  let rows = tokenUsageStats.value.servers;
  if (tokenUsageServerFilter.value) {
    rows = rows.filter((r) => r.server_id === tokenUsageServerFilter.value);
  }
  return rows.map((r, i) => ({ ...r, __key: `${r.server_id}-${r.model}-${i}` }));
});

const totalTokenUsage = computed(() => {
  const rows = filteredTokenUsageRows.value;
  const input = rows.reduce((s, r) => s + r.total_input_tokens, 0);
  const output = rows.reduce((s, r) => s + r.total_output_tokens, 0);
  const cacheCreation = rows.reduce((s, r) => s + r.total_cache_creation_tokens, 0);
  const cacheRead = rows.reduce((s, r) => s + r.total_cache_read_tokens, 0);
  const requests = rows.reduce((s, r) => s + r.request_count, 0);
  const costRows = rows.filter((r) => r.cost_usd != null);
  const cost = costRows.length > 0 ? costRows.reduce((s, r) => s + (r.cost_usd ?? 0), 0) : null;
  return { input, output, cacheCreation, cacheRead, requests, cost };
});
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
  loadTtftKeys();
  loadCircuitStatus();
  loadTokenUsage();
});

onUnmounted(() => {
  stopCircuitPoll();
});



async function loadGroup() {
  const id = route.params.id as string;
  const data = await groupsStore.getGroup(id);
  group.value = data;
  servers.value = data.servers;
  allowedModels.value = data.allowed_models || [];
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

async function loadCircuitStatus() {
  if (!group.value) return;
  try {
    circuitStatuses.value = await groupsStore.fetchCircuitStatus(group.value.id);
  } catch {
    circuitStatuses.value = [];
  }
  // Start/stop polling based on whether any circuit is open
  if (circuitStatuses.value.some((c) => c.is_open)) {
    startCircuitPoll();
  } else {
    stopCircuitPoll();
  }
}

function startCircuitPoll() {
  if (circuitPollTimer) return;
  circuitPollTimer = setInterval(loadCircuitStatus, 10_000);
}

function stopCircuitPoll() {
  if (circuitPollTimer) {
    clearInterval(circuitPollTimer);
    circuitPollTimer = null;
  }
}

function getCircuitStatus(serverId: string): CircuitStatus | undefined {
  return circuitStatuses.value.find((c) => c.server_id === serverId);
}

function formatCircuitRemaining(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return m > 0 ? `${m}m ${s}s` : `${s}s`;
}

async function loadTokenUsage() {
  if (!group.value) return;
  tokenUsageLoading.value = true;
  tokenUsageError.value = '';
  try {
    const params: { period?: string; is_dynamic_key?: boolean; key_hash?: string } = {
      period: tokenUsagePeriod.value,
    };
    if (tokenUsageDynamicKeyFilter.value) params.is_dynamic_key = true;
    if (tokenUsageKeyHashFilter.value?.trim()) params.key_hash = tokenUsageKeyHashFilter.value.trim();
    tokenUsageStats.value = await groupsStore.fetchTokenUsageStats(group.value.id, params);
  } catch {
    tokenUsageError.value = 'Failed to load token usage data';
  } finally {
    tokenUsageLoading.value = false;
  }
}

watch(tokenUsagePeriod, () => loadTokenUsage());
watch(tokenUsageDynamicKeyFilter, () => loadTokenUsage());
watch(tokenUsageKeyHashFilter, () => loadTokenUsage());

// Sub-key methods
function maskKey(key: string) {
  if (key.length <= 16) return key;
  return `${key.slice(0, 14)}...${key.slice(-4)}`;
}

async function loadSubKeys() {
  if (!group.value) return;
  subKeysLoading.value = true;
  subKeyError.value = '';
  try {
    const params: { page?: number; limit?: number; search?: string } = {
      page: subKeyPagination.value.page,
      limit: subKeyPagination.value.rowsPerPage,
    };
    if (subKeySearch.value) params.search = subKeySearch.value;
    const result = await groupsStore.fetchGroupKeys(group.value.id, params);
    subKeys.value = result.data;
    subKeyPagination.value.rowsNumber = result.total;
  } catch {
    subKeyError.value = 'Failed to load sub-keys';
  } finally {
    subKeysLoading.value = false;
  }
}

function onSubKeyRequest(props: { pagination: { page: number; rowsPerPage: number } }) {
  subKeyPagination.value.page = props.pagination.page;
  subKeyPagination.value.rowsPerPage = props.pagination.rowsPerPage;
  loadSubKeys();
}

async function onCreateKey() {
  if (!group.value || !newKeyName.value.trim()) return;
  creatingKey.value = true;
  try {
    await groupsStore.createGroupKey(group.value.id, newKeyName.value.trim());
    showCreateKey.value = false;
    newKeyName.value = '';
    loadSubKeys();
    $q.notify({ type: 'positive', message: 'Key created' });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to create key';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    creatingKey.value = false;
  }
}

async function toggleSubKey(row: GroupKey, val: boolean) {
  if (!group.value) return;
  try {
    const updated = await groupsStore.updateGroupKey(group.value.id, row.id, { is_active: val });
    Object.assign(row, updated);
  } catch {
    $q.notify({ type: 'negative', message: 'Failed to update key status' });
  }
}

async function onRegenerateSubKey(row: GroupKey) {
  if (!group.value) return;
  $q.dialog({ title: 'Regenerate Key', message: `Regenerate key "${row.name}"? The old key will stop working.`, cancel: true })
    .onOk(async () => {
      if (!group.value) return;
      try {
        const updated = await groupsStore.regenerateGroupKey(group.value.id, row.id);
        Object.assign(row, updated);
        $q.notify({ type: 'positive', message: 'Key regenerated' });
      } catch {
        $q.notify({ type: 'negative', message: 'Failed to regenerate key' });
      }
    });
}

watch(activeTab, (tab) => {
  router.replace({ query: { ...route.query, tab } });
  if (tab === 'keys' && subKeys.value.length === 0) loadSubKeys();
  if (tab === 'allowed-models') loadAllowedModels();
  if (tab === 'token-usage') loadTokenUsage();
});

function onCbFieldClear(field: 'cb_max_failures' | 'cb_window_seconds' | 'cb_cooldown_seconds') {
  if (editServerCbForm.value[field] === null || editServerCbForm.value[field] === undefined) {
    editServerCbForm.value.cb_max_failures = null;
    editServerCbForm.value.cb_window_seconds = null;
    editServerCbForm.value.cb_cooldown_seconds = null;
  }
}

function onRlFieldClear() {
  editServerRlForm.value.max_requests = null;
  editServerRlForm.value.rate_window_seconds = null;
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

async function toggleServerEnabled(s: GroupServerDetail) {
  if (!group.value) return;
  try {
    await groupsStore.updateAssignment(group.value.id, s.server_id, { is_enabled: s.is_enabled });
    await loadGroup();
  } catch {
    s.is_enabled = !s.is_enabled;
    $q.notify({ type: 'negative', message: 'Failed to update server status' });
  }
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
  editServerCbForm.value = {
    cb_max_failures: s.cb_max_failures,
    cb_window_seconds: s.cb_window_seconds,
    cb_cooldown_seconds: s.cb_cooldown_seconds,
  };
  editServerRlForm.value = {
    max_requests: s.max_requests,
    rate_window_seconds: s.rate_window_seconds,
  };
  showEditServer.value = true;
}

async function onSaveEditServer() {
  // Validate CB fields: all-or-nothing
  const { cb_max_failures, cb_window_seconds, cb_cooldown_seconds } = editServerCbForm.value;
  const cbValues = [cb_max_failures, cb_window_seconds, cb_cooldown_seconds];
  const cbNonNull = cbValues.filter((v) => v !== null && v !== undefined);
  if (cbNonNull.length > 0 && cbNonNull.length < 3) {
    $q.notify({ type: 'negative', message: 'Circuit breaker requires all three fields or none.' });
    return;
  }
  if (cbNonNull.length === 3 && cbNonNull.some((v) => (v as number) < 1)) {
    $q.notify({ type: 'negative', message: 'Circuit breaker values must be >= 1.' });
    return;
  }

  // Validate rate limit fields: all-or-nothing
  const { max_requests, rate_window_seconds } = editServerRlForm.value;
  const rlValues = [max_requests, rate_window_seconds];
  const rlNonNull = rlValues.filter((v) => v !== null && v !== undefined);
  if (rlNonNull.length === 1) {
    $q.notify({ type: 'negative', message: 'Rate limit requires both fields or none.' });
    return;
  }
  if (rlNonNull.length === 2 && rlNonNull.some((v) => (v as number) < 1)) {
    $q.notify({ type: 'negative', message: 'Rate limit values must be >= 1.' });
    return;
  }

  savingServer.value = true;
  try {
    await serversStore.updateServer(editServerId.value, {
      name: editServerForm.value.name,
      base_url: editServerForm.value.base_url,
      api_key: editServerForm.value.api_key || null,
    });
    // Save circuit breaker fields via assignment update
    if (group.value) {
      await groupsStore.updateAssignment(group.value.id, editServerId.value, {
        cb_max_failures: editServerCbForm.value.cb_max_failures,
        cb_window_seconds: editServerCbForm.value.cb_window_seconds,
        cb_cooldown_seconds: editServerCbForm.value.cb_cooldown_seconds,
        max_requests: editServerRlForm.value.max_requests,
        rate_window_seconds: editServerRlForm.value.rate_window_seconds,
      });
    }
    showEditServer.value = false;
    loadGroup();
    loadCircuitStatus();
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

// Allowed models methods
async function loadAllowedModels() {
  if (!group.value) return;
  allowedModels.value = group.value.allowed_models || [];
  // Load master models list for the picker
  try {
    const result = await modelsStore.fetchModels({ limit: 100 });
    allModelsOptions.value = result.data
      .filter((m) => !allowedModels.value.some((am) => am.id === m.id))
      .map((m) => ({ label: m.name, value: m.id }));
    filteredModelOptions.value = allModelsOptions.value;
  } catch {
    // silently fail
  }
}

function onModelFilter(val: string, update: (fn: () => void) => void) {
  modelSearchText.value = val;
  update(() => {
    const needle = val.toLowerCase();
    filteredModelOptions.value = allModelsOptions.value.filter((o) => o.label.toLowerCase().includes(needle));
  });
}

function onNewModelValue(val: string, done: (item?: { label: string; value: string }, mode?: 'add' | 'add-unique' | 'toggle') => void) {
  // When user types a new value and presses Enter, create the model inline
  if (val.trim()) {
    onCreateAndAddModel(val.trim());
  }
  done();
}

async function onAddAllowedModel(modelId: string | null) {
  if (!group.value || !modelId) return;
  allowedModelsLoading.value = true;
  try {
    await groupsStore.addGroupAllowedModel(group.value.id, { model_id: modelId });
    modelToAdd.value = null;
    await loadGroup();
    loadAllowedModels();
    $q.notify({ type: 'positive', message: 'Model added' });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to add model';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    allowedModelsLoading.value = false;
  }
}

async function onCreateAndAddModel(name?: string) {
  const modelName = (name || modelSearchText.value).trim();
  if (!group.value || !modelName) return;
  allowedModelsLoading.value = true;
  try {
    await groupsStore.addGroupAllowedModel(group.value.id, { name: modelName });
    modelToAdd.value = null;
    modelSearchText.value = '';
    await loadGroup();
    loadAllowedModels();
    $q.notify({ type: 'positive', message: `Model "${modelName}" created and added` });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to create model';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    allowedModelsLoading.value = false;
  }
}

async function onRemoveAllowedModel(m: Model) {
  if (!group.value) return;
  allowedModelsLoading.value = true;
  try {
    await groupsStore.removeGroupAllowedModel(group.value.id, m.id);
    await loadGroup();
    loadAllowedModels();
    $q.notify({ type: 'positive', message: `"${m.name}" removed` });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to remove model';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    allowedModelsLoading.value = false;
  }
}

// Key allowed models methods
function onExpandSubKey(props: { expand: boolean; row: GroupKey }) {
  props.expand = !props.expand;
  if (props.expand) {
    if (allowedModels.value.length > 0) {
      loadKeyAllowedModels(props.row.id);
    }
    loadKeySubscriptions(props.row.id);
    loadActivePlans();
  }
}

async function loadKeySubscriptions(keyId: string) {
  if (!group.value) return;
  try {
    const { data } = await api.get<KeySubscription[]>(`/api/admin/groups/${group.value.id}/keys/${keyId}/subscriptions`);
    keySubscriptions.value[keyId] = data;
  } catch { /* ignore */ }
}

async function loadActivePlans() {
  if (activePlans.value.length > 0) return;
  try {
    const { data } = await api.get<SubscriptionPlan[]>('/api/admin/subscription-plans');
    activePlans.value = data.filter((p) => p.is_active);
  } catch { /* ignore */ }
}

async function onAssignSubscription(keyId: string, planId: string) {
  if (!group.value || !planId) return;
  try {
    await api.post(`/api/admin/groups/${group.value.id}/keys/${keyId}/subscriptions`, { plan_id: planId });
    loadKeySubscriptions(keyId);
    $q.notify({ type: 'positive', message: 'Subscription assigned' });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to assign';
    $q.notify({ type: 'negative', message: msg });
  }
}

async function onCancelSubscription(keyId: string, subId: string) {
  if (!group.value) return;
  try {
    await api.patch(`/api/admin/groups/${group.value.id}/keys/${keyId}/subscriptions/${subId}`, { status: 'cancelled' });
    loadKeySubscriptions(keyId);
    $q.notify({ type: 'positive', message: 'Subscription cancelled' });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to cancel';
    $q.notify({ type: 'negative', message: msg });
  }
}

function getKeyAllowedModels(keyId: string): Model[] {
  return keyAllowedModelsMap.value[keyId] || [];
}

function keyModelOptions(keyId: string) {
  const keyModels = getKeyAllowedModels(keyId);
  return allowedModels.value
    .filter((m) => !keyModels.some((km) => km.id === m.id))
    .map((m) => ({ label: m.name, value: m.id }));
}

async function loadKeyAllowedModels(keyId: string) {
  if (!group.value) return;
  try {
    const models = await groupsStore.fetchKeyAllowedModels(group.value.id, keyId);
    keyAllowedModelsMap.value[keyId] = models;
  } catch {
    // silently fail
  }
}

async function onAddKeyModel(keyId: string, modelId: string) {
  if (!group.value || !modelId) return;
  keyModelsLoading.value[keyId] = true;
  try {
    await groupsStore.addKeyAllowedModel(group.value.id, keyId, modelId);
    await loadKeyAllowedModels(keyId);
    const modelName = allowedModels.value.find((m) => m.id === modelId)?.name || modelId;
    $q.notify({ type: 'positive', message: `"${modelName}" added to key` });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to add model to key';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    keyModelsLoading.value[keyId] = false;
  }
}

async function onRemoveKeyModel(keyId: string, modelId: string) {
  if (!group.value) return;
  keyModelsLoading.value[keyId] = true;
  try {
    const modelName = getKeyAllowedModels(keyId).find((m) => m.id === modelId)?.name || modelId;
    await groupsStore.removeKeyAllowedModel(group.value.id, keyId, modelId);
    await loadKeyAllowedModels(keyId);
    $q.notify({ type: 'positive', message: `"${modelName}" removed from key` });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to remove model from key';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    keyModelsLoading.value[keyId] = false;
  }
}

// Rate badge helpers
function hasNonDefaultRate(s: GroupServerDetail): boolean {
  return [s.rate_input, s.rate_output, s.rate_cache_write, s.rate_cache_read].some(
    (r) => r !== null && r !== undefined && r !== 1.0,
  );
}

function displayRate(s: GroupServerDetail): string {
  const rates = [s.rate_input, s.rate_output, s.rate_cache_write, s.rate_cache_read];
  const nonDefault = rates.filter((r) => r !== null && r !== undefined && r !== 1.0);
  if (nonDefault.length === 0) return '1.0';
  const allSame = nonDefault.length === rates.length && nonDefault.every((r) => r === nonDefault[0]);
  if (allSame) return String(nonDefault[0]);
  return 'custom';
}

function openRateModal(s: GroupServerDetail) {
  rateEditServer.value = s;
  rateForm.value = {
    rate_input: s.rate_input,
    rate_output: s.rate_output,
    rate_cache_write: s.rate_cache_write,
    rate_cache_read: s.rate_cache_read,
  };
  showRateModal.value = true;
}

async function onSaveRates() {
  if (!group.value || !rateEditServer.value) return;
  for (const field of ['rate_input', 'rate_output', 'rate_cache_write', 'rate_cache_read'] as const) {
    if (rateForm.value[field] !== null && rateForm.value[field] !== undefined && (rateForm.value[field] as number) < 0) {
      $q.notify({ type: 'negative', message: `${field} must be non-negative` });
      return;
    }
  }
  savingRate.value = true;
  try {
    await groupsStore.updateAssignment(group.value.id, rateEditServer.value.server_id, {
      rate_input: rateForm.value.rate_input,
      rate_output: rateForm.value.rate_output,
      rate_cache_write: rateForm.value.rate_cache_write,
      rate_cache_read: rateForm.value.rate_cache_read,
    });
    showRateModal.value = false;
    await loadGroup();
    $q.notify({ type: 'positive', message: 'Rates saved' });
  } catch (e: unknown) {
    const msg = (e as { response?: { data?: { error?: string } } })?.response?.data?.error || 'Failed to save rates';
    $q.notify({ type: 'negative', message: msg });
  } finally {
    savingRate.value = false;
  }
}

async function loadTtftKeys() {
  if (!group.value) return;
  if (subKeys.value.length === 0) {
    try {
      const data = await groupsStore.fetchGroupKeys(group.value.id, { limit: 100 });
      subKeys.value = data.data;
    } catch {
      // silently fail — dropdown just shows "All keys"
    }
  }
}
</script>

<style scoped>
.disabled-server {
  opacity: 0.5;
}
</style>
