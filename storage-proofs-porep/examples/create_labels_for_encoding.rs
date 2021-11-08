use digest::consts::*;
use filecoin_hashers::poseidon::PoseidonHasher;
use log::LevelFilter;
use merkletree::store::StoreConfig;
use storage_proofs_core::api_version::ApiVersion;
use storage_proofs_core::cache_key::CacheKey;
use storage_proofs_core::drgraph::BASE_DEGREE;
use storage_proofs_core::merkle::LCTree;
use storage_proofs_core::util::NODE_SIZE;
use storage_proofs_porep::stacked::create_label::single_saving::create_labels_for_encoding;
use storage_proofs_porep::stacked::{StackedBucketGraph, EXP_DEGREE};
use tempfile::tempdir;

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Trace)
        .init();
    let layers = 11;
    let replica_id = [9u8; 32];
    let sector_size = 32 * NODE_SIZE;

    // These PoRepIDs are only useful to distinguish legacy/new sectors.
    // They do not correspond to registered proofs of the sizes used here.
    let new_porep_id = [123; 32];

    let nodes = sector_size / NODE_SIZE;

    let cache_dir = tempdir().expect("tempdir failure");
    let config = StoreConfig::new(
        cache_dir.path(),
        CacheKey::CommDTree.to_string(),
        nodes.trailing_zeros() as usize,
    );

    let graph = StackedBucketGraph::<PoseidonHasher>::new(
        None,
        nodes,
        BASE_DEGREE,
        EXP_DEGREE,
        new_porep_id,
        ApiVersion::V1_1_0,
    )
    .expect("stacked bucket graph new failed");
    let mut cache = graph.parent_cache().expect("parent_cache failed");

    let _labels = create_labels_for_encoding::<LCTree<PoseidonHasher, U8, U0, U2>, _>(
        &graph, &mut cache, layers, replica_id, config,
    )
    .expect("create_labels_for_decoding failed");
}
