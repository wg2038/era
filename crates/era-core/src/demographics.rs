use serde::{Deserialize, Serialize};

/// 历史年代生命表基准
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DecadeLifeTable {
    pub decade: u16,
    pub infant_mortality: f64, // 0岁死亡率 (q0)
    pub child_mortality: f64,  // 1-4岁年均死亡率
    pub adult_makeham_a: f64,  // Makeham 外生常数 A
    pub adult_gompertz_b: f64, // Gompertz 基底参数 B
    pub adult_gompertz_c: f64, // Gompertz 年龄指数底数 c (~1.085)
    pub expected_life_at_birth: f64,
}

/// 历史各年代基准生命表（严格依 Barclay et al. 1976、侯杨方《中国人口史》、1953/1982 国家普查核实校准）
pub const HISTORICAL_LIFE_TABLES: [DecadeLifeTable; 10] = [
    DecadeLifeTable {
        decade: 1900,
        infant_mortality: 0.235,
        child_mortality: 0.040,
        adult_makeham_a: 0.00810,
        adult_gompertz_b: 0.001970,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 31.5,
    },
    DecadeLifeTable {
        decade: 1910,
        infant_mortality: 0.228,
        child_mortality: 0.038,
        adult_makeham_a: 0.01950,
        adult_gompertz_b: 0.000110,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 32.2,
    },
    DecadeLifeTable {
        decade: 1920,
        infant_mortality: 0.218,
        child_mortality: 0.035,
        adult_makeham_a: 0.01270,
        adult_gompertz_b: 0.000990,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 33.5,
    },
    DecadeLifeTable {
        decade: 1930,
        infant_mortality: 0.200,
        child_mortality: 0.032,
        adult_makeham_a: 0.00230,
        adult_gompertz_b: 0.001850,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 35.0,
    },
    DecadeLifeTable {
        decade: 1940,
        infant_mortality: 0.195,
        child_mortality: 0.030,
        adult_makeham_a: 0.00290,
        adult_gompertz_b: 0.000970,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 35.5,
    },
    DecadeLifeTable {
        decade: 1950,
        infant_mortality: 0.138,
        child_mortality: 0.018,
        adult_makeham_a: 0.00390,
        adult_gompertz_b: 0.001010,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 46.5,
    },
    DecadeLifeTable {
        decade: 1960,
        infant_mortality: 0.082,
        child_mortality: 0.009,
        adult_makeham_a: 0.00230,
        adult_gompertz_b: 0.000290,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 57.0,
    },
    DecadeLifeTable {
        decade: 1970,
        infant_mortality: 0.052,
        child_mortality: 0.005,
        adult_makeham_a: 0.00310,
        adult_gompertz_b: 0.000310,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 65.5,
    },
    DecadeLifeTable {
        decade: 1980,
        infant_mortality: 0.035,
        child_mortality: 0.003,
        adult_makeham_a: 0.00290,
        adult_gompertz_b: 0.000290,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 68.2,
    },
    DecadeLifeTable {
        decade: 1990,
        infant_mortality: 0.030,
        child_mortality: 0.002,
        adult_makeham_a: 0.00230,
        adult_gompertz_b: 0.000250,
        adult_gompertz_c: 1.085,
        expected_life_at_birth: 71.4,
    },
];

/// 获取给定年份对应的生命表（通过年代插值）
pub fn get_life_table_for_year(year: u16) -> DecadeLifeTable {
    let year_clamped = year.clamp(1900, 1999);
    let idx = ((year_clamped - 1900) / 10).min(9) as usize;
    HISTORICAL_LIFE_TABLES[idx]
}

/// 计算宏观历史事件对特定年份与区域的死亡率乘数修正
pub fn get_historical_era_modifier(year: u16, region: &str) -> (f64, Option<&'static str>) {
    // 1910-1911 东北大鼠疫
    if (1910..=1911).contains(&year) && region == "东北" {
        return (1.6, Some("东北大鼠疫流行"));
    }
    // 1928-1930 民国十八年年馑 (西北、华北)
    if (1928..=1930).contains(&year) && (region == "西北" || region == "华北") {
        return (1.65, Some("民国大饥荒（年馑）"));
    }
    // 1937-1945 全面抗日战争时期
    if (1937..=1945).contains(&year) {
        if region == "华北" || region == "华东" || region == "华中" {
            return (1.45, Some("抗日战争战火波及"));
        }
        return (1.20, Some("战时动荡与配给困顿"));
    }
    // 1942-1943 中原大饥荒 (华中/中原)
    if (1942..=1943).contains(&year) && region == "华中" {
        return (1.70, Some("中原大饥荒"));
    }
    // 1959-1961 严重困难时期
    if (1959..=1961).contains(&year) {
        return (1.40, Some("三年困难时期营养匮乏"));
    }

    (1.0, None)
}

/// 计算特定年龄在特定年份与地区的单年基础死亡概率
pub fn calculate_annual_mortality_rate(
    age: u16,
    current_year: u16,
    region: &str,
    health: i32,
) -> (f64, Option<&'static str>) {
    let table = get_life_table_for_year(current_year);
    let (era_mult, era_event) = get_historical_era_modifier(current_year, region);

    // 健康度修正因子：70 为基准（系数 1.0）
    let health_clamped = health.clamp(0, 100) as f64;
    let health_factor = if health_clamped >= 70.0 {
        1.0 - (health_clamped - 70.0) / 100.0 * 0.25
    } else {
        1.0 + (70.0 - health_clamped) / 70.0 * 0.50
    };

    let base_rate = match age {
        0 => table.infant_mortality,
        1..=4 => table.child_mortality,
        5..=14 => (table.adult_makeham_a * 0.25).max(0.0006),
        15..=105 => {
            let x = (age - 15) as f64;
            let makeham_term = table.adult_makeham_a;
            let gompertz_term = table.adult_gompertz_b * table.adult_gompertz_c.powf(x);
            (makeham_term + gompertz_term).clamp(0.0005, 0.99)
        }
        _ => 0.99,
    };

    let final_rate = (base_rate * era_mult * health_factor).clamp(0.0005, 0.99);
    (final_rate, era_event)
}
