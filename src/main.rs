#![feature(total_cmp)]

use std::fmt::Display;

use noise::{NoiseFn, Perlin};
use rayon::{iter::IntoParallelRefMutIterator, slice::ParallelSliceMut};


#[derive(Debug, Clone)]
struct Node {
    x: usize,
    y: usize,
    parent: Option<usize>,
    aggregated_cost: f64,
    is_path : bool
}

struct SimulationSpace {
    nodes: Vec<Node>,
    width: usize,
    height: usize,
    noise_scale: f64,
}

impl SimulationSpace {
    fn new(width: usize, height: usize, noise_scale: f64) -> Self {
        let mut simulation_nodes = Vec::new();
        simulation_nodes.reserve((width * height) as usize);
        for x in 0..width {
            for y in 0..height {
                simulation_nodes.push(Node {
                    x,
                    y,
                    parent: None,
                    aggregated_cost: 0.0,
                    is_path: false
                });
            }
        }
        SimulationSpace {
            nodes: simulation_nodes,
            width,
            height,
            noise_scale,
        }
    }
    fn simulation_step<'a>(&'a mut self, x: usize) -> (&'a [Node], &'a mut [Node]) {
        let (left, right) = self.nodes.split_at_mut(x * self.height);
        let previous = if x > 0 {
            let (_, previous) = left.split_at((x - 1) * self.height);
            previous
        } else {
            left
        };
        let next = if x < self.width - 1 {
            let (next, _) = right.split_at_mut(self.height);
            next
        } else {
            right
        };
        (previous, next)
    }
    fn get_cost_field(&self) -> CostField {
        CostField {
            width: self.width,
            height: self.height,
            perlin: Perlin::new(),
            noise_scale: self.noise_scale,
        }
    }
}

impl Display for SimulationSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let cost = self.nodes[x * self.height + y].aggregated_cost;
                if cost != 0.0 {
                    write!(f, "{:+.2} ", cost)?;
                } else {
                    write!(f, " x ")?;
                }
            }
            writeln!(f, "")?;
        }
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(parent_id) = self.nodes[x * self.height + y].parent {
                    write!(f, "{:#2} ", parent_id)?;
                } else {
                    write!(f, " x ")?;
                }
            }
            writeln!(f, "")?;
        }
        let cost_f = self.get_cost_field();

        let mut min_cost = f64::MAX;
        let mut max_cost = f64::MIN;
        for y in 0..self.height {
            for x in 1..self.width {
                if let Some(parent_id) = self.nodes[x * self.height + y].parent {
                    let cost = cost_f.get_cost(
                        &self.nodes[x * self.height + y],
                        &self.nodes[(x - 1) * self.height + parent_id],
                    );
                    min_cost = min_cost.min(cost);
                    max_cost = max_cost.max(cost);
                }
            }
        }

        for y in 0..self.height {
            write!(f, "\x1b[38;2;255;0;0m\x1b[48;2;0;0;0m")?;
            write!(f, " x ")?;
            for x in 1..self.width {
                if let Some(parent_id) = self.nodes[x * self.height + y].parent {
                    let cost = cost_f.get_cost(
                        &self.nodes[x * self.height + y],
                        &self.nodes[(x - 1) * self.height + parent_id],
                    );
                    let is_path = 
                    if self.nodes[x * self.height + y].is_path {
                        "\x1b[38;2;0;255;0m"
                    } else {
                        "\x1b[38;2;255;0;0m"
                    };
                    write!(
                        f,
                        "{2}\x1b[48;2;{0};{0};{0}m {1:+.2} ",
                        255 - (255.0 * (cost - min_cost) / (max_cost - min_cost)) as u32,
                        cost, is_path
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

struct CostField {
    width: usize,
    height: usize,
    noise_scale: f64,
    perlin: Perlin,
}
impl CostField {
    fn get_cost(&self, curr: &Node, prev: &Node) -> f64 {
        let x = (curr.x + prev.x) as f64 / 2.0;
        let y = (curr.y + prev.y) as f64 / 2.0;
        let energy_needed = 1.05
            + self.perlin.get([
                x / self.width as f64 * self.noise_scale,
                y / self.height as f64 * self.noise_scale,
            ]);
        let y_diff = curr.y as f64 - prev.y as f64;
        let distance = (y_diff * y_diff + 1.0).sqrt();
        energy_needed * distance
    }
}

fn main() {
    let mut simulation = SimulationSpace::new(1 * 1024, 1 * 1024, 6.0);
    let cost_field = simulation.get_cost_field();

    for x in 0..simulation.width {
        let (previous, current) = simulation.simulation_step(x);
        for curr in current.as_parallel_slice_mut() {
            for prev in previous {
                curr.parent = match curr.parent {
                    None => {
                        curr.aggregated_cost =
                            cost_field.get_cost(curr, prev) + prev.aggregated_cost;
                        Some(prev.y)
                    }
                    Some(_)
                        if curr.aggregated_cost
                            >= prev.aggregated_cost + cost_field.get_cost(curr, prev) =>
                    {
                        curr.aggregated_cost =
                            cost_field.get_cost(curr, prev) + prev.aggregated_cost;
                        Some(prev.y)
                    }
                    _ => curr.parent,
                };
            }
        }
    }
    let last_column =
        (simulation.width - 1) * simulation.height..simulation.width * simulation.height;
    let mut target = simulation.nodes[last_column].iter_mut().min_by(|a,b|{a.aggregated_cost.total_cmp(&b.aggregated_cost)}).unwrap();
    target.is_path = true;

    for x in 1..=simulation.width {
        if let Some(parent_id) = target.parent {
            target = &mut simulation.nodes[ (simulation.width - x) * simulation.height + parent_id];
            println!("{} {}", x, target.x);

            target.is_path = true;
        }
    } 
    

    // println!("{}", simulation);
}
