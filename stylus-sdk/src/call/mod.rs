// Copyright 2022-2023, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/stylus/licenses/COPYRIGHT.md

//! Call other contracts.
//!
//! There are two primary ways to make calls to other contracts via the Stylus SDK.
//! - [`CallContext`] for richly-typed calls.
//! - The `unsafe` [`RawCall`] for `unsafe`, bytes-in bytes-out calls.
//!
//! Additional helpers exist for specific use-cases like [`transfer_eth`].

use alloc::vec::Vec;
use alloy_primitives::Address;

pub use self::{context::Call, error::Error, raw::RawCall, traits::*, transfer::transfer_eth};

pub(crate) use raw::CachePolicy;

#[cfg(all(feature = "storage-cache", feature = "reentrant"))]
use crate::storage::Storage;

mod context;
mod error;
mod raw;
mod traits;
mod transfer;

macro_rules! unsafe_reentrant {
    ($block:block) => {
        #[cfg(all(feature = "storage-cache", feature = "reentrant"))]
        unsafe {
            $block
        }

        #[cfg(not(all(feature = "storage-cache", feature = "reentrant")))]
        $block
    };
}

/// Static calls the contract at the given address.
pub fn static_call(
    context: impl StaticCallContext,
    to: Address,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    #[cfg(all(feature = "storage-cache", feature = "reentrant"))]
    Storage::flush(); // flush storage to persist changes, but don't invalidate the cache

    unsafe_reentrant! {{
        RawCall::new_static()
            .gas(context.gas())
            .call(to, data)
            .map_err(Error::Revert)
    }}
}

/// Delegate calls the contract at the given address.
///
/// # Safety
///
/// A delegate call must trust the other contract to uphold safety requirements.
/// Though this function clears any cached values, the other contract may arbitrarily change storage,
/// spend ether, and do other things one should never blindly allow other contracts to do.
pub unsafe fn delegate_call(
    context: impl MutatingCallContext,
    to: Address,
    data: &[u8],
) -> Result<Vec<u8>, Error> {
    #[cfg(all(feature = "storage-cache", feature = "reentrant"))]
    Storage::clear(); // clear the storage to persist changes, invalidating the cache

    RawCall::new_with_value(context.value())
        .gas(context.gas())
        .call(to, data)
        .map_err(Error::Revert)
}

/// Calls the contract at the given address.
pub fn call(context: impl MutatingCallContext, to: Address, data: &[u8]) -> Result<Vec<u8>, Error> {
    #[cfg(all(feature = "storage-cache", feature = "reentrant"))]
    Storage::clear(); // clear the storage to persist changes, invalidating the cache

    unsafe_reentrant! {{
        RawCall::new_with_value(context.value())
            .gas(context.gas())
            .call(to, data)
            .map_err(Error::Revert)
    }}
}
