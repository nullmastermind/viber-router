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
  ];
}
