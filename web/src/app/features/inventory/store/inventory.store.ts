import { inject, Injectable, signal } from '@angular/core';
import { ApiClient } from '../../../core/api/api-client';
import { toProblem } from '../../../core/api/problem';
import type { LoadState } from '../../../core/state/store-utils';
import type { Agent, Problem, Project, Provider } from '../../../domain/models';

/** Signal store for the inventory surfaces. */
@Injectable({ providedIn: 'root' })
export class InventoryStore {
  private readonly api = inject(ApiClient);
  readonly state = signal<LoadState>('idle');
  readonly agents = signal<readonly Agent[]>([]);
  readonly projects = signal<readonly Project[]>([]);
  readonly providers = signal<readonly Provider[]>([]);
  readonly error = signal<Problem | null>(null);
  readonly demo = this.api.usingDemo;

  addAgent(agent: Agent): void {
    this.agents.update((agents) => [...agents, agent]);
  }

  setAgentStatus(id: string, status: Agent['status']): void {
    this.agents.update((agents) =>
      agents.map((agent) => (agent.id === id ? { ...agent, status } : agent)),
    );
  }

  removeAgent(id: string): void {
    this.agents.update((agents) => agents.filter((agent) => agent.id !== id));
  }

  addProject(project: Project): void {
    this.projects.update((projects) => [...projects, project]);
  }

  addProvider(provider: Provider): void {
    this.providers.update((providers) => [...providers, provider]);
  }

  async load(): Promise<void> {
    this.state.set('loading');
    this.error.set(null);
    try {
      const [agents, projects, providers] = await Promise.all([
        this.api.listAgents(),
        this.api.listProjects(),
        this.api.listProviders(),
      ]);
      this.agents.set(agents);
      this.projects.set(projects);
      this.providers.set(providers);
      this.state.set('ready');
    } catch (error) {
      this.error.set(toProblem(error));
      this.state.set('error');
    }
  }
}
