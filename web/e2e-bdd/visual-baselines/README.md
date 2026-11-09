# Visual baselines

Golden screenshots for sections 22 and 23 live here. They are generated from the deterministic seed and are never hand-edited. Playwright writes them through `toHaveScreenshot` with the `snapshotPathTemplate` in `../playwright.config.ts`.

- A baseline change requires an intentional reviewed change with the new screenshot in the diff.
- Never widen the 0.1 percent threshold, add an undocumented mask or edit a baseline to hide a regression.
- The matrix covers desktop, tablet and phone widths, DPR 1 and 2, the Amiga theme, the management routes and reduced motion on and off.
