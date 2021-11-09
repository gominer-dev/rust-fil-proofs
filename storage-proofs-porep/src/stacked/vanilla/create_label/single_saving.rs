use crate::stacked::ExpLabelsBuffer;
use anyhow::{Context, Result};
use filecoin_hashers::Hasher;
use log::info;
use merkletree::store::StoreConfig;
use sha2raw::Sha256;
use std::marker::PhantomData;
use storage_proofs_core::{
    drgraph::Graph,
    merkle::MerkleTreeTrait,
    util::{data_at_node_offset, NODE_SIZE},
};

use crate::stacked::vanilla::{
    cache::ParentCache,
    create_label::{prepare_layers, write_layer},
    proof::LayerState,
    Labels, StackedBucketGraph,
};

#[allow(clippy::type_complexity)]
pub fn create_labels_for_encoding<Tree: 'static + MerkleTreeTrait, T: AsRef<[u8]>>(
    graph: &StackedBucketGraph<Tree::Hasher>,
    parents_cache: &mut ParentCache,
    layers: usize,
    replica_id: T,
    config: StoreConfig,
) -> Result<(Labels<Tree>, Vec<LayerState>)> {
    info!("generate labels");

    let layer_states = prepare_layers::<Tree>(graph, &config, layers);

    let layer_size = graph.size() * NODE_SIZE;
    // NOTE: this means we currently keep 2x sector size around, to improve speed.
    let mut layer_labels = vec![0u8; layer_size]; // Buffer for labels of the current layer
    let mut exp_labels = ExpLabelsBuffer::new(8, true)?; // Buffer for labels of the previous layer, needed for expander parents
    for (layer, layer_state) in (1..=layers).zip(layer_states.iter()) {
        info!("generating layer: {}", layer);

        parents_cache.reset()?;

        if layer == 1 {
            for node in 0..graph.size() {
                create_label(
                    graph,
                    Some(parents_cache),
                    &replica_id,
                    &mut layer_labels,
                    layer,
                    node,
                )?;
            }
        } else {
            for node in 0..graph.size() {
                create_label_exp(
                    graph,
                    Some(parents_cache),
                    &replica_id,
                    &mut exp_labels,
                    &mut layer_labels,
                    layer,
                    node,
                )?;
            }
        }

        // Write the result to disk to avoid keeping it in memory all the time.
        let layer_config = &layer_state.config;

        info!("  storing labels on disk");
        write_layer(&layer_labels, layer_config).context("failed to store labels")?;

        info!(
            "  generated layer {} store with id {}",
            layer, layer_config.id
        );

        info!("  setting exp parents");
        exp_labels.clear()?;
        for node in 0..graph.size() {
            prefill_exp_labels_data(graph, &mut exp_labels, &mut layer_labels, node + 1)?;
        }
        exp_labels.flip()?;
    }

    Ok((
        Labels::<Tree> {
            labels: layer_states.iter().map(|s| s.config.clone()).collect(),
            _h: PhantomData,
        },
        layer_states,
    ))
}

pub fn create_label<H: Hasher, T: AsRef<[u8]>>(
    graph: &StackedBucketGraph<H>,
    cache: Option<&mut ParentCache>,
    replica_id: T,
    layer_labels: &mut [u8],
    layer_index: usize,
    node: usize,
) -> Result<()> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 32];

    buffer[..4].copy_from_slice(&(layer_index as u32).to_be_bytes());
    buffer[4..12].copy_from_slice(&(node as u64).to_be_bytes());
    hasher.input(&[replica_id.as_ref(), &buffer[..]][..]);

    // hash parents for all non 0 nodes
    let hash = if node > 0 {
        // prefetch previous node, which is always a parent
        let prev = &layer_labels[(node - 1) * NODE_SIZE..node * NODE_SIZE];
        prefetch!(prev.as_ptr() as *const i8);

        graph.copy_parents_data(node as u32, &*layer_labels, hasher, cache)?
    } else {
        hasher.finish()
    };

    // store the newly generated key
    let start = data_at_node_offset(node);
    let end = start + NODE_SIZE;
    layer_labels[start..end].copy_from_slice(&hash[..]);

    // strip last two bits, to ensure result is in Fr.
    layer_labels[end - 1] &= 0b0011_1111;

    Ok(())
}

pub fn create_label_exp<H: Hasher, T: AsRef<[u8]>>(
    graph: &StackedBucketGraph<H>,
    cache: Option<&mut ParentCache>,
    replica_id: T,
    exp_parents_data: &mut ExpLabelsBuffer,
    layer_labels: &mut [u8],
    layer_index: usize,
    node: usize,
) -> Result<()> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 32];

    buffer[0..4].copy_from_slice(&(layer_index as u32).to_be_bytes());
    buffer[4..12].copy_from_slice(&(node as u64).to_be_bytes());
    hasher.input(&[replica_id.as_ref(), &buffer[..]][..]);

    // hash parents for all non 0 nodes
    let hash = if node > 0 {
        // prefetch previous node, which is always a parent
        let prev = &layer_labels[(node - 1) * NODE_SIZE..node * NODE_SIZE];
        prefetch!(prev.as_ptr() as *const i8);

        graph.copy_parents_data_exp_buffer(
            node as u32,
            layer_labels,
            exp_parents_data,
            hasher,
            cache,
        )?
    } else {
        hasher.finish()
    };

    // store the newly generated key
    let start = data_at_node_offset(node);
    let end = start + NODE_SIZE;
    layer_labels[start..end].copy_from_slice(&hash[..]);

    // strip last two bits, to ensure result is in Fr.
    layer_labels[end - 1] &= 0b0011_1111;

    Ok(())
}

pub fn prefill_exp_labels_data<H: Hasher>(
    graph: &StackedBucketGraph<H>,
    exp_parents_data: &mut ExpLabelsBuffer,
    layer_labels: &mut [u8],
    node: usize,
) -> Result<()> {
    // prefetch previous node, which is always a parent
    let prev = &layer_labels[(node - 1) * NODE_SIZE..node * NODE_SIZE];
    prefetch!(prev.as_ptr() as *const i8);

    graph.prefill_parents_data_exp_buffer(node as u32, layer_labels, exp_parents_data)?;

    Ok(())
}

pub use self::single::create_labels_for_decoding;
mod single {
    include!("single.rs");
}
