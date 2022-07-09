/**
 * Originally from Intertrait by CodeChain
 *
 * https://github.com/CodeChain-io/intertrait
 * https://crates.io/crates/intertrait/0.2.2
 *
 * Licensed under either of
 *
 * Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

 * at your option.
*/
use std::convert::TryInto;
use std::hash::{BuildHasherDefault, Hasher};
use std::mem::size_of;

/// A simple `Hasher` implementation tuned for performance.
#[derive(Default)]
pub struct FastHasher(u64);

/// A `BuildHasher` for `FastHasher`.
pub type BuildFastHasher = BuildHasherDefault<FastHasher>;

impl Hasher for FastHasher
{
    fn finish(&self) -> u64
    {
        self.0
    }

    fn write(&mut self, bytes: &[u8])
    {
        let mut bytes = bytes;
        while bytes.len() > size_of::<u64>() {
            let (u64_bytes, remaining) = bytes.split_at(size_of::<u64>());
            self.0 ^= u64::from_ne_bytes(u64_bytes.try_into().unwrap());
            bytes = remaining
        }
        self.0 ^= bytes
            .iter()
            .fold(0u64, |result, b| (result << 8) | *b as u64);
    }
}
