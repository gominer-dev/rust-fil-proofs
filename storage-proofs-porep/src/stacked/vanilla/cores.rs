use core_allocator::*;
use lazy_static::lazy_static;
use anyhow::Result;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub enum Level {
    L3,
    L2,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoreAllocatorSettings {
    #[serde(default)]
    pub cpus: Option<Vec<usize>>,
    pub level: Level,
}

impl CoreAllocatorSettings {
    pub fn load() -> Result<Self> {
        let content = std::fs::read_to_string("core_allocator.yaml")?;
        let this = serde_yaml::from_str(&content)?;
        Ok(this)
    }
    pub fn get_allocator(&self) -> Result<Box<dyn CoreAllocator>> {
        let depth = match self.level {
            Level::L3 => {HierarchicalAllocator::L3_CACHE}
            Level::L2 => {HierarchicalAllocator::L2_CACHE}
        };
        let mut allocator = HierarchicalAllocator::new_at_depth(depth);
        if let Some(cpus) = self.cpus.clone() {
            allocator = allocator.on_cpu(cpus);
        }
        Ok(Box::new(allocator.finish()))
    }
}
pub fn checkout_core_group() -> Option<CoreGroup> {
    lazy_static! {
        static ref ALLOCATOR: Box<dyn CoreAllocator> = {
            CoreAllocatorSettings::load().unwrap().get_allocator().unwrap()
        };
    }
    Some(ALLOCATOR.allocate_core().unwrap())
}