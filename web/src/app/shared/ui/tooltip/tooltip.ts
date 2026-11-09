import { Directive, ElementRef, inject, input, OnChanges } from '@angular/core';

/** Accessible tooltip via the native `title` attribute. */
@Directive({ selector: '[appTooltip]' })
export class Tooltip implements OnChanges {
  private readonly host = inject<ElementRef<HTMLElement>>(ElementRef);
  readonly appTooltip = input.required<string>();

  ngOnChanges(): void {
    this.host.nativeElement.setAttribute('title', this.appTooltip());
  }
}
