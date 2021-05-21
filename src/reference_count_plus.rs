use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs::File,
    io::Write,
};

use crate::memory_profiler::AllocationData;
use crate::score::Score;
use crate::simulation::{LeftNode, RightNode};
use crate::simulation::{CostField, Simulation};

use noise::{NoiseFn, Perlin};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
#[derive(Debug, Clone)]
struct Parent {
    y: usize,
    parent: Option<Arc<Parent>>,
}
#[derive(Debug, Clone)]
struct Node {
    x: usize,
    y: usize,
    parent: Option<Arc<Parent>>,
    aggregated_cost: Score,
}

impl Default for Node {
    fn default() -> Self {
        Node::new(0, 0)
    }
}

impl LeftNode for Node {
    fn aggregated_cost(&self) -> Score {
        self.aggregated_cost
    }
}
impl RightNode for Node{
    fn set_aggregated_cost(&mut self, score: Score) {
        self.aggregated_cost = score;
    }
    fn aggregated_cost(&self) -> Score {
        self.aggregated_cost
    }
}

impl Node {
    fn reverse_path(self: Arc<Self>) -> ReversePath {
        ReversePath { value: Some(Arc::new(Parent { y: self.y, parent: self.parent.clone()} )) }
    }
    const fn new(x: usize, y: usize) -> Self {
        Node {
            x,
            y,
            parent: None,
            aggregated_cost: Score::new(0.0),
        }
    }
}

struct ReversePath {
    value: Option<Arc<Parent>>,
}
impl Iterator for ReversePath {
    type Item = Arc<Parent>;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.value.clone();
        self.value = v.as_ref()?.parent.clone();
        v
    }
}

struct SimulationSpace {
    width: usize,
    height: usize,
    cost_field: Arc<PerlinCostField>,
    previous: Vec<Node>,
    current: Vec<Node>,
    x: usize,
}

impl Simulation for SimulationSpace {
    type LeftNodeType = Node;
    type RightNodeType = Node;
    type CostFieldType = PerlinCostField;

    fn get_cost_field(&self) -> Arc<Self::CostFieldType> {
        self.cost_field.clone()
    }

    fn prepare_step_slices(&mut self, _: usize) -> (&[Self::LeftNodeType], &mut [Self::RightNodeType]) {
        std::mem::swap(&mut self.current , &mut self.previous);
        self.x += 1;
        self.current = 
            (0..self.height)
            .into_par_iter()
            .map(|y| Node::new(self.x, y))
            .collect();

        (&self.previous[..], &mut self.current[..])
    }

    fn set_parent_of(parent: &Self::LeftNodeType, child: &mut Self::RightNodeType) {
        child.parent = Some(Arc::new(Parent { y: parent.y, parent: parent.parent.clone()} ));
    }
}

impl SimulationSpace {
    fn new(width: usize, height: usize, noise_scale: f64) -> Self {
        SimulationSpace {
            width,
            height,
            cost_field: Arc::new(PerlinCostField {
                width,
                height,
                perlin: Perlin::new(),
                noise_scale,
            }),
            previous: Vec::new(),
            current: Vec::new(),
            x: 0,
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

pub fn reference_count_plus(
    out_path: String,
    x: usize,
    y: usize,
    _debug: bool,
) -> Result<(), Box<dyn Error>> {
    AllocationData::collect_data()?;
    let mut simulation = SimulationSpace::new(x, y, 6.0);
    for x in 0..simulation.width {
        print!("{} ", x);
        std::io::stdout().flush().unwrap();
        simulation.simulate_par(x);
        AllocationData::collect_data()?;
    }
    println!("Done");
    let target = simulation
        .current
        .iter_mut()
        .min_by_key(|x| LeftNode::aggregated_cost(*x))
        .unwrap()
        .clone();
    drop(simulation);

    let mut r_path = vec![];

    for node in Arc::new(target).reverse_path() {
        r_path.push(node.y);
    }
    r_path.reverse();
    let path = r_path;

    println!("{:?}", path);

    AllocationData::collect_data()?;
    AllocationData::dump_data(&mut File::create(out_path)?)?;
    Ok(())
}

impl Display for SimulationSpace {
    fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}
