use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

/// 确定性高品质伪随机数生成器
/// 内部基于 Xoshiro256++ 算法，状态长度 256 位，周期长达 2^256 - 1
#[derive(Clone, Debug)]
pub struct DeterministicRng {
    rng: Xoshiro256PlusPlus,
    seed: u64,
    step_count: u64,
}

impl DeterministicRng {
    /// 从单个 64 位无符号整型种子创建
    pub fn new(seed: u64) -> Self {
        // 使用 SplitMix64 算法将单个 u64 种子膨胀初始化为 256 位状态
        let rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        Self {
            rng,
            seed,
            step_count: 0,
        }
    }

    /// 获取原始种子
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// 获取当前调用步数
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// 生成下一个 64 位随机数
    pub fn next_u64(&mut self) -> u64 {
        self.step_count += 1;
        self.rng.next_u64()
    }

    /// 生成下一个 32 位随机数
    pub fn next_u32(&mut self) -> u32 {
        self.step_count += 1;
        self.rng.next_u32()
    }

    /// 生成 [0.0, 1.0) 之间的 64 位双精度浮点数
    pub fn next_f64(&mut self) -> f64 {
        // 取 53 位精度生成标准双精度浮点
        let v = self.next_u64() >> 11;
        (v as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    /// 在整数半开区间 [min, max) 内生成均匀分布随机数
    pub fn gen_range_u32(&mut self, min: u32, max: u32) -> u32 {
        assert!(min < max, "min must be less than max");
        let range = max - min;
        min + (self.next_u64() % (range as u64)) as u32
    }

    /// 在闭区间 [min, max] 内生成整数
    pub fn gen_range_inclusive_i32(&mut self, min: i32, max: i32) -> i32 {
        assert!(min <= max, "min must be <= max");
        let range = (max - min + 1) as u64;
        min + (self.next_u64() % range) as i32
    }

    /// 根据概率 prob (0.0..=1.0) 进行伯努利试验
    pub fn gen_bool(&mut self, prob: f64) -> bool {
        if prob <= 0.0 {
            return false;
        }
        if prob >= 1.0 {
            return true;
        }
        self.next_f64() < prob
    }

    /// 加权轮盘赌抽样：根据传入的权重数组采样对应索引
    pub fn choose_weighted(&mut self, weights: &[f64]) -> usize {
        assert!(!weights.is_empty(), "weights slice must not be empty");
        let total_weight: f64 = weights.iter().sum();
        assert!(total_weight > 0.0, "total weight must be greater than zero");

        let mut roll = self.next_f64() * total_weight;
        for (i, &w) in weights.iter().enumerate() {
            if roll <= w {
                return i;
            }
            roll -= w;
        }
        weights.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism_same_seed() {
        let mut rng1 = DeterministicRng::new(123456789);
        let mut rng2 = DeterministicRng::new(123456789);

        for _ in 0..1000 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
            assert_eq!(rng1.next_f64(), rng2.next_f64());
        }
        assert_eq!(rng1.step_count(), rng2.step_count());
    }

    #[test]
    fn test_determinism_different_seed() {
        let mut rng1 = DeterministicRng::new(123456789);
        let mut rng2 = DeterministicRng::new(987654321);

        let mut identical = true;
        for _ in 0..10 {
            if rng1.next_u64() != rng2.next_u64() {
                identical = false;
                break;
            }
        }
        assert!(!identical);
    }

    #[test]
    fn test_weighted_choice() {
        let mut rng = DeterministicRng::new(42);
        let weights = [1.0, 2.0, 7.0]; // 10%, 20%, 70%
        let mut counts = [0; 3];

        for _ in 0..10000 {
            let choice = rng.choose_weighted(&weights);
            counts[choice] += 1;
        }

        // 验证在大数定律下比例接近
        let p0 = counts[0] as f64 / 10000.0;
        let p1 = counts[1] as f64 / 10000.0;
        let p2 = counts[2] as f64 / 10000.0;

        assert!((p0 - 0.10).abs() < 0.02, "p0 expected ~0.10, got {p0}");
        assert!((p1 - 0.20).abs() < 0.02, "p1 expected ~0.20, got {p1}");
        assert!((p2 - 0.70).abs() < 0.02, "p2 expected ~0.70, got {p2}");
    }
}
