use sbe_context::ContextPack;
use sbe_graph::{FileId, SymbolId};
use serde::{Deserialize, Serialize};

pub type FlowId = SymbolId;
pub type TestId = SymbolId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationOperation {
    Modify,
    Delete,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationRequest {
    pub operation: SimulationOperation,
    pub target_symbol: SymbolId,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn from_score(score: f32) -> Self {
        match score {
            score if score >= 80.0 => Self::Critical,
            score if score >= 55.0 => Self::High,
            score if score >= 30.0 => Self::Medium,
            _ => Self::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RiskBreakdown {
    pub caller_score: f32,
    pub dependency_score: f32,
    pub cross_module_score: f32,
    pub depth_score: f32,
    pub critical_flow_score: f32,
    pub operation_score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationReport {
    pub operation: SimulationOperation,
    pub target_symbol: SymbolId,
    pub risk_score: f32,
    pub risk_level: RiskLevel,
    pub risk_breakdown: RiskBreakdown,
    pub traversal_depth: usize,
    pub affected_symbols: Vec<SymbolId>,
    pub affected_files: Vec<FileId>,
    pub affected_flows: Vec<FlowId>,
    pub recommended_tests: Vec<TestId>,
    pub context_pack: ContextPack,
}
