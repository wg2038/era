pub mod currency;
pub mod data;
pub mod demographics;
pub mod destiny;
pub mod engine;
pub mod event_engine;
pub mod models;
pub mod prng;
pub mod social;

use wasm_bindgen::prelude::*;

use crate::engine::{MonteCarloDecadeReport, SimulationKernel};
use crate::models::{LifeFocus, SocialActionType};

#[wasm_bindgen]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn kernel_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// WASM 暴露的核心模拟实例
#[wasm_bindgen]
pub struct WasmSimulation {
    inner: SimulationKernel,
}

#[wasm_bindgen]
impl WasmSimulation {
    /// 创建全新模拟器
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64, birth_year: Option<u16>) -> Self {
        init_panic_hook();
        let inner = SimulationKernel::new(seed, birth_year);
        Self { inner }
    }

    /// 回合制推进一年（指定精力重心：读书/谋生/家人/身体/人情）
    pub fn advance_year(&mut self, focus_str: &str) -> Result<JsValue, JsValue> {
        let focus = LifeFocus::from_str_id(focus_str)
            .ok_or_else(|| JsValue::from_str(&format!("Invalid focus: {focus_str}")))?;
        self.inner
            .advance_year(focus)
            .map_err(|e| JsValue::from_str(&e))
            .and_then(|state| {
                serde_wasm_bindgen::to_value(&state)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
            })
    }

    /// 封存人生节点心声
    pub fn submit_milestone_quote(&mut self, quote: &str) -> Result<JsValue, JsValue> {
        self.inner
            .submit_milestone_quote(quote)
            .map_err(|e| JsValue::from_str(&e))
            .and_then(|state| {
                serde_wasm_bindgen::to_value(&state)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
            })
    }

    /// 亲属多类型微互动（寄钱、写信、探望、断联、赡养）
    pub fn interact_relative_action(
        &mut self,
        relative_id: u32,
        action_str: &str,
    ) -> Result<JsValue, JsValue> {
        let action = SocialActionType::from_str_id(action_str)
            .ok_or_else(|| JsValue::from_str(&format!("Invalid action: {action_str}")))?;
        self.inner
            .interact_social_relative(relative_id, action)
            .map_err(|e| JsValue::from_str(&e))
            .and_then(|state| {
                serde_wasm_bindgen::to_value(&state)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
            })
    }

    /// 朋友多类型微互动（寄钱、写信、探望、断联、相助）
    pub fn interact_friend_action(
        &mut self,
        friend_id: u32,
        action_str: &str,
    ) -> Result<JsValue, JsValue> {
        let action = SocialActionType::from_str_id(action_str)
            .ok_or_else(|| JsValue::from_str(&format!("Invalid action: {action_str}")))?;
        self.inner
            .interact_social_friend(friend_id, action)
            .map_err(|e| JsValue::from_str(&e))
            .and_then(|state| {
                serde_wasm_bindgen::to_value(&state)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
            })
    }

    /// 推进一年并返回当前最新状态
    pub fn step_year(&mut self) -> Result<JsValue, JsValue> {
        let state = self.inner.step_year();
        serde_wasm_bindgen::to_value(&state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    /// 快进模式自适应步进（次要抉择自动判定，仅在重大关口暂停）
    pub fn step_year_auto(&mut self) -> Result<JsValue, JsValue> {
        let state = self.inner.step_year_auto();
        serde_wasm_bindgen::to_value(&state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    /// 做出抉择
    pub fn make_choice(&mut self, choice_id: &str) -> Result<JsValue, JsValue> {
        self.inner
            .make_choice(choice_id)
            .map_err(|e| JsValue::from_str(&e))
            .and_then(|state| {
                serde_wasm_bindgen::to_value(&state)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
            })
    }

    /// 互动侍奉亲属
    pub fn interact_relative(&mut self, relative_id: u32) -> Result<JsValue, JsValue> {
        self.inner
            .interact_relative(relative_id)
            .map_err(|e| JsValue::from_str(&e))
            .and_then(|state| {
                serde_wasm_bindgen::to_value(&state)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
            })
    }

    /// 拜会好友
    pub fn interact_friend(&mut self, friend_id: u32) -> Result<JsValue, JsValue> {
        self.inner
            .interact_friend(friend_id)
            .map_err(|e| JsValue::from_str(&e))
            .and_then(|state| {
                serde_wasm_bindgen::to_value(&state)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
            })
    }

    /// 获取完整快照状态
    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        let state = self.inner.get_state();
        serde_wasm_bindgen::to_value(&state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    /// 早夭时切换至同年同县出生的另外四人之一
    pub fn switch_to_peer(&mut self, peer_index: usize) -> bool {
        self.inner.switch_to_peer(peer_index)
    }

    /// 连续模拟直至人生终局
    pub fn run_to_end(&mut self) -> Result<JsValue, JsValue> {
        let state = self.inner.run_to_end();
        serde_wasm_bindgen::to_value(&state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }
}

/// 在浏览器/Worker 中执行蒙特卡洛统计检验
#[wasm_bindgen]
pub fn run_monte_carlo(decade: u16, count: usize) -> Result<JsValue, JsValue> {
    let report = SimulationKernel::run_monte_carlo(decade, count);
    serde_wasm_bindgen::to_value(&report)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
}

/// 批量执行 1900–1990 全年代蒙特卡洛测试
#[wasm_bindgen]
pub fn run_monte_carlo_all_decades(count_per_decade: usize) -> Result<JsValue, JsValue> {
    let decades = [1900, 1910, 1920, 1930, 1940, 1950, 1960, 1970, 1980, 1990];
    let reports: Vec<MonteCarloDecadeReport> = decades
        .iter()
        .map(|&d| SimulationKernel::run_monte_carlo(d, count_per_decade))
        .collect();

    serde_wasm_bindgen::to_value(&reports)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_version() {
        assert_eq!(kernel_version(), "0.1.0");
    }

    #[test]
    fn test_full_simulation_run() {
        let mut sim = SimulationKernel::new(42, Some(1920));
        let final_state = sim.run_to_end();
        assert!(final_state.is_game_over);
        assert!(final_state.protagonist.log.len() > 1);
    }
}
