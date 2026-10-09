//! Git sync (docs/v1-sync-spec.md), built in steps: snapshot shards, reconcile, apply, Git.

pub mod apply;
pub mod git;
pub mod reconcile;
pub mod run;
pub mod runner;
pub mod shards;
pub mod state;
pub mod token;
