use era_core::engine::SimulationKernel;

#[test]
fn test_monte_carlo_historical_demographics_100k() {
    println!("\n==========================================================================================");
    println!("          《Era》蒙特卡洛 100,000 次历史人口学对齐验证报告 (每年代 10,000 次)              ");
    println!("==========================================================================================");
    println!(" 出生年代 | 模拟均寿 | 史料基准均寿 | 均寿偏差 | 模拟婴儿死亡率 | 史料婴儿死亡率 | 活至60岁比例 | 见证2025比例 ");
    println!("----------|----------|--------------|----------|----------------|----------------|--------------|--------------");

    let decades = [1900, 1910, 1920, 1930, 1940, 1950, 1960, 1970, 1980, 1990];
    let mut prev_lifespan = 0.0;

    for &decade in &decades {
        let report = SimulationKernel::run_monte_carlo(decade, 10_000);

        println!(
            "  {}s   |  {:5.2}岁 |    {:5.2}岁   |  {:+.2}%  |     {:6.2}%    |     {:6.2}%    |    {:6.2}%   |    {:6.2}%   ",
            report.decade,
            report.avg_lifespan,
            report.benchmark_expected_lifespan,
            report.lifespan_deviation_pct,
            report.infant_mortality_rate * 100.0,
            report.benchmark_imr * 100.0,
            report.reached_60_pct,
            report.reached_2025_pct
        );

        // 断言统计偏差在科学容差内（预期寿命与历史基准偏差绝对值 < 6.0%）
        assert!(
            report.lifespan_deviation_pct.abs() < 6.0,
            "Decade {} lifespan deviation {:+.2}% exceeds tolerance 6.0%",
            report.decade,
            report.lifespan_deviation_pct
        );

        // 验证 20 世纪下半叶预期寿命飞跃性增长
        if decade >= 1950 {
            assert!(
                report.avg_lifespan > prev_lifespan,
                "Expected lifespan to increase monotonically in second half of century: {} vs {}",
                report.avg_lifespan,
                prev_lifespan
            );
        }
        prev_lifespan = report.avg_lifespan;

        // 验证婴儿死亡率演变规律：清末民国 > 18%，90年代 < 5%
        if decade <= 1930 {
            assert!(
                report.infant_mortality_rate >= 0.18,
                "Infant mortality in {} should be >= 18%, got {:.2}%",
                decade,
                report.infant_mortality_rate * 100.0
            );
        } else if decade >= 1990 {
            assert!(
                report.infant_mortality_rate <= 0.05,
                "Infant mortality in 1990 should be <= 5%, got {:.2}%",
                report.infant_mortality_rate * 100.0
            );
        }
    }

    println!("==========================================================================================\n");
}
