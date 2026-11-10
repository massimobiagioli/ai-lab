import { Component, computed, input } from '@angular/core';

@Component({
  selector: 'app-avatar',
  template: `<span class="ui-avatar" [attr.aria-label]="name()">{{ initials() }}</span>`,
  styles: `
    .ui-avatar {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 32px;
      height: 32px;
      border: 1px solid var(--ailab-color-accent-soft);
      border-radius: var(--ailab-radius-pill);
      background: var(--ailab-color-accent-soft);
      color: var(--ailab-color-accent);
      font-size: var(--ailab-font-size-sm);
      font-weight: 700;
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
