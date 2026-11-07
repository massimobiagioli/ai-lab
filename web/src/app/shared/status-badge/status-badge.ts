import { Component, input } from '@angular/core';

export type StatusTone = 'neutral' | 'success' | 'warning' | 'danger';

@Component({
  selector: 'app-status-badge',
  templateUrl: './status-badge.html',
  styleUrl: './status-badge.scss',
})
export class StatusBadge {
  readonly label = input.required<string>();
  readonly tone = input<StatusTone>('neutral');
}
