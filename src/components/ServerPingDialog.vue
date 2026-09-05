<template>
  <q-dialog :model-value="modelValue" @update:model-value="emit('update:modelValue', $event)">
    <q-card style="width: 560px; max-width: 95vw">
      <q-card-section>
        <div class="text-h6">Ping {{ serverName }}</div>
        <div class="text-caption text-grey">
          Measures liveness and TTFT. Model mappings are applied before the request.
        </div>
      </q-card-section>
      <q-card-section class="q-pt-none">
        <div v-if="!models.length" class="text-grey">No models to ping</div>
        <q-list v-else separator>
          <q-item v-for="m in models" :key="m.name">
            <q-item-section>
              <q-item-label>{{ m.name }}</q-item-label>
              <q-item-label v-if="m.mapped" caption>→ {{ m.mapped }}</q-item-label>
              <q-item-label
                v-if="formatPingStatus(statusFor(m.name))"
                caption
                :class="{
                  'text-positive': statusFor(m.name).state === 'ok',
                  'text-negative': statusFor(m.name).state === 'error',
                }"
              >
                {{ formatPingStatus(statusFor(m.name)) }}
              </q-item-label>
            </q-item-section>
            <q-item-section side>
              <q-btn
                dense
                unelevated
                color="primary"
                label="Ping"
                :loading="statusFor(m.name).state === 'pending'"
                :disable="busy"
                :aria-label="`Ping ${m.name}`"
                @click="emit('ping', m.name)"
              />
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat label="Close" v-close-popup />
        <q-btn
          color="primary"
          label="Ping all"
          :disable="!models.length || busy"
          :loading="pingingAll"
          @click="emit('pingAll')"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { formatPingStatus, type PingModelOption, type PingRowStatus } from 'src/utils/pingModels';

const props = defineProps<{
  modelValue: boolean;
  serverName: string;
  models: PingModelOption[];
  results: Record<string, PingRowStatus>;
  pingingAll: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  ping: [model: string];
  pingAll: [];
}>();

const busy = computed(
  () => props.pingingAll || Object.values(props.results).some((r) => r.state === 'pending'),
);

function statusFor(name: string): PingRowStatus {
  return props.results[name] ?? { state: 'idle' };
}
</script>
