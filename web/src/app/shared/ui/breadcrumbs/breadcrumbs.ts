import { Component, input } from '@angular/core';
import { RouterLink } from '@angular/router';

export interface Crumb {
  readonly label: string;
  readonly path: string;
}

@Component({
  selector: 'app-breadcrumbs',
  imports: [RouterLink],
  template: `
    <nav class="ui-crumbs" aria-label="Breadcrumb">
      <ol class="ui-crumbs__list">
        @for (crumb of items(); track crumb.path; let last = $last) {
          <li class="ui-crumbs__item">
            @if (!last) {
              <a [routerLink]="crumb.path">{{ crumb.label }}</a>
              <span aria-hidden="true">/</span>
            } @else {
              <span aria-current="page">{{ crumb.label }}</span>
            }
          </li>
        }
      </ol>
    </nav>
  `,
  styles: `
    .ui-crumbs__list {
      display: flex;
      flex-wrap: wrap;
      gap: var(--ailab-space-2);
      list-style: none;
      margin: 0 0 var(--ailab-space-3);
      padding: 0;
      font-size: var(--ailab-font-size-sm);
      color: var(--ailab-color-muted);
    }
    .ui-crumbs__item {
      display: flex;
      gap: var(--ailab-space-2);
    }
    a {
      color: var(--ailab-color-accent);
    }
  `,
})
export class Breadcrumbs {
  readonly items = input.required<readonly Crumb[]>();
}
