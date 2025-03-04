//! Blockchain functions - chain height, generated coins, etc.

//---------------------------------------------------------------------------------------------------- Import
use crate::{
    database::DatabaseRo,
    error::RuntimeError,
    ops::macros::doc_error,
    tables::{BlockHeights, BlockInfos},
    types::BlockHeight,
};

//---------------------------------------------------------------------------------------------------- Free Functions
/// Retrieve the height of the chain.
///
/// This returns the chain-tip, not the [`top_block_height`].
///
/// For example:
/// - The blockchain has 0 blocks => this returns `0`
/// - The blockchain has 1 block (height 0) => this returns `1`
/// - The blockchain has 2 blocks (height 1) => this returns `2`
///
/// So the height of a new block would be `chain_height()`.
#[doc = doc_error!()]
#[inline]
pub fn chain_height(
    table_block_heights: &impl DatabaseRo<BlockHeights>,
) -> Result<BlockHeight, RuntimeError> {
    table_block_heights.len()
}

/// Retrieve the height of the top block.
///
/// This returns the height of the top block, not the [`chain_height`].
///
/// For example:
/// - The blockchain has 0 blocks => this returns `Err(RuntimeError::KeyNotFound)`
/// - The blockchain has 1 block (height 0) => this returns `Ok(0)`
/// - The blockchain has 2 blocks (height 1) => this returns `Ok(1)`
///
/// Note that in cases where no blocks have been written to the
/// database yet, an error is returned: `Err(RuntimeError::KeyNotFound)`.
///
#[doc = doc_error!()]
#[inline]
pub fn top_block_height(
    table_block_heights: &impl DatabaseRo<BlockHeights>,
) -> Result<BlockHeight, RuntimeError> {
    match table_block_heights.len()? {
        0 => Err(RuntimeError::KeyNotFound),
        height => Ok(height - 1),
    }
}

/// Check how many cumulative generated coins there have been until a certain [`BlockHeight`].
///
/// This returns the total amount of Monero generated up to `block_height`
/// (including the block itself) in atomic units.
///
/// For example:
/// - on the genesis block `0`, this returns the amount block `0` generated
/// - on the next block `1`, this returns the amount block `0` and `1` generated
///
/// If no blocks have been added and `block_height == 0`
/// (i.e., the cumulative generated coins before genesis block is being calculated),
/// this returns `Ok(0)`.
#[doc = doc_error!()]
#[inline]
pub fn cumulative_generated_coins(
    block_height: &BlockHeight,
    table_block_infos: &impl DatabaseRo<BlockInfos>,
) -> Result<u64, RuntimeError> {
    match table_block_infos.get(block_height) {
        Ok(block_info) => Ok(block_info.cumulative_generated_coins),
        Err(RuntimeError::KeyNotFound) if block_height == &0 => Ok(0),
        Err(e) => Err(e),
    }
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use cuprate_test_utils::data::{block_v16_tx0, block_v1_tx2, block_v9_tx3};

    use super::*;
    use crate::{
        ops::block::add_block,
        tables::Tables,
        tests::{assert_all_tables_are_empty, tmp_concrete_env, AssertTableLen},
        transaction::TxRw,
        Env, EnvInner,
    };

    /// Tests all above functions.
    ///
    /// Note that this doesn't test the correctness of values added, as the
    /// functions have a pre-condition that the caller handles this.
    ///
    /// It simply tests if the proper tables are mutated, and if the data
    /// stored and retrieved is the same.
    #[test]
    fn all_blockchain_functions() {
        let (env, _tmp) = tmp_concrete_env();
        let env_inner = env.env_inner();
        assert_all_tables_are_empty(&env);

        let mut blocks = [
            block_v1_tx2().clone(),
            block_v9_tx3().clone(),
            block_v16_tx0().clone(),
        ];
        let blocks_len = u64::try_from(blocks.len()).unwrap();

        // Add blocks.
        {
            let tx_rw = env_inner.tx_rw().unwrap();
            let mut tables = env_inner.open_tables_mut(&tx_rw).unwrap();

            assert!(matches!(
                top_block_height(tables.block_heights()),
                Err(RuntimeError::KeyNotFound),
            ));
            assert_eq!(
                0,
                cumulative_generated_coins(&0, tables.block_infos()).unwrap()
            );

            for (i, block) in blocks.iter_mut().enumerate() {
                let i = u64::try_from(i).unwrap();
                // HACK: `add_block()` asserts blocks with non-sequential heights
                // cannot be added, to get around this, manually edit the block height.
                block.height = i;
                add_block(block, &mut tables).unwrap();
            }

            // Assert reads are correct.
            AssertTableLen {
                block_infos: 3,
                block_blobs: 3,
                block_heights: 3,
                key_images: 69,
                num_outputs: 41,
                pruned_tx_blobs: 0,
                prunable_hashes: 0,
                outputs: 111,
                prunable_tx_blobs: 0,
                rct_outputs: 8,
                tx_blobs: 8,
                tx_ids: 8,
                tx_heights: 8,
                tx_unlock_time: 3,
            }
            .assert(&tables);

            assert_eq!(blocks_len, chain_height(tables.block_heights()).unwrap());
            assert_eq!(
                blocks_len - 1,
                top_block_height(tables.block_heights()).unwrap()
            );
            assert_eq!(
                cumulative_generated_coins(&0, tables.block_infos()).unwrap(),
                14_535_350_982_449,
            );
            assert_eq!(
                cumulative_generated_coins(&1, tables.block_infos()).unwrap(),
                17_939_125_004_612,
            );
            assert_eq!(
                cumulative_generated_coins(&2, tables.block_infos()).unwrap(),
                18_539_125_004_612,
            );
            assert!(matches!(
                cumulative_generated_coins(&3, tables.block_infos()),
                Err(RuntimeError::KeyNotFound),
            ));

            drop(tables);
            TxRw::commit(tx_rw).unwrap();
        }
    }
}
