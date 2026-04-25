/**
 * Composable for subscription type labels and tooltips
 */

export interface SubTypeOption {
  label: string;
  value: string;
  tooltip: string;
}

/**
 * Get human-readable label for a subscription type
 */
export function getSubTypeLabel(subType: string): string {
  switch (subType) {
    case 'fixed':
      return 'Fixed Quota';
    case 'hourly_reset':
      return 'Hourly Reset';
    case 'pay_per_request':
      return 'Pay Per Request';
    case 'bonus':
      return 'Bonus';
    default:
      return subType; // Fallback to raw value for unknown types
  }
}

/**
 * Get tooltip description for a subscription type
 */
export function getSubTypeTooltip(subType: string): string {
  switch (subType) {
    case 'fixed':
      return 'Budget does not reset. Once exhausted, the subscription ends.';
    case 'hourly_reset':
      return 'Budget resets every X hours. Usage counter resets at the start of each window.';
    case 'pay_per_request':
      return 'Each request costs a flat rate based on the model. Budget can optionally reset.';
    case 'bonus':
      return 'External API subscription with its own server. Tried before group servers.';
    default:
      return '';
  }
}

/**
 * Get all subscription type options with labels and tooltips
 */
export function getSubTypeOptions(): SubTypeOption[] {
  return [
    {
      label: 'Fixed Quota',
      value: 'fixed',
      tooltip: 'Budget does not reset. Once exhausted, the subscription ends.',
    },
    {
      label: 'Hourly Reset',
      value: 'hourly_reset',
      tooltip: 'Budget resets every X hours. Usage counter resets at the start of each window.',
    },
    {
      label: 'Pay Per Request',
      value: 'pay_per_request',
      tooltip: 'Each request costs a flat rate based on the model. Budget can optionally reset.',
    },
    {
      value: 'bonus',
      label: 'Bonus',
      tooltip: 'External API subscription with its own server. Tried before group servers.',
    },
  ];
}
