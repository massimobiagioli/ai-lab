# AI-Lab Frontend

| Field | Value |
|---|---|
| Document | `.docs/product/frontend.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/architecture.md`, `.docs/FRONTEND-INSTRUCTIONS.md`, `web/` |

The frontend is Angular 22 zoneless with signals, strict TypeScript 6.0 (ADR-0024), standalone components and the pixel design system.

## 1. Design tokens

`web/src/styles/_tokens.scss` defines the pixel palette, spacing, borders, typography and status colours as CSS custom properties. Tokens are the only source of visual values; components do not hardcode colours or spacing.

## 2. Components

- `StatusBadge` (`web/src/app/shared/status-badge/`): signal inputs `label` and `tone`, `role="status"`, uppercase pixel styling, no color-only meaning.
- The shell is `App` with the router outlet; feature surfaces consume signal stores (WP-19).

## 3. Accessibility

- WCAG 2.2 AA contrast.
- `role="status"` for live status; keyboard paths; focus-visible styles.
- Reduced-motion support for animated surfaces (WP-18).
- No color-only status: the badge text carries the state.

## 4. Budgets and quality

- Initial bundle budget 350 kB error, 300 kB warning; office chunk 180 kB; component styles 60 kB.
- `npm run lint`, `npm test`, `npm run build` are required.
- The component test covers rendering, the status role and the tone class.

## 5. Deferred to consumer WPs

- Full design system kit and route shell: WP-17 follow-up.
- Canvas office host: WP-18.
- Dashboard, inventory, Kanban, SOW, memory and settings pages: WP-19.
- Playwright a11y/visual suites: WP-21.

## 6. References

- `.docs/FRONTEND-INSTRUCTIONS.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §11–§12
- `web/src/styles/_tokens.scss`, `web/src/app/shared/status-badge/`
