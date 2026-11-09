import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';

const SLUG = /^[a-z0-9][a-z0-9-]{0,62}$/;

/** Validate a lab/project route parameter and redirect to the dashboard if invalid. */
export const paramGuard: CanActivateFn = (route) => {
  const router = inject(Router);
  const value = route.paramMap.get('labId') ?? route.paramMap.get('projectId') ?? '';
  return SLUG.test(value) ? true : router.parseUrl('/');
};
