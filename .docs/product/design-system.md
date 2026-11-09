# AI-Lab Design System

| Field | Value |
|---|---|
| Document | `.docs/product/design-system.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/frontend.md`, `web/src/styles/_tokens.scss`, `web/src/app/shared/ui/` |

The design system is a pixel-art, accessible set of tokens and primitives. It is the only source of visual values for every product surface.

## 1. Tokens

`web/src/styles/_tokens.scss` defines CSS custom properties:

- Colour ramp: background, surface, raised, sunken, text, muted, accent, success, warning, danger, info, overlay.
- Borders: 2 px solid muted/accent; radii 0/2/4.
- Spacing: 4 px scale (`--ailab-space-1..8`).
- Shadows: hard pixel shadows (`4px 4px 0`).
- Typography: monospace stack, 12/14/16/24 px sizes, 1.4 line height.
- Motion: 80/160/240 ms; the global `prefers-reduced-motion` rule disables animation.
- Z-index: base/drawer/toast.

## 2. Primitives

| Component | Selector | States |
|---|---|---|
| Status badge | `app-status-badge` | neutral/success/warning/danger |
| Button | `app-button` | ghost/primary/danger, hover/focus/disabled |
| Card | `app-card` | titled/untitled |
| Empty state | `app-empty-state` | title/message/actions |
| Spinner | `app-spinner` | reduced-motion aware |
| Page header | `app-page-header` | title/subtitle/actions |
| Drawer | `app-drawer` | open/closed, focus trap, `aria-modal` |
| Input | `app-input` | text/search/email, label |
| Select | `app-select` | options, label |
| Tabs | `app-tabs` | `role="tablist"`, active tab |
| Dialog | `app-dialog` | modal, focus trap, confirm/cancel |
| Toast | `app-toast` | neutral/success/warning/danger |
| Breadcrumbs | `app-breadcrumbs` | linked trail, `aria-current` |
| Pagination | `app-pagination` | previous/next disabled states |
| Skeleton | `app-skeleton` | reduced-motion aware |
| Progress | `app-progress` | `role="progressbar"` |
| Avatar | `app-avatar` | initials |
| Icon | `app-icon` | 16x16 pixel SVG set |
| Menu | `app-menu` | `role="menu"` |
| Divider | `app-divider` | — |
| Kbd | `app-kbd` | key hint |
| Tooltip | `[appTooltip]` | native title fallback |
| Pixel panel | `app-pixel-panel` | titled/raised |
| Pixel table | `app-pixel-table` | caption, columns, rows |
| Pixel icon button | `app-pixel-icon-button` | icon + aria-label |
| Pixel textarea | `app-pixel-textarea` | error state, `aria-invalid` |
| Pixel checkbox | `app-pixel-checkbox` | checked model |

## 3. Accessibility rules

- Every interactive element has a visible focus ring.
- No colour-only meaning; icons and text accompany colour.
- The drawer traps Tab focus and restores it on close.
- The shell exposes a skip link, a `main` landmark and an `aria-live` toast host.
- Errors use `role="alert"` and include a trace id; form errors are linked with `aria-describedby`.
- Tabs implement roving tabindex plus Arrow/Home/End; menus implement Arrow/Home/End/Escape.
- Interactive primitives expose disabled/loading/error states where applicable.

## 4. Usage rules

- Feature components consume primitives; they do not restyle tokens directly.
- Tokens are additive; removing a token requires a visual review.
- New primitives need default/hover/focus/disabled/loading/error coverage and a unit test.
