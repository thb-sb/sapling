/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

mod save_store;
mod version;

pub use self::save_store::IdDagSaveStore;
pub use self::version::SqlIdDagVersionStore;
