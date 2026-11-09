import { Routes } from '@angular/router';
import { Placeholder } from './features/placeholder/placeholder';
import { NotFound } from './features/not-found/not-found';

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
    loadComponent: () => Promise.resolve(Placeholder),
    data: { title: 'Office', body: 'Virtual office host (WP-18).' },
  },
  {
    path: 'projects/:projectId/board',
    title: 'AI-Lab · Board',
    loadComponent: () => Promise.resolve(Placeholder),
    data: { title: 'Kanban board', body: 'Board surface (WP-19).' },
  },
  {
    path: 'projects/:projectId/sow',
    title: 'AI-Lab · SOW',
    loadComponent: () => Promise.resolve(Placeholder),
    data: { title: 'SOW & delivery graph', body: 'SOW surface (WP-19).' },
  },
  {
    path: 'inventory/agents',
    title: 'AI-Lab · Agents',
    loadComponent: () => Promise.resolve(Placeholder),
    data: { title: 'Agents', body: 'Agent inventory (WP-19).' },
  },
  {
    path: 'inventory/projects',
    title: 'AI-Lab · Projects',
    loadComponent: () => Promise.resolve(Placeholder),
    data: { title: 'Projects', body: 'Project inventory (WP-19).' },
  },
  {
    path: 'inventory/providers',
    title: 'AI-Lab · Providers',
    loadComponent: () => Promise.resolve(Placeholder),
    data: { title: 'Providers', body: 'Provider inventory (WP-19).' },
  },
  {
    path: 'memory',
    title: 'AI-Lab · Memory',
    loadComponent: () => Promise.resolve(Placeholder),
    data: { title: 'Memory', body: 'Memory browser (WP-19).' },
  },
  {
    path: 'settings',
    title: 'AI-Lab · Settings',
    loadComponent: () => Promise.resolve(Placeholder),
    data: { title: 'Settings', body: 'Lab and autonomy settings (WP-19).' },
  },
  {
    path: '**',
    title: 'AI-Lab · Not found',
    loadComponent: () => Promise.resolve(NotFound),
  },
];
