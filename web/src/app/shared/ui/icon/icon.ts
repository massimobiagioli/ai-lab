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
  dashboard: 'M4 4h6v6H4zM14 4h6v4h-6zM14 12h6v8h-6zM4 14h6v6H4z',
  runs: 'M8 5v14l11-7zM4 5v14',
  office: 'M4 20V9l8-5 8 5v11M9 20v-6h6v6',
  board: 'M4 4h4v16H4zM10 4h4v10h-4zM16 4h4v7h-4z',
  agents: 'M12 12a4 4 0 100-8 4 4 0 000 8zM4 21a8 8 0 0116 0',
  projects: 'M3 7a2 2 0 012-2h5l2 2h7a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2z',
  providers: 'M13 2L4 14h6l-1 8 9-12h-6z',
  memory: 'M6 4h12v16H6zM9 8h6M9 12h6M9 16h4',
  settings:
    'M12 15.5a3.5 3.5 0 100-7 3.5 3.5 0 000 7zM19.4 15a1.7 1.7 0 00.34 1.88l.06.06-1.9 1.9-.06-.06a1.7 1.7 0 00-1.88-.34 1.7 1.7 0 00-1.03 1.56V20h-2.68v-.1a1.7 1.7 0 00-1.03-1.56 1.7 1.7 0 00-1.88.34l-.06.06-1.9-1.9.06-.06A1.7 1.7 0 006.6 15a1.7 1.7 0 00-1.56-1.03H5v-2.68h.1A1.7 1.7 0 006.66 10a1.7 1.7 0 00-.34-1.88l-.06-.06 1.9-1.9.06.06a1.7 1.7 0 001.88.34A1.7 1.7 0 0011.13 5V4.9h2.68V5a1.7 1.7 0 001.03 1.56 1.7 1.7 0 001.88-.34l.06-.06 1.9 1.9-.06.06A1.7 1.7 0 0018.28 10a1.7 1.7 0 001.56 1.03H20v2.68h-.1A1.7 1.7 0 0018.34 15z',
  close: 'M6 6l12 12M18 6L6 18',
  menu: 'M4 7h16M4 12h16M4 17h16',
};

@Component({
  selector: 'app-icon',
  template: `
    <svg
      class="ui-icon"
      viewBox="0 0 24 24"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path [attr.d]="path()" />
    </svg>
  `,
  styles: `
    .ui-icon {
      display: inline-block;
      flex: none;
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
