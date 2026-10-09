//! Git sync (docs/v1-sync-spec.md), built in steps: snapshot shards, reconcile, apply, Git.
// The pieces are used by the sync run, which is wired in step S4.
#![cfg_attr(not(test), expect(dead_code, reason = "used by the sync run (S4)"))]

pub mod shards;
