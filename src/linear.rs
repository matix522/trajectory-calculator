use std::{error::Error, fmt::Display, fs::File};

use crate::memory_profiler::AllocationData;
use crate::score::Score;
use crate::simulation::{LeftNode, RightNode};
use crate::{
    score,
    simulation::{CostField, Simulation},
};
use noise::{NoiseFn, Perlin};
use std::io::Write;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct Node {
    x: usize,
    y: usize,
    parent: Option<usize>,
    aggregated_cost: Score,
    is_path: bool,
}
impl LeftNode for Node {
    fn aggregated_cost(&self) -> Score {
        self.aggregated_cost
    }
}
impl RightNode for Node {
    fn set_aggregated_cost(&mut self, score: Score) {
        self.aggregated_cost = score;
    }
    fn aggregated_cost(&self) -> Score {
        self.aggregated_cost
    }
}

struct SimulationSpace {
    nodes: Vec<Vec<Node>>,
    width: usize,
    height: usize,
    noise: Arc<PerlinCostField>,
}

impl Simulation for SimulationSpace {
    type LeftNodeType = Node;
    type RightNodeType = Node;
    type CostFieldType = PerlinCostField;

    fn prepare_step_slices(
        &mut self,
        x: usize,
    ) -> (&[Self::LeftNodeType], &mut [Self::RightNodeType]) {
        let right: Vec<_> = (0..self.height)
            .map(|y| Node {
                x,
                y,
                parent: None,
                aggregated_cost: Score::new(0.0),
                is_path: false,
            })
            .collect();
        self.nodes.push(right);
        let len = self.nodes.len();
        let (left, right) = self.nodes.split_at_mut(len - 1);
        (&left[len - 2], &mut right[0])
    }
    fn get_cost_field(&self) -> Arc<Self::CostFieldType> {
        self.noise.clone()
    }

    fn set_parent_of(parent: &Self::LeftNodeType, child: &mut Self::RightNodeType) {
        child.parent = Some(parent.y);
    }
}

impl SimulationSpace {
    fn new(width: usize, height: usize, noise_scale: f64) -> Self {
        let mut simulation_nodes = Vec::new();
        simulation_nodes.reserve(width as usize);
        simulation_nodes.push(Vec::new());
        SimulationSpace {
            nodes: simulation_nodes,
            width,
            height,
            noise: Arc::new(PerlinCostField {
                width,
                height,
                perlin: Perlin::new(),
                noise_scale,
            }),
        }
    }
}
struct PerlinCostField {
    width: usize,
    height: usize,
    noise_scale: f64,
    perlin: Perlin,
}
impl CostField for PerlinCostField {
    type LeftNodeType = Node;
    type RightNodeType = Node;
    fn get_cost(&self, prev: &Self::LeftNodeType, curr: &Self::RightNodeType) -> Score {
        let x = (curr.x + prev.x) as f64 / 2.0;
        let y = (curr.y + prev.y) as f64 / 2.0;
        let energy_needed = 1.05
            + self.perlin.get([
                x / self.width as f64 * self.noise_scale,
                y / self.height as f64 * self.noise_scale,
            ]);
        let y_diff = curr.y as f64 - prev.y as f64;
        let distance = (y_diff * y_diff + 1.0).sqrt();
        Score::new(energy_needed * distance)
    }
}

pub fn linear(out_path: String, x: usize, y: usize, debug: bool) -> Result<(), Box<dyn Error>> {
    AllocationData::collect_data()?;
    let mut simulation = SimulationSpace::new(x, y, 6.0);
    for x in 0..simulation.width {
        print!("{} ", x);
        std::io::stdout().flush().unwrap();
        simulation.simulate_par(x);
        AllocationData::collect_data()?;
    }
    println!("Done");
    let last_column = simulation.width - 1;
    let mut target = simulation.nodes[last_column]
        .iter_mut()
        .min_by_key(|x| x.aggregated_cost)
        .unwrap();
    let mut r_path = vec![];

    for x in 1..=simulation.width {
        if let Some(parent_id) = target.parent {
            target = &mut simulation.nodes[simulation.width - x][parent_id];
            target.is_path = true;
            r_path.push(target.y);
        }
    }
    if debug {
        println!("{}", simulation);
    }
    drop(simulation);
    r_path.reverse();
    let path = r_path;

    println!("{:?}", path);
    AllocationData::collect_data()?;
    AllocationData::dump_data(&mut File::create(out_path)?)?;
    Ok(())
}

impl Display for SimulationSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let cost = self.nodes[x][y].aggregated_cost;
                if cost == Score::new(0.0) {
                    write!(f, " x ")?;
                } else {
                    write!(f, "{:+.2} ", cost.0)?;
                }
            }
            writeln!(f)?;
        }
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(parent_id) = self.nodes[x][y].parent {
                    write!(f, "{:#2} ", parent_id)?;
                } else {
                    write!(f, " x ")?;
                }
            }
            writeln!(f)?;
        }
        let cost_f = self.get_cost_field();

        let mut min_cost = score::INFINITY.0;
        let mut max_cost = score::NEG_INFINITY.0;
        for y in 0..self.height {
            for x in 1..self.width {
                if let Some(parent_id) = self.nodes[x][y].parent {
                    let cost = cost_f
                        .get_cost(&self.nodes[x][y], &self.nodes[x - 1][parent_id])
                        .0;
                    min_cost = min_cost.min(cost);
                    max_cost = max_cost.max(cost);
                }
            }
        }

        for y in 0..self.height {
            write!(f, "\x1b[38;2;255;0;0m\x1b[48;2;0;0;0m")?;
            write!(f, " x ")?;
            for x in 1..self.width {
                if let Some(parent_id) = self.nodes[x][y].parent {
                    let cost = cost_f
                        .get_cost(&self.nodes[x][y], &self.nodes[x - 1][parent_id])
                        .0;
                    let is_path = if self.nodes[x][y].is_path {
                        "\x1b[38;2;0;255;0m"
                    } else {
                        "\x1b[38;2;255;0;0m"
                    };
                    write!(
                        f,
                        "{2}\x1b[48;2;{0};{0};{0}m {1:+.2} ",
                        255 - (255.0 * (cost - min_cost) / (max_cost - min_cost)) as u32,
                        cost,
                        is_path
                    )?;
                } else {
                    write!(f, "\x1b[48;2;0;0;0m x ")?;
                }
            }
            writeln!(f, "\x1b[0m")?;
        }
        write!(f, "\x1b[0m")?;
        Ok(())
    }
}
