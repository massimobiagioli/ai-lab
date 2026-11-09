import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Card } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';

@Component({
  selector: 'app-not-found',
  imports: [RouterLink, PageHeader, Card, Button],
  template: `
    <app-page-header title="404" subtitle="That route does not exist" />
    <app-card title="Lost in the lab">
      <p>The page you requested is not part of this workspace.</p>
      <a routerLink="/"><app-button variant="primary">Back to dashboard</app-button></a>
    </app-card>
  `,
})
export class NotFound {}
