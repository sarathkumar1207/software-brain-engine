use crate::model::{RiskBreakdown, SimulationOperation};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiskConfig {
    pub caller_weight: f32,
    pub dependency_weight: f32,
    pub cross_module_weight: f32,
    pub depth_weight: f32,
    pub critical_flow_weight: f32,
    pub replace_operation_weight: f32,
    pub delete_operation_weight: f32,
    pub caller_saturation: usize,
    pub dependency_saturation: usize,
    pub cross_module_saturation: usize,
    pub flow_saturation: usize,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            caller_weight: 25.0,
            dependency_weight: 15.0,
            cross_module_weight: 20.0,
            depth_weight: 15.0,
            critical_flow_weight: 15.0,
            replace_operation_weight: 5.0,
            delete_operation_weight: 10.0,
            caller_saturation: 20,
            dependency_saturation: 20,
            cross_module_saturation: 10,
            flow_saturation: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiskInput {
    pub operation: SimulationOperation,
    pub caller_count: usize,
    pub dependency_count: usize,
    pub cross_module_edges: usize,
    pub reached_depth: usize,
    pub max_depth: usize,
    pub critical_flows: usize,
}

pub trait RiskScoring {
    fn score(&self, input: RiskInput) -> (f32, RiskBreakdown);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RiskScorer {
    config: RiskConfig,
}

impl RiskScorer {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }
}

impl RiskScoring for RiskScorer {
    fn score(&self, input: RiskInput) -> (f32, RiskBreakdown) {
        let caller_score = saturated(input.caller_count, self.config.caller_saturation)
            * self.config.caller_weight;
        let dependency_score = saturated(input.dependency_count, self.config.dependency_saturation)
            * self.config.dependency_weight;
        let cross_module_score = saturated(
            input.cross_module_edges,
            self.config.cross_module_saturation,
        ) * self.config.cross_module_weight;
        let depth_score = if input.max_depth == 0 {
            0.0
        } else {
            (input.reached_depth.min(input.max_depth) as f32 / input.max_depth as f32)
                * self.config.depth_weight
        };
        let critical_flow_score = saturated(input.critical_flows, self.config.flow_saturation)
            * self.config.critical_flow_weight;
        let operation_score = match input.operation {
            SimulationOperation::Modify => 0.0,
            SimulationOperation::Replace => self.config.replace_operation_weight,
            SimulationOperation::Delete => self.config.delete_operation_weight,
        };
        let breakdown = RiskBreakdown {
            caller_score: round(caller_score),
            dependency_score: round(dependency_score),
            cross_module_score: round(cross_module_score),
            depth_score: round(depth_score),
            critical_flow_score: round(critical_flow_score),
            operation_score: round(operation_score),
        };
        let total = caller_score
            + dependency_score
            + cross_module_score
            + depth_score
            + critical_flow_score
            + operation_score;
        (round(total.clamp(0.0, 100.0)), breakdown)
    }
}

fn saturated(value: usize, saturation: usize) -> f32 {
    if saturation == 0 {
        return 0.0;
    }
    (value as f32 / saturation as f32).min(1.0)
}

fn round(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_is_riskier_than_modify_for_same_graph_shape() {
        let scorer = RiskScorer::default();
        let base = RiskInput {
            operation: SimulationOperation::Modify,
            caller_count: 8,
            dependency_count: 4,
            cross_module_edges: 3,
            reached_depth: 3,
            max_depth: 6,
            critical_flows: 1,
        };
        let (modify, _) = scorer.score(base);
        let (delete, _) = scorer.score(RiskInput {
            operation: SimulationOperation::Delete,
            ..base
        });

        assert!(delete > modify);
        assert_eq!(delete - modify, 10.0);
    }

    #[test]
    fn score_is_clamped_to_one_hundred() {
        let (score, _) = RiskScorer::default().score(RiskInput {
            operation: SimulationOperation::Delete,
            caller_count: usize::MAX,
            dependency_count: usize::MAX,
            cross_module_edges: usize::MAX,
            reached_depth: 100,
            max_depth: 100,
            critical_flows: usize::MAX,
        });

        assert_eq!(score, 100.0);
    }
}
