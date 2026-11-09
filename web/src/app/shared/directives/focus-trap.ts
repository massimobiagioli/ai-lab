import { AfterViewInit, Directive, ElementRef, inject, OnDestroy } from '@angular/core';

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/** Traps Tab focus inside the host and restores focus on destroy. */
@Directive({ selector: '[appFocusTrap]' })
export class FocusTrap implements AfterViewInit, OnDestroy {
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  private previous: HTMLElement | null = null;
  private readonly onKeydown = (event: KeyboardEvent): void => {
    if (event.key !== 'Tab') {
      return;
    }
    const focusable = this.focusable();
    if (focusable.length === 0) {
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (first === undefined || last === undefined) {
      return;
    }
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  };

  ngAfterViewInit(): void {
    this.previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    document.addEventListener('keydown', this.onKeydown);
    queueMicrotask(() => this.focusable()[0]?.focus());
  }

  ngOnDestroy(): void {
    document.removeEventListener('keydown', this.onKeydown);
    this.previous?.focus();
  }

  private focusable(): HTMLElement[] {
    return Array.from(this.host.nativeElement.querySelectorAll<HTMLElement>(FOCUSABLE));
  }
}
