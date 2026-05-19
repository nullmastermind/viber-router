import type { KeyTokenUsage } from 'stores/groups';

const formatCompact = (v: number) =>
  new Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 2 }).format(v);

export const formatCost = (v: number | null) => (v != null ? `$${v.toFixed(4)}` : '—');

export const subKeyUsageColumns = [
  { name: 'index', label: '#', field: () => '', align: 'center' as const },
  {
    name: 'key_name',
    label: 'Key Name',
    field: 'key_name',
    align: 'left' as const,
    sortable: true,
    format: (v: string | null, row: KeyTokenUsage) =>
      v != null ? v : row.group_key_id == null ? 'Master / Dynamic Keys' : 'Deleted Key',
  },
  {
    name: 'input',
    label: 'Input Tokens',
    field: 'total_input_tokens',
    align: 'right' as const,
    sortable: true,
    format: formatCompact,
  },
  {
    name: 'output',
    label: 'Output Tokens',
    field: 'total_output_tokens',
    align: 'right' as const,
    sortable: true,
    format: formatCompact,
  },
  {
    name: 'cache_creation',
    label: 'Cache Write',
    field: 'total_cache_creation_tokens',
    align: 'right' as const,
    sortable: true,
    format: formatCompact,
  },
  {
    name: 'cache_read',
    label: 'Cache Read',
    field: 'total_cache_read_tokens',
    align: 'right' as const,
    sortable: true,
    format: formatCompact,
  },
  {
    name: 'requests',
    label: 'Requests',
    field: 'request_count',
    align: 'right' as const,
    sortable: true,
    format: formatCompact,
  },
  {
    name: 'peak_tpm',
    label: 'Peak TPM',
    field: 'peak_tpm',
    align: 'right' as const,
    sortable: true,
    format: formatCompact,
  },
  {
    name: 'cost',
    label: 'Cost ($)',
    field: 'cost_usd',
    align: 'right' as const,
    sortable: true,
    format: (v: number | null) => (v != null ? `$${v.toFixed(4)}` : '—'),
  },
  {
    name: 'created_at',
    label: 'Created At',
    field: 'created_at',
    align: 'left' as const,
    format: (v: string | null) => (v != null ? v.slice(0, 10) : '—'),
  },
  { name: 'actions', label: '', field: 'group_key_id', align: 'right' as const },
];
