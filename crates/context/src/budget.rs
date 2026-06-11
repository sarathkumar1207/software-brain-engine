use crate::ranking::RankedSymbol;
use rustc_hash::FxHashSet;
use sbe_graph::SymbolId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub max_tokens: usize,
}

impl Budget {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }
}

pub fn apply_budget(
    ranked: &[RankedSymbol],
    required: &FxHashSet<SymbolId>,
    token_estimate: impl Fn(SymbolId) -> usize,
    budget: Option<Budget>,
) -> Vec<SymbolId> {
    let Some(budget) = budget else {
        return ranked.iter().map(|rank| rank.symbol_id).collect();
    };

    let mut selected: FxHashSet<SymbolId> = required.iter().copied().collect();
    let mut total: usize = selected.iter().map(|id| token_estimate(*id)).sum();

    for rank in ranked {
        if selected.contains(&rank.symbol_id) {
            continue;
        }
        let tokens = token_estimate(rank.symbol_id);
        if total.saturating_add(tokens) <= budget.max_tokens {
            selected.insert(rank.symbol_id);
            total += tokens;
        }
    }

    let mut output: Vec<SymbolId> = ranked
        .iter()
        .filter(|rank| selected.contains(&rank.symbol_id))
        .map(|rank| rank.symbol_id)
        .collect();
    for required_id in required {
        if !output.contains(required_id) {
            output.push(*required_id);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_prunes_low_ranked_symbols_but_keeps_required() {
        let ranked = vec![
            RankedSymbol {
                symbol_id: 1,
                score: 100.0,
            },
            RankedSymbol {
                symbol_id: 2,
                score: 90.0,
            },
            RankedSymbol {
                symbol_id: 3,
                score: 10.0,
            },
        ];
        let mut required = FxHashSet::default();
        required.insert(1);
        required.insert(2);

        let selected = apply_budget(&ranked, &required, |_| 100, Some(Budget::new(210)));

        assert_eq!(selected, vec![1, 2]);
    }
}
