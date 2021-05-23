use crate::score::Score;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::sync::Arc;
pub trait CostField {
    type LeftNodeType;
    type RightNodeType;
    fn get_cost(&self, a: &Self::LeftNodeType, b: &Self::RightNodeType) -> Score;
}

pub trait LeftNode {
    fn aggregated_cost(&self) -> Score;
}
pub trait RightNode {
    fn set_aggregated_cost(&mut self, score: Score);
    fn aggregated_cost(&self) -> Score;
}
pub trait Simulation {
    type LeftNodeType: LeftNode + Send + Sync;
    type RightNodeType: RightNode + Send + Sync;

    type CostFieldType: CostField<LeftNodeType = Self::LeftNodeType, RightNodeType = Self::RightNodeType>
        + Send
        + Sync;
    fn prepare_step_slices(
        &mut self,
        iteration: usize,
    ) -> (&[Self::LeftNodeType], &mut [Self::RightNodeType]);

    fn get_cost_field(&self) -> Arc<Self::CostFieldType>;

    fn simulate_par(&mut self, iteration: usize) {
        let cost_field = self.get_cost_field();
        let (previous, current) = self.prepare_step_slices(iteration);
        current.par_iter_mut().for_each(|curr| {
            if let Some((cost, prev_node)) = previous
                .iter()
                .filter_map(|prev| {
                    Some((
                        (cost_field.get_cost(prev, curr) + prev.aggregated_cost())?,
                        prev,
                    ))
                })
                .min_by(|(cost0, _), (cost1, _)| cost0.cmp(cost1))
            {
                curr.set_aggregated_cost(cost);
                Self::set_parent_of(prev_node, curr);
            }
        });
    }
    fn set_parent_of(parent: &Self::LeftNodeType, child: &mut Self::RightNodeType);
}
