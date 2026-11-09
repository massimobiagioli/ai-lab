import { Component, input } from '@angular/core';

export interface TableColumn {
  readonly key: string;
  readonly label: string;
}

@Component({
  selector: 'app-pixel-table',
  template: `
    <table class="pixel-table">
      <caption class="pixel-table__caption">
        {{
          caption()
        }}
      </caption>
      <thead>
        <tr>
          @for (column of columns(); track column.key) {
            <th scope="col">{{ column.label }}</th>
          }
        </tr>
      </thead>
      <tbody>
        @for (row of rows(); track $index) {
          <tr>
            @for (column of columns(); track column.key) {
              <td>{{ row[column.key] ?? '' }}</td>
            }
          </tr>
        }
      </tbody>
    </table>
  `,
  styles: `
    .pixel-table {
      width: 100%;
      border-collapse: collapse;
      font-size: var(--ailab-font-size-md);
    }
    .pixel-table__caption {
      text-align: left;
      color: var(--ailab-color-muted);
      padding-bottom: var(--ailab-space-2);
    }
    .pixel-table th,
    .pixel-table td {
      text-align: left;
      padding: var(--ailab-space-2);
      border-bottom: 1px solid var(--ailab-color-surface-raised);
    }
  `,
})
export class PixelTable {
  readonly caption = input.required<string>();
  readonly columns = input.required<readonly TableColumn[]>();
  readonly rows = input.required<readonly Readonly<Record<string, string>>[]>();
}
