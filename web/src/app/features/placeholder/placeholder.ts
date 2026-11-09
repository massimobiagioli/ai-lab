import { Component, input } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Card } from '../../shared/ui/card/card';

@Component({
  selector: 'app-placeholder',
  imports: [PageHeader, Card],
  template: `
    <app-page-header [title]="title()" [subtitle]="body()" />
    <app-card title="Platform ready">
      <p>
        The route and shell are wired. The functional surface is delivered by WP-18 (office) and
        WP-19 (product UI) on top of this platform.
      </p>
    </app-card>
  `,
})
export class Placeholder {
  readonly title = input.required<string>();
  readonly body = input.required<string>();
}
