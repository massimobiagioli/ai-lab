const TOKEN_KEY = 'ailab.token';

/** Session-scoped token storage. Never persisted to localStorage. */
export const sessionToken = {
  read(): string | null {
    try {
      return sessionStorage.getItem(TOKEN_KEY);
    } catch {
      return null;
    }
  },
  write(token: string | null): void {
    try {
      if (token === null) {
        sessionStorage.removeItem(TOKEN_KEY);
      } else {
        sessionStorage.setItem(TOKEN_KEY, token);
      }
    } catch {
      // Storage can be unavailable in hardened contexts; the runtime config wins.
    }
  },
};
