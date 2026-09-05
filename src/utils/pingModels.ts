export interface PingModelOption {
  name: string;
  mapped?: string;
}

export function pingModelOptions(input: {
  supportedModels: string[];
  allowedModels: string[];
  allModels: string[];
  mappings: Record<string, string>;
}): PingModelOption[] {
  const base = input.supportedModels.length
    ? input.supportedModels
    : input.allowedModels.length
      ? input.allowedModels
      : input.allModels;

  const names = new Set(base);
  for (const key of Object.keys(input.mappings)) {
    if (key) names.add(key);
  }

  return [...names]
    .sort((a, b) => a.localeCompare(b))
    .map((name) => {
      const mapped = input.mappings[name];
      return mapped && mapped !== name ? { name, mapped } : { name };
    });
}

export type PingRowStatus =
  | { state: 'idle' }
  | { state: 'pending' }
  | { state: 'ok'; ttftMs: number; model: string; mappedModel: string }
  | { state: 'error'; status: number; error?: string | null };

export function formatPingStatus(status: PingRowStatus): string {
  switch (status.state) {
    case 'idle':
      return '';
    case 'pending':
      return 'Pinging…';
    case 'ok': {
      const mapped = status.mappedModel !== status.model ? ` → ${status.mappedModel}` : '';
      return `Alive · ${status.ttftMs} ms${mapped}`;
    }
    case 'error': {
      const detail = status.error || (status.status ? String(status.status) : 'unreachable');
      return `Dead · ${detail}`;
    }
  }
}
