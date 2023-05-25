use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PerformanceMetric {
    pub phase: String,
    pub elapsed: f64,
}

pub type PerformanceMetrics = Vec<PerformanceMetric>;

#[macro_export]
// Time the given expression and print/save the result.
macro_rules! time_expr {
    ($description:expr, $key:expr, $expression:expr, $build_config:expr, $metrics:expr) => {{
        if let Some(cfg) = $build_config {
            if cfg.time_phases || cfg.metrics_outfile.is_some() {
                let expr_start = std::time::Instant::now();
                let output = { $expression };
                let elapsed = expr_start.elapsed();
                if cfg.time_phases {
                    println!("  Time elapsed to {}: {:?}", $description, elapsed);
                }
                if cfg.metrics_outfile.is_some() {
                    $metrics.push(PerformanceMetric {
                        phase: $key.to_string(),
                        elapsed: elapsed.as_secs_f64(),
                    });
                }
                output
            } else {
                $expression
            }
        } else {
            $expression
        }
    }};
}
