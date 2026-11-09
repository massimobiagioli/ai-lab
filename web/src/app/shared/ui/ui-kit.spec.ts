import { Type } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { Button } from './button/button';
import { Card } from './card/card';
import { EmptyState } from './empty-state/empty-state';
import { Spinner } from './spinner/spinner';
import { PageHeader } from './page-header/page-header';
import { Drawer } from './drawer/drawer';
import { Input } from './input/input';
import { Select } from './select/select';
import { Tabs } from './tabs/tabs';
import { Dialog } from './dialog/dialog';
import { Toast } from './toast/toast';
import { Breadcrumbs } from './breadcrumbs/breadcrumbs';
import { Pagination } from './pagination/pagination';
import { Skeleton } from './skeleton/skeleton';
import { Progress } from './progress/progress';
import { Avatar } from './avatar/avatar';
import { Icon } from './icon/icon';
import { Menu } from './menu/menu';
import { Divider } from './divider/divider';
import { Kbd } from './kbd/kbd';
import { PixelPanel } from './pixel-panel/pixel-panel';
import { PixelTable } from './pixel-table/pixel-table';
import { PixelIconButton } from './pixel-icon-button/pixel-icon-button';
import { PixelTextarea } from './pixel-textarea/pixel-textarea';
import { PixelCheckbox } from './pixel-checkbox/pixel-checkbox';

async function render<T>(component: Type<T>, inputs: Record<string, unknown>) {
  TestBed.resetTestingModule();
  await TestBed.configureTestingModule({
    imports: [component],
    providers: [provideRouter([])],
  }).compileComponents();
  const fixture = TestBed.createComponent(component);
  for (const [key, value] of Object.entries(inputs)) {
    fixture.componentRef.setInput(key, value);
  }
  fixture.detectChanges();
  return fixture;
}

describe('UI kit', () => {
  it('Button disables while loading', async () => {
    const fixture = await render(Button, { loading: true });
    const button = (fixture.nativeElement as HTMLElement).querySelector('button');
    expect(button?.disabled).toBe(true);
    expect(button?.getAttribute('aria-busy')).toBe('true');
  });

  it('Input links the error with aria-describedby', async () => {
    const fixture = await render(Input, { label: 'Name', error: 'Required' });
    const input = (fixture.nativeElement as HTMLElement).querySelector('input');
    const describedBy = input?.getAttribute('aria-describedby');
    expect(describedBy).toBeTruthy();
    expect(
      (fixture.nativeElement as HTMLElement).querySelector(`#${describedBy}`)?.textContent,
    ).toContain('Required');
  });

  it('Input supports disabled', async () => {
    const fixture = await render(Input, { label: 'Name', disabled: true });
    expect((fixture.nativeElement as HTMLElement).querySelector('input')?.disabled).toBe(true);
  });

  it('PixelTextarea supports disabled', async () => {
    const fixture = await render(PixelTextarea, { label: 'Notes', disabled: true });
    expect((fixture.nativeElement as HTMLElement).querySelector('textarea')?.disabled).toBe(true);
  });

  it('Select supports disabled and error', async () => {
    const fixture = await render(Select, {
      label: 'Scope',
      options: [{ value: 'all', label: 'All' }],
      disabled: true,
      error: 'Choose one',
    });
    const select = (fixture.nativeElement as HTMLElement).querySelector('select');
    expect(select?.disabled).toBe(true);
    expect(select?.getAttribute('aria-invalid')).toBe('true');
  });

  it('Tabs moves with ArrowRight', async () => {
    const fixture = await render(Tabs, {
      label: 'Tabs',
      tabs: [
        { id: 'a', label: 'A' },
        { id: 'b', label: 'B' },
      ],
      active: 'a',
    });
    const tablist = (fixture.nativeElement as HTMLElement).querySelector('[role="tablist"]');
    tablist?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
    fixture.detectChanges();
    expect(fixture.componentInstance.active()).toBe('b');
  });

  it('Menu moves with ArrowDown', async () => {
    const fixture = await render(Menu, {
      label: 'Menu',
      items: [
        { id: 'a', label: 'A' },
        { id: 'b', label: 'B' },
      ],
    });
    const menu = (fixture.nativeElement as HTMLElement).querySelector('[role="menu"]');
    menu?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown' }));
    fixture.detectChanges();
    const items = (fixture.nativeElement as HTMLElement).querySelectorAll('[role="menuitem"]');
    expect(items[1]?.getAttribute('tabindex')).toBe('0');
  });

  it('PixelIconButton supports disabled', async () => {
    const fixture = await render(PixelIconButton, {
      icon: 'close',
      label: 'Close',
      disabled: true,
    });
    expect((fixture.nativeElement as HTMLElement).querySelector('button')?.disabled).toBe(true);
  });

  it('PixelCheckbox links its error', async () => {
    const fixture = await render(PixelCheckbox, { label: 'Done', error: 'Required' });
    const input = (fixture.nativeElement as HTMLElement).querySelector('input');
    expect(input?.getAttribute('aria-invalid')).toBe('true');
    expect(input?.getAttribute('aria-describedby')).toBeTruthy();
  });

  it.each([
    [PixelPanel, { title: 'Panel' }],
    [PixelTable, { caption: 'Table', columns: [{ key: 'k', label: 'K' }], rows: [{ k: 'v' }] }],
    [PixelTextarea, { label: 'Notes' }],
    [Card, { title: 'Card' }],
    [EmptyState, { title: 'Empty', message: 'Nothing' }],
    [Spinner, { label: 'Loading' }],
    [PageHeader, { title: 'Title' }],
    [Drawer, { label: 'Drawer', open: false }],
    [Dialog, { label: 'Dialog', title: 'Dialog', open: false }],
    [Toast, { message: 'Saved' }],
    [Breadcrumbs, { items: [{ label: 'Home', path: '/' }] }],
    [Pagination, { label: '1 of 2' }],
    [Skeleton, {}],
    [Progress, { value: 50 }],
    [Avatar, { name: 'Ada Lovelace' }],
    [Icon, { name: 'dashboard' }],
    [Divider, {}],
    [Kbd, {}],
  ])('renders %p', async (component, inputs) => {
    const fixture = await render(component as Type<unknown>, inputs);
    expect(fixture.componentInstance).toBeTruthy();
  });
});
