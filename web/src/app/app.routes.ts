import { Routes } from '@angular/router';
import { paramGuard } from './core/guards/param.guard';

/** Lazy application routes. */
export const routes: Routes = [
  {
    path: '',
    title: 'AI-Lab · Dashboard',
    loadComponent: () =>
      import('./features/dashboard/dashboard').then((module) => module.Dashboard),
  },
  {
    path: 'runs',
    title: 'AI-Lab · Runs',
    loadComponent: () => import('./features/runs/runs').then((module) => module.Runs),
  },
  {
    path: 'lab/:labId',
    title: 'AI-Lab · Office',
    canActivate: [paramGuard],
    loadComponent: () => import('./features/office/office').then((module) => module.Office),
  },
  {
    path: 'projects/:projectId/board',
    title: 'AI-Lab · Board',
    canActivate: [paramGuard],
    loadComponent: () => import('./features/board/board').then((module) => module.Board),
  },
  {
    path: 'projects/:projectId/sow',
    title: 'AI-Lab · SOW',
    canActivate: [paramGuard],
    loadComponent: () => import('./features/sow/sow').then((module) => module.Sow),
  },
  {
    path: 'inventory/agents',
    title: 'AI-Lab · Agents',
    loadComponent: () =>
      import('./features/inventory/inventory').then((module) => module.Inventory),
    data: { section: 'agents' },
  },
  {
    path: 'inventory/projects',
    title: 'AI-Lab · Projects',
    loadComponent: () =>
      import('./features/inventory/inventory').then((module) => module.Inventory),
    data: { section: 'projects' },
  },
  {
    path: 'inventory/providers',
    title: 'AI-Lab · Providers',
    loadComponent: () =>
      import('./features/inventory/inventory').then((module) => module.Inventory),
    data: { section: 'providers' },
  },
  {
    path: 'memory',
    title: 'AI-Lab · Memory',
    loadComponent: () =>
      import('./features/placeholder/placeholder').then((module) => module.Placeholder),
    data: { title: 'Memory', body: 'Memory browser (WP-19).' },
  },
  {
    path: 'settings',
    title: 'AI-Lab · Settings',
    loadComponent: () =>
      import('./features/placeholder/placeholder').then((module) => module.Placeholder),
    data: { title: 'Settings', body: 'Lab and autonomy settings (WP-19).' },
  },
  {
    path: 'design',
    title: 'AI-Lab · Design',
    loadComponent: () => import('./features/design/design').then((module) => module.DesignGallery),
  },
  {
    path: '**',
    title: 'AI-Lab · Not found',
    loadComponent: () => import('./features/not-found/not-found').then((module) => module.NotFound),
  },
];
