import { Component, input, output } from '@angular/core';

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
          <tr
            tabindex="0"
            (click)="rowSelected.emit($index)"
            (keydown.enter)="rowSelected.emit($index)"
          >
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
      font-family: var(--ailab-font-sans);
      font-size: var(--ailab-font-size-md);
    }
    .pixel-table__caption {
      text-align: left;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 650;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      padding-bottom: var(--ailab-space-2);
    }
    .pixel-table th,
    .pixel-table td {
      text-align: left;
      padding: 0.85rem 0.75rem;
      border-bottom: 1px solid var(--ailab-color-border);
    }
    .pixel-table th {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 650;
      letter-spacing: 0.06em;
      text-transform: uppercase;
    }
    .pixel-table td {
      color: var(--ailab-color-text-soft);
    }
    .pixel-table tbody tr {
      cursor: pointer;
      transition: background var(--ailab-motion-fast) ease;
    }
    .pixel-table tbody tr:hover,
    .pixel-table tbody tr:focus-visible {
      background: var(--ailab-color-surface-raised);
      outline: none;
    }
  `,
})
export class PixelTable {
  readonly caption = input.required<string>();
  readonly columns = input.required<readonly TableColumn[]>();
  readonly rows = input.required<readonly Readonly<Record<string, string>>[]>();
  readonly rowSelected = output<number>();
}
