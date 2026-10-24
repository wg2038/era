use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

/// 历史货币展示封装
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CurrencyDisplay {
    /// 货币体系名称
    pub currency_name: String,
    /// 格式化后的金额展示（如 "120 块银元"、"50 元人民币 / 20 斤粮票"）
    pub formatted_amount: String,
    /// 时代金融背景说明
    pub era_description: String,
}

/// 依据年份与内部购买力点数（wealth_ppp）生成历史货币表皮
pub fn format_currency(year: u16, wealth_ppp: i32) -> CurrencyDisplay {
    let ppp = wealth_ppp.max(0);

    match year {
        // 1900–1935: 清末至民国银本位时期
        1900..=1934 => {
            let silver_dollars = ppp;
            CurrencyDisplay {
                currency_name: "银元 (大洋)".to_string(),
                formatted_amount: format!("{silver_dollars} 块银元"),
                era_description: "银本位时期，以袁大头、龙洋银元及铜元辅币为主，币值坚挺。"
                    .to_string(),
            }
        }

        // 1935–1947: 法币时期（1935法币改革，抗战期间通胀逐步显现）
        1935..=1947 => {
            // 随年份快速膨胀的法币名义面额
            let multiplier = match year {
                1935..=1937 => 1,
                1938..=1941 => 10,
                1942..=1944 => 200,
                1945 => 1_500,
                1946 => 15_000,
                _ => 120_000, // 1947
            };
            let nominal = (ppp as i64) * multiplier;
            CurrencyDisplay {
                currency_name: "法币".to_string(),
                formatted_amount: format!("{nominal} 元法币"),
                era_description: "1935年国民政府实施法币改革，抗战爆发后财政赤字导致法币持续贬值。"
                    .to_string(),
            }
        }

        // 1948–1948: 金圆券狂潮（世界罕见的恶性通货膨胀）
        1948 => {
            let nominal = (ppp as i64) * 800_000;
            CurrencyDisplay {
                currency_name: "金圆券".to_string(),
                formatted_amount: format!("{nominal} 元金圆券"),
                era_description: "1948年8月推行金圆券改革以回收法币，数月内信用彻底崩溃，物价飞涨。"
                    .to_string(),
            }
        }

        // 1949–1954: 第一套人民币（旧币）与折实单位
        1949..=1954 => {
            // 旧人民币 10,000 元 = 后来的新人民币 1 元
            let nominal = (ppp as i64) * 10_000;
            let units = (ppp / 2).max(1);
            CurrencyDisplay {
                currency_name: "旧人民币 / 折实单位".to_string(),
                formatted_amount: format!("{nominal} 旧人民币元 (合 {units} 份折实单位)"),
                era_description: "新中国成立初期发行第一套人民币，推行折实公债平抑旧时代恶性通胀。"
                    .to_string(),
            }
        }

        // 1955–1978: 第二/三套人民币 + 粮票配给制
        1955..=1978 => {
            // 工薪与物价高度稳定：月薪通常在 30-80 元，内部 ppp 50 对应约 40-50 元
            let rmb = (ppp as f64 * 0.8).round() as i32;
            let food_coupon_catties = (ppp as f64 * 0.5).round() as i32;
            CurrencyDisplay {
                currency_name: "第二/三套人民币 + 全国粮票".to_string(),
                formatted_amount: format!("{rmb} 元人民币 + {food_coupon_catties} 斤通用粮票"),
                era_description: "计划经济与票证配给时代，物价稳定，城镇职工凭票定量供应油粮布匹。"
                    .to_string(),
            }
        }

        // 1979–1992: 改革开放初期 / 第四套人民币 / 价格双轨制
        1979..=1992 => {
            let rmb = (ppp as f64 * 2.5).round() as i32;
            let food_coupon_catties = ((ppp as f64 * 0.3) * ((1993 - year) as f64 / 14.0)).round() as i32;
            let formatted = if food_coupon_catties > 0 {
                format!("{rmb} 元人民币 + {food_coupon_catties} 斤地方粮票 (双轨制)")
            } else {
                format!("{rmb} 元人民币 (市场化自选)")
            };
            CurrencyDisplay {
                currency_name: "第四套人民币 (过渡期)".to_string(),
                formatted_amount: formatted,
                era_description: "改革开放与物价双轨制时期，个体商业活跃，粮票等票证逐步退市。"
                    .to_string(),
            }
        }

        // 1993–2025: 全面市场经济与第五套人民币
        _ => {
            let multiplier = match year {
                1993..=1999 => 20,
                2000..=2010 => 80,
                2011..=2020 => 250,
                _ => 400, // 2021-2025
            };
            let rmb = (ppp as i64) * multiplier;
            CurrencyDisplay {
                currency_name: "人民币 (CNY)".to_string(),
                formatted_amount: format!("{rmb} 元人民币"),
                era_description: "粮票于1993年彻底退市，现代市场经济与移动支付时代全面开启。"
                    .to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_skinning_across_eras() {
        let c1912 = format_currency(1912, 100);
        assert_eq!(c1912.currency_name, "银元 (大洋)");
        assert_eq!(c1912.formatted_amount, "100 块银元");

        let c1948 = format_currency(1948, 100);
        assert_eq!(c1948.currency_name, "金圆券");
        assert!(c1948.formatted_amount.contains("金圆券"));

        let c1965 = format_currency(1965, 50);
        assert!(c1965.formatted_amount.contains("粮票"));

        let c2020 = format_currency(2020, 100);
        assert_eq!(c2020.currency_name, "人民币 (CNY)");
    }
}
