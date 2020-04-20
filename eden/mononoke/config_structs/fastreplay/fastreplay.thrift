// @generated SignedSource<<e660e73a362306ee7fc69717a1ac484c>>
// DO NOT EDIT THIS FILE MANUALLY!
// This file is a mechanical copy of the version in the configerator repo. To
// modify it, edit the copy in the configerator repo instead and copy it over by
// running the following in your fbcode directory:
//
// configerator-thrift-updater scm/mononoke/fastreplay/fastreplay.thrift

namespace py configerator.fastreplay.fastreplay

struct FastReplayConfig {
  // What % of traffic should be replayed?
  1: i64 admission_rate;

  // How many requests should be executed concurrently?
  2: i64 max_concurrency;

  // One in scuba_sampling_target entries will be logged. This should be > 0.
  3: i64 scuba_sampling_target;

  // Which repos shouldn't have their traffic replayed?
  4: set<string> skipped_repos;
}
