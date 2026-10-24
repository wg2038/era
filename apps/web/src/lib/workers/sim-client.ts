import type { MonteCarloDecadeReport, SimulationState } from '@era/era-core';
import type { WorkerMessage, WorkerResponse } from './sim.worker';

export class SimulationWorkerClient {
  private worker: Worker | null = null;
  private pendingResolvers: Array<(res: WorkerResponse) => void> = [];

  constructor() {
    if (typeof window !== 'undefined') {
      this.worker = new Worker(new URL('./sim.worker.ts', import.meta.url), {
        type: 'module',
      });

      this.worker.onmessage = (e: MessageEvent<WorkerResponse>) => {
        const resolver = this.pendingResolvers.shift();
        if (resolver) {
          resolver(e.data);
        }
      };
    }
  }

  private send(msg: WorkerMessage): Promise<WorkerResponse> {
    if (!this.worker) {
      return Promise.reject(new Error('Web Worker not available in this environment'));
    }
    return new Promise((resolve) => {
      this.pendingResolvers.push(resolve);
      this.worker?.postMessage(msg);
    });
  }

  async init(seed?: number | bigint, birthYear?: number): Promise<SimulationState> {
    const res = await this.send({ type: 'INIT', seed, birthYear });
    if (res.type === 'STATE') return res.state;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async step(): Promise<SimulationState> {
    const res = await this.send({ type: 'STEP' });
    if (res.type === 'STATE') return res.state;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async stepAuto(): Promise<SimulationState> {
    const res = await this.send({ type: 'STEP_AUTO' });
    if (res.type === 'STATE') return res.state;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async makeChoice(choiceId: string): Promise<SimulationState> {
    const res = await this.send({ type: 'MAKE_CHOICE', choiceId });
    if (res.type === 'STATE') return res.state;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async interactRelative(relativeId: number): Promise<SimulationState> {
    const res = await this.send({ type: 'INTERACT_RELATIVE', relativeId });
    if (res.type === 'STATE') return res.state;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async interactFriend(friendId: number): Promise<SimulationState> {
    const res = await this.send({ type: 'INTERACT_FRIEND', friendId });
    if (res.type === 'STATE') return res.state;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async runToEnd(): Promise<SimulationState> {
    const res = await this.send({ type: 'RUN_TO_END' });
    if (res.type === 'STATE') return res.state;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async switchPeer(peerIndex: number): Promise<SimulationState> {
    const res = await this.send({ type: 'SWITCH_PEER', peerIndex });
    if (res.type === 'STATE') return res.state;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async runMonteCarlo(decade: number, count = 10000): Promise<MonteCarloDecadeReport> {
    const res = await this.send({ type: 'RUN_MONTE_CARLO', decade, count });
    if (res.type === 'MONTE_CARLO') return res.report;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  async runMonteCarloAll(countPerDecade = 1000): Promise<MonteCarloDecadeReport[]> {
    const res = await this.send({ type: 'RUN_MONTE_CARLO_ALL', countPerDecade });
    if (res.type === 'MONTE_CARLO_ALL') return res.reports;
    throw new Error(res.type === 'ERROR' ? res.message : 'Unexpected response');
  }

  terminate() {
    this.worker?.terminate();
    this.worker = null;
  }
}
