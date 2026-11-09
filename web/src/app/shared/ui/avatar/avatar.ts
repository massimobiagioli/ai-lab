import { Component, computed, input } from '@angular/core';

@Component({
  selector: 'app-avatar',
  template: `<span class="ui-avatar" [attr.aria-label]="name()">{{ initials() }}</span>`,
  styles: `
    .ui-avatar {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 28px;
      height: 28px;
      border: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface-raised);
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-sm);
      text-transform: uppercase;
    }
  `,
})
export class Avatar {
  readonly name = input.required<string>();
  protected readonly initials = computed(() =>
    this.name()
      .split(/\s+/)
      .map((part) => part.charAt(0))
      .join('')
      .slice(0, 2),
  );
}
