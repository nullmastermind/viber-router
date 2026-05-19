import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import { defineComponent, h } from 'vue';
import type { KeyTokenUsage } from '../stores/groups';
import { subKeyUsageColumns } from './subKeyUsageColumns';

type Col = (typeof subKeyUsageColumns)[number] & {
  field?: string | ((row: KeyTokenUsage) => unknown);
  format?: (v: unknown, row: KeyTokenUsage) => unknown;
};

// Minimal table renderer that mirrors how QTable consumes `columns`:
// each column has a `field` (string or fn) and optional `format(value, row)`.
// We avoid mounting QTable directly so the test stays free of Quasar plugin setup.
const TestTable = defineComponent({
  props: {
    columns: { type: Array, required: true },
    rows: { type: Array, required: true },
  },
  setup(props) {
    return () =>
      h('table', [
        h('thead', [
          h(
            'tr',
            (props.columns as Col[]).map((c) =>
              h('th', { 'data-col': c.name }, String(c.label)),
            ),
          ),
        ]),
        h(
          'tbody',
          (props.rows as KeyTokenUsage[]).map((row) =>
            h(
              'tr',
              (props.columns as Col[]).map((c) => {
                const raw =
                  typeof c.field === 'function'
                    ? c.field(row)
                    : c.field
                      ? (row as unknown as Record<string, unknown>)[c.field]
                      : '';
                const cell = c.format ? c.format(raw, row) : raw;
                return h('td', { 'data-col': c.name }, String(cell ?? ''));
              }),
            ),
          ),
        ),
      ]);
  },
});

const sampleRows: KeyTokenUsage[] = [
  {
    group_key_id: '00000000-0000-0000-0000-000000000001',
    key_name: 'alpha-key',
    api_key: 'sk-aaa',
    created_at: '2026-05-01T00:00:00Z',
    total_input_tokens: 1_234_000,
    total_output_tokens: 5_000,
    total_cache_creation_tokens: 0,
    total_cache_read_tokens: 0,
    request_count: 42,
    cost_usd: 1.2345,
    peak_tpm: 87_500,
  },
  {
    group_key_id: '00000000-0000-0000-0000-000000000002',
    key_name: 'beta-key',
    api_key: 'sk-bbb',
    created_at: '2026-05-02T00:00:00Z',
    total_input_tokens: 100,
    total_output_tokens: 200,
    total_cache_creation_tokens: 0,
    total_cache_read_tokens: 0,
    request_count: 3,
    cost_usd: null,
    peak_tpm: 1_500_000,
  },
];

describe('subKeyUsageColumns — Peak TPM column', () => {
  it('includes a Peak TPM column bound to peak_tpm with compact formatting', () => {
    const col = subKeyUsageColumns.find((c) => c.name === 'peak_tpm');
    expect(col, 'peak_tpm column must exist').toBeDefined();
    expect(col?.label).toBe('Peak TPM');
    expect(col?.field).toBe('peak_tpm');
    expect(col?.sortable).toBe(true);
    expect(col?.align).toBe('right');
  });

  it('renders Peak TPM header and per-row values with compact format', () => {
    const wrapper = mount(TestTable, {
      props: { columns: subKeyUsageColumns, rows: sampleRows },
    });

    const header = wrapper.find('th[data-col="peak_tpm"]');
    expect(header.exists()).toBe(true);
    expect(header.text()).toBe('Peak TPM');

    const cells = wrapper.findAll('td[data-col="peak_tpm"]');
    expect(cells).toHaveLength(2);
    // 87_500 -> "87.5K", 1_500_000 -> "1.5M" via Intl.NumberFormat compact
    expect(cells[0]?.text()).toBe('87.5K');
    expect(cells[1]?.text()).toBe('1.5M');
  });

  it('positions Peak TPM between Requests and Cost', () => {
    const names = subKeyUsageColumns.map((c) => c.name);
    const reqIdx = names.indexOf('requests');
    const peakIdx = names.indexOf('peak_tpm');
    const costIdx = names.indexOf('cost');
    expect(peakIdx).toBe(reqIdx + 1);
    expect(costIdx).toBe(peakIdx + 1);
  });
});
