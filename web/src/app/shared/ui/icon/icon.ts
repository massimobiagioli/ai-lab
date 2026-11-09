import { Component, input } from '@angular/core';

export type IconName =
  | 'dashboard'
  | 'runs'
  | 'office'
  | 'board'
  | 'agents'
  | 'projects'
  | 'providers'
  | 'memory'
  | 'settings'
  | 'close'
  | 'menu';

const PATHS: Record<IconName, string> = {
  dashboard: 'M1 1h6v6H1zM9 1h6v6H9zM1 9h6v6H1zM9 9h6v6H9z',
  runs: 'M3 1l5 4-5 4zM2 10h12v2H2z',
  office: 'M2 2h12v12H2zM4 4h3v3H4zM9 4h3v3H9zM6 9h4v3H6z',
  board: 'M1 1h4v14H1zM6 1h4v9H6zM11 1h4v6h-4z',
  agents: 'M6 2h4v4H6zM4 7h8v7H4zM2 8h2v4H2zM12 8h2v4h-2z',
  projects: 'M2 3h5l2 2h5v9H2z',
  providers: 'M7 1h2v3h3v2H9v3h3v2H9v3H7v-3H4v-2h3V6H4V4h3z',
  memory: 'M3 2h10v12H3zM5 4h6v2H5zM5 8h6v2H5zM5 12h4v1H5z',
  settings: 'M7 5a3 3 0 100 6 3 3 0 000-6zM1 8h2M13 8h2M8 1v2M8 13v2',
  close: 'M3 3l10 10M13 3L3 13',
  menu: 'M2 3h12v2H2zM2 7h12v2H2zM2 11h12v2H2z',
};

@Component({
  selector: 'app-icon',
  template: `
    <svg
      class="ui-icon"
      viewBox="0 0 16 16"
      width="16"
      height="16"
      aria-hidden="true"
      shape-rendering="crispEdges"
    >
      <path [attr.d]="path()" fill="currentColor" />
    </svg>
  `,
  styles: `
    .ui-icon {
      display: inline-block;
      vertical-align: middle;
    }
  `,
})
export class Icon {
  readonly name = input.required<IconName>();

  protected path(): string {
    return PATHS[this.name()];
  }
}
