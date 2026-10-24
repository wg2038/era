/* tslint:disable */
/* eslint-disable */
/**
 * 个人生命实体
 */
export interface Person {
    id: number;
    name: string;
    gender: Gender;
    birth_year: number;
    birth_location_id: string;
    current_year: number;
    age: number;
    is_alive: boolean;
    death_year: number | undefined;
    death_age: number | undefined;
    cause_of_death: CauseOfDeath | undefined;
    attributes: Attributes;
    wealth_ppp: number;
    personality: PersonalityTrait;
    family_tree: KinshipMember[];
    friends: Friend[];
    pending_delayed: PendingDelayedConsequence[];
    triggered_events: string[];
    milestone_quotes: MilestoneQuote[];
    actions_remaining_this_year: number;
    log: LifeLogEntry[];
}

/**
 * 亲友微互动类型
 */
export type SocialActionType = "Remit" | "Letter" | "Visit" | "Sever" | "Support";

/**
 * 亲属亲缘关系类别
 */
export type KinshipRelation = "Father" | "Mother" | "ElderBrother" | "YoungerBrother" | "ElderSister" | "YoungerSister" | "Spouse" | "Son" | "Daughter";

/**
 * 人生年鉴记录
 */
export interface LifeLogEntry {
    year: number;
    age: number;
    title: string;
    content: string;
    category: string;
}

/**
 * 人生节点反思存留记录（5、15、30、50、70 岁）
 */
export interface MilestoneQuote {
    age: number;
    year: number;
    question: string;
    quote: string;
}

/**
 * 历史死因枚举（严格对齐近代中国病理与社会历史）
 */
export type CauseOfDeath = { InfantMortality: string } | { ChildMortality: string } | { Famine: string } | { WartimeCasualty: string } | { Epidemic: string } | { Accident: string } | { OrganDisease: string } | "OldAge" | "SurviveTo2025";

/**
 * 历史货币展示封装
 */
export interface CurrencyDisplay {
    /**
     * 货币体系名称
     */
    currency_name: string;
    /**
     * 格式化后的金额展示（如 \"120 块银元\"、\"50 元人民币 / 20 斤粮票\"）
     */
    formatted_amount: string;
    /**
     * 时代金融背景说明
     */
    era_description: string;
}

/**
 * 完整模拟状态快照
 */
export interface SimulationState {
    seed: number;
    current_year: number;
    active_index: number;
    protagonist: Person;
    peers: Person[];
    is_game_over: boolean;
    can_switch_peer: boolean;
    pending_choice: ActiveEventPrompt | undefined;
    active_milestone: MilestonePrompt | undefined;
    current_focus: LifeFocus | undefined;
    birth_historical_name: string;
    birth_current_name: string;
    current_currency: CurrencyDisplay;
}

/**
 * 家族谱系实体
 */
export interface KinshipMember {
    id: number;
    name: string;
    relation: KinshipRelation;
    birth_year: number;
    age: number;
    is_alive: boolean;
    death_year: number | undefined;
    intimacy: number;
    occupation: string;
}

/**
 * 年度生命精力重心（回合制每年必选其一）
 */
export type LifeFocus = "Study" | "Livelihood" | "Family" | "Health" | "Social";

/**
 * 当前待玩家作答的人生节点自省提示
 */
export interface MilestonePrompt {
    age: number;
    year: number;
    title: string;
    narrative: string;
    question: string;
    default_quote: string;
}

/**
 * 当前待玩家做出决断的事件视图
 */
export interface ActiveEventPrompt {
    event_id: string;
    tier: string;
    title: string;
    narrative: string;
    is_crossroads: boolean;
    choices: ActiveChoiceOption[];
}

/**
 * 待生效的延迟后果队列项（3-10年显现）
 */
export interface PendingDelayedConsequence {
    source_event_id: string;
    target_year: number;
    dh: number;
    dk: number;
    dm: number;
    di: number;
    d_wealth: number;
    message: string;
}

/**
 * 性别
 */
export type Gender = "Male" | "Female";

/**
 * 性格倾向（用于快进模式下的次要抉择自动判定）
 */
export type PersonalityTrait = "Scholar" | "Enterprising" | "Pragmatic" | "Harmonious" | "Idealist";

/**
 * 抉择分支视图
 */
export interface ActiveChoiceOption {
    id: string;
    text: string;
    dh: number;
    dk: number;
    dm: number;
    di: number;
    d_wealth: number;
    risk: number | undefined;
}

/**
 * 朋辈社交实体（朋友系统：不维系会随岁月自然淡化）
 */
export interface Friend {
    id: number;
    name: string;
    acquaintance_year: number;
    age: number;
    is_alive: boolean;
    intimacy: number;
    tag: string;
}

/**
 * 蒙特卡洛单个年代统计报告
 */
export interface MonteCarloDecadeReport {
    decade: number;
    sample_count: number;
    avg_lifespan: number;
    benchmark_expected_lifespan: number;
    lifespan_deviation_pct: number;
    infant_mortality_rate: number;
    benchmark_imr: number;
    imr_deviation_pct: number;
    under_5_mortality_rate: number;
    reached_60_pct: number;
    reached_2025_pct: number;
}

/**
 * 角色基础属性包（四维属性，取值范围 0..=100）
 */
export interface Attributes {
    health: number;
    knowledge: number;
    mood: number;
    insight: number;
}

export interface LocationData {
    id: string;
    currentName: string;
    region: string;
    populationWeight: Map<string, number>;
    historicalNames: HistoricalNameEntry[];
    climate: string;
    source: string;
    confidence: string;
}


/**
 * WASM 暴露的核心模拟实例
 */
export class WasmSimulation {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 回合制推进一年（指定精力重心：读书/谋生/家人/身体/人情）
     */
    advance_year(focus_str: string): any;
    /**
     * 获取完整快照状态
     */
    get_state(): any;
    /**
     * 拜会好友
     */
    interact_friend(friend_id: number): any;
    /**
     * 朋友多类型微互动（寄钱、写信、探望、断联、相助）
     */
    interact_friend_action(friend_id: number, action_str: string): any;
    /**
     * 互动侍奉亲属
     */
    interact_relative(relative_id: number): any;
    /**
     * 亲属多类型微互动（寄钱、写信、探望、断联、赡养）
     */
    interact_relative_action(relative_id: number, action_str: string): any;
    /**
     * 做出抉择
     */
    make_choice(choice_id: string): any;
    /**
     * 创建全新模拟器
     */
    constructor(seed: bigint, birth_year?: number | null);
    /**
     * 连续模拟直至人生终局
     */
    run_to_end(): any;
    /**
     * 推进一年并返回当前最新状态
     */
    step_year(): any;
    /**
     * 快进模式自适应步进（次要抉择自动判定，仅在重大关口暂停）
     */
    step_year_auto(): any;
    /**
     * 封存人生节点心声
     */
    submit_milestone_quote(quote: string): any;
    /**
     * 早夭时切换至同年同县出生的另外四人之一
     */
    switch_to_peer(peer_index: number): boolean;
}

export function init_panic_hook(): void;

export function kernel_version(): string;

/**
 * 在浏览器/Worker 中执行蒙特卡洛统计检验
 */
export function run_monte_carlo(decade: number, count: number): any;

/**
 * 批量执行 1900–1990 全年代蒙特卡洛测试
 */
export function run_monte_carlo_all_decades(count_per_decade: number): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmsimulation_free: (a: number, b: number) => void;
    readonly init_panic_hook: () => void;
    readonly kernel_version: () => [number, number];
    readonly run_monte_carlo: (a: number, b: number) => [number, number, number];
    readonly run_monte_carlo_all_decades: (a: number) => [number, number, number];
    readonly wasmsimulation_advance_year: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmsimulation_get_state: (a: number) => [number, number, number];
    readonly wasmsimulation_interact_friend: (a: number, b: number) => [number, number, number];
    readonly wasmsimulation_interact_friend_action: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmsimulation_interact_relative: (a: number, b: number) => [number, number, number];
    readonly wasmsimulation_interact_relative_action: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasmsimulation_make_choice: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmsimulation_new: (a: bigint, b: number) => number;
    readonly wasmsimulation_run_to_end: (a: number) => [number, number, number];
    readonly wasmsimulation_step_year: (a: number) => [number, number, number];
    readonly wasmsimulation_step_year_auto: (a: number) => [number, number, number];
    readonly wasmsimulation_submit_milestone_quote: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmsimulation_switch_to_peer: (a: number, b: number) => number;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
