//! Partition guard for lab tools. Lab code may read train and validation views only.

use ignis_core::contracts::{Partition, Split};
use ignis_core::views::View;

use crate::util::refused;

pub const LAB_PARTITIONS: [&str; 2] = ["train", "validation"];

pub fn assert_lab_partition(partition: &str) -> anyhow::Result<()> {
    if !LAB_PARTITIONS.contains(&partition) {
        return Err(refused(format!(
            "lab tools may only load {LAB_PARTITIONS:?}; {partition:?} is off-limits \
             (final-test outcomes are reachable only through `ignis eval reveal --final-test`)"
        )));
    }
    Ok(())
}

/// Keep views whose run lies in `partition`. Runs unknown to the split are dropped.
pub fn filter_views(views: Vec<View>, split: &Split, partition: &str) -> anyhow::Result<Vec<View>> {
    assert_lab_partition(partition)?;
    let want = if partition == "train" {
        Partition::Train
    } else {
        Partition::Validation
    };
    Ok(views
        .into_iter()
        .filter(|v| split.partition_of(v.run_id()) == Some(want))
        .collect())
}
