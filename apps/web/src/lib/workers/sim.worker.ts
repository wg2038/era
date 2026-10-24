import initWasm, {
  type MonteCarloDecadeReport,
  run_monte_carlo,
  run_monte_carlo_all_decades,
  type SimulationState,
  WasmSimulation,
} from '@era/era-core';

let simulation: WasmSimulation | null = null;
let isInitialized = false;

async function ensureWasm() {
  if (!isInitialized) {
    await initWasm();
    isInitialized = true;
  }
}

export type WorkerMessage =
  | { type: 'INIT'; seed?: bigint | number; birthYear?: number }
  | { type: 'ADVANCE_YEAR'; focus: string }
  | { type: 'SUBMIT_MILESTONE'; quote: string }
  | { type: 'STEP' }
  | { type: 'STEP_AUTO' }
  | { type: 'MAKE_CHOICE'; choiceId: string }
  | { type: 'INTERACT_RELATIVE'; relativeId: number }
  | { type: 'INTERACT_FRIEND'; friendId: number }
  | { type: 'INTERACT_RELATIVE_ACTION'; relativeId: number; action: string }
  | { type: 'INTERACT_FRIEND_ACTION'; friendId: number; action: string }
  | { type: 'RUN_TO_END' }
  | { type: 'SWITCH_PEER'; peerIndex: number }
  | { type: 'RUN_MONTE_CARLO'; decade: number; count: number }
  | { type: 'RUN_MONTE_CARLO_ALL'; countPerDecade: number };

export type WorkerResponse =
  | { type: 'STATE'; state: SimulationState }
  | { type: 'MONTE_CARLO'; report: MonteCarloDecadeReport }
  | { type: 'MONTE_CARLO_ALL'; reports: MonteCarloDecadeReport[] }
  | { type: 'ERROR'; message: string };

self.onmessage = async (e: MessageEvent<WorkerMessage>) => {
  try {
    await ensureWasm();
    const msg = e.data;

    switch (msg.type) {
      case 'INIT': {
        const seed = BigInt(msg.seed ?? Date.now());
        simulation = new WasmSimulation(seed, msg.birthYear);
        const state = simulation.get_state() as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'ADVANCE_YEAR': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.advance_year(msg.focus) as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'SUBMIT_MILESTONE': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.submit_milestone_quote(msg.quote) as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'STEP': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.step_year() as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'STEP_AUTO': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.step_year_auto() as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'MAKE_CHOICE': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.make_choice(msg.choiceId) as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'INTERACT_RELATIVE': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.interact_relative(msg.relativeId) as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'INTERACT_FRIEND': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.interact_friend(msg.friendId) as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'INTERACT_RELATIVE_ACTION': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.interact_relative_action(
          msg.relativeId,
          msg.action,
        ) as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'INTERACT_FRIEND_ACTION': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.interact_friend_action(
          msg.friendId,
          msg.action,
        ) as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'RUN_TO_END': {
        if (!simulation) throw new Error('Simulation not initialized');
        const state = simulation.run_to_end() as SimulationState;
        self.postMessage({ type: 'STATE', state });
        break;
      }

      case 'SWITCH_PEER': {
        if (!simulation) throw new Error('Simulation not initialized');
        const success = simulation.switch_to_peer(msg.peerIndex);
        if (success) {
          const state = simulation.get_state() as SimulationState;
          self.postMessage({ type: 'STATE', state });
        } else {
          self.postMessage({ type: 'ERROR', message: 'Failed to switch to peer' });
        }
        break;
      }

      case 'RUN_MONTE_CARLO': {
        const report = run_monte_carlo(msg.decade, msg.count);
        self.postMessage({ type: 'MONTE_CARLO', report });
        break;
      }

      case 'RUN_MONTE_CARLO_ALL': {
        const reports = run_monte_carlo_all_decades(msg.countPerDecade);
        self.postMessage({ type: 'MONTE_CARLO_ALL', reports });
        break;
      }
    }
  } catch (err: unknown) {
    const errorMsg = err instanceof Error ? err.message : String(err);
    self.postMessage({ type: 'ERROR', message: errorMsg });
  }
};
