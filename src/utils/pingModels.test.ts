import { describe, expect, it } from 'vitest';
import { formatPingStatus, pingModelOptions } from './pingModels';

describe('pingModelOptions', () => {
  it('uses supported models when the assignment lists them', () => {
    const options = pingModelOptions({
      supportedModels: ['claude-sonnet-4-6'],
      allowedModels: ['claude-opus-4-6'],
      allModels: ['claude-haiku-4-5'],
      mappings: {},
    });
    expect(options.map((o) => o.name)).toEqual(['claude-sonnet-4-6']);
  });

  it('falls back to group allowed models when supported is empty', () => {
    const options = pingModelOptions({
      supportedModels: [],
      allowedModels: ['claude-opus-4-6'],
      allModels: ['claude-haiku-4-5'],
      mappings: {},
    });
    expect(options.map((o) => o.name)).toEqual(['claude-opus-4-6']);
  });

  it('falls back to all models when neither supported nor allowed is set', () => {
    const options = pingModelOptions({
      supportedModels: [],
      allowedModels: [],
      allModels: ['claude-haiku-4-5', 'claude-sonnet-4-6'],
      mappings: {},
    });
    expect(options.map((o) => o.name)).toEqual(['claude-haiku-4-5', 'claude-sonnet-4-6']);
  });

  it('includes mapping source keys and shows the mapped upstream name', () => {
    const options = pingModelOptions({
      supportedModels: ['claude-sonnet-4-6'],
      allowedModels: [],
      allModels: [],
      mappings: { 'gpt-4o': 'claude-sonnet-4-6', 'claude-sonnet-4-6': 'my-sonnet' },
    });
    expect(options).toEqual([
      { name: 'claude-sonnet-4-6', mapped: 'my-sonnet' },
      { name: 'gpt-4o', mapped: 'claude-sonnet-4-6' },
    ]);
  });
});

describe('formatPingStatus', () => {
  it('is blank while idle', () => {
    expect(formatPingStatus({ state: 'idle' })).toBe('');
  });

  it('says pinging while a request is in flight', () => {
    expect(formatPingStatus({ state: 'pending' })).toBe('Pinging…');
  });

  it('shows TTFT when the server is alive', () => {
    expect(
      formatPingStatus({
        state: 'ok',
        ttftMs: 123,
        model: 'claude-sonnet-4-6',
        mappedModel: 'claude-sonnet-4-6',
      }),
    ).toBe('Alive · 123 ms');
  });

  it('shows the mapped upstream name when it differs', () => {
    expect(
      formatPingStatus({
        state: 'ok',
        ttftMs: 80,
        model: 'claude-sonnet-4-6',
        mappedModel: 'my-sonnet',
      }),
    ).toBe('Alive · 80 ms → my-sonnet');
  });

  it('shows the error body when the server is dead', () => {
    expect(formatPingStatus({ state: 'error', status: 429, error: 'rate limited' })).toBe(
      'Dead · rate limited',
    );
  });

  it('falls back to unreachable when there is no HTTP status', () => {
    expect(formatPingStatus({ state: 'error', status: 0, error: null })).toBe('Dead · unreachable');
  });
});
