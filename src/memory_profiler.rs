use std::{
    error::Error,
    io::Write,
    sync::{
        atomic::{AtomicUsize, Ordering},
        RwLock,
    },
};

use itertools::Itertools;
use jemalloc_ctl::{epoch, stats};

lazy_static! {
    static ref ALLOCATION_DATA: RwLock<Vec<AllocationData>> = RwLock::new(Vec::new());
}

static ALLOCATION_DATA_ID: AtomicUsize = AtomicUsize::new(0);
#[derive(Debug, Clone, Copy)]
pub struct AllocationData {
    pub id: usize,
    pub allocated: usize,
    pub resident: usize,
    pub correction: usize,
}
impl AllocationData {
    fn get_data() -> Self {
        epoch::advance().unwrap();

        let allocated = stats::allocated::read().unwrap();
        let resident = stats::resident::read().unwrap();
        let correction =
            ALLOCATION_DATA.read().unwrap().capacity() * std::mem::size_of::<AllocationData>();
        AllocationData {
            id: ALLOCATION_DATA_ID.fetch_add(1, Ordering::Relaxed),
            allocated,
            resident,
            correction,
        }
    }
    pub fn collect_data() -> Result<(), Box<dyn Error>> {
        let data = AllocationData::get_data();
        ALLOCATION_DATA.write()?.push(data);
        Ok(())
    }
    pub fn dump_data<F: Write>(file: &mut F) -> Result<(), Box<dyn Error>> {
        writeln!(file, "id\tallocated\tresident\tcorrection")?;
        for data in ALLOCATION_DATA.read()?.iter().sorted_by_key(|&a| a.id) {
            writeln!(
                file,
                "{}\t{}\t{}\t{}",
                data.id, data.allocated, data.resident, data.correction
            )?;
        }
        Ok(())
    }
}
