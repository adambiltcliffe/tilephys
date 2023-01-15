use crate::physics::IntRect;
use hecs::Entity;
use smallset::SmallSet;
use std::collections::HashMap;

const BUCKET_SIZE: i32 = 128;

pub struct SpatialIndex {
    buckets: HashMap<(i32, i32), SmallSet<[Entity; 8]>>,
}

fn get_bounds(rect: &IntRect) -> (i32, i32, i32, i32) {
    (
        rect.x.div_euclid(BUCKET_SIZE),
        (rect.x + rect.w - 1).div_euclid(BUCKET_SIZE),
        rect.y.div_euclid(BUCKET_SIZE),
        (rect.y + rect.h - 1).div_euclid(BUCKET_SIZE),
    )
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    pub fn insert_at(&mut self, entity: Entity, rect: &IntRect) {
        let (min_kx, max_kx, min_ky, max_ky) = get_bounds(rect);
        for kx in min_kx..=max_kx {
            for ky in min_ky..=max_ky {
                let v = self.buckets.entry((kx, ky)).or_insert_with(SmallSet::new);
                v.insert(entity);
            }
        }
    }

    pub fn remove_at(&mut self, entity: Entity, rect: &IntRect) {
        let (min_kx, max_kx, min_ky, max_ky) = get_bounds(rect);
        for kx in min_kx..=max_kx {
            for ky in min_ky..=max_ky {
                let v = self.buckets.entry((kx, ky)).or_insert_with(SmallSet::new);
                v.remove(&entity);
            }
        }
    }

    pub fn entities(&self, rect: &IntRect) -> SmallSet<[Entity; 8]> {
        let mut result: SmallSet<[Entity; 8]> = SmallSet::new();
        let (min_kx, max_kx, min_ky, max_ky) = get_bounds(rect);
        for kx in min_kx..=max_kx {
            for ky in min_ky..=max_ky {
                if let Some(set) = self.buckets.get(&(kx, ky)) {
                    for id in set.iter() {
                        result.insert(*id);
                    }
                }
            }
        }
        result
    }

    #[allow(dead_code)]
    pub fn debug(&self) {
        println!("spatial index has {} buckets", self.buckets.len());
        let mut counts = HashMap::<usize, usize>::new();
        for s in self.buckets.values() {
            *counts.entry(s.len()).or_insert(0) += 1;
        }
        let mut k: Vec<usize> = counts.keys().copied().collect();
        k.sort();
        for key in k {
            println!("{} buckets with {} entries", counts[&key], key);
        }
    }
}
