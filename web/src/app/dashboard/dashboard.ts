import { Component, computed, input } from '@angular/core';
import { StatusBadge } from '../shared/status-badge/status-badge';

export interface FlowMetrics {
  readonly leadTimeP50Minutes: number;
  readonly leadTimeP95Minutes: number;
  readonly cycleTimeP50Minutes: number;
  readonly wip: number;
  readonly throughputWeekly: number;
  readonly blockedShare: number;
}

@Component({
  selector: 'app-dashboard',
  imports: [StatusBadge],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.scss',
})
export class Dashboard {
  readonly metrics = input.required<FlowMetrics>();
  readonly health = computed(() => (this.metrics().blockedShare > 0.1 ? 'warning' : 'success'));
}
