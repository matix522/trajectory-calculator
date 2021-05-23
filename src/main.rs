#![feature(const_float_classify)]
#![feature(const_panic)]

use std::{borrow::Borrow, collections::HashMap, error::Error};
use structopt::StructOpt;

#[macro_use]
extern crate lazy_static;

mod linear;
mod memory_profiler;
mod naive;
mod reference_count;
mod reference_count_plus;
mod score;
mod simulation;
mod utils;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(StructOpt, Debug)]
#[structopt()]
struct ProgramOptions {
    #[structopt(short = "t", long, default_value = "naive")]
    simulation_type: String,
    #[structopt(short = "o", long, default_value = "/dev/null")]
    out_file: String,
    #[structopt(short = "x", long, default_value = "16")]
    width: usize,
    #[structopt(short = "y", long, default_value = "16")]
    height: usize,
    #[structopt(short, long)]
    debug: bool,
}
type SimulationFunc = fn(String, usize, usize, bool) -> Result<(), Box<dyn Error>>;

lazy_static! {
    static ref SIMULATIONS: HashMap<&'static str, SimulationFunc> = vec![
        ("rc", reference_count::reference_count as SimulationFunc),
        (
            "rc+",
            reference_count_plus::reference_count_plus as SimulationFunc
        ),
        ("naive", naive::naive as SimulationFunc),
        ("linear", linear::linear as SimulationFunc),
    ]
    .into_iter()
    .collect();
}
fn main() -> Result<(), Box<dyn Error>> {
    let opts = ProgramOptions::from_args();

    SIMULATIONS.get(&opts.simulation_type.borrow()).unwrap()(
        opts.out_file,
        opts.width,
        opts.height,
        opts.debug,
    )
}
