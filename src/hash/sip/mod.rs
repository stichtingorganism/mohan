// Copyright 2019 Stichting Organism
// Copyright 2012-2016 The Rust Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.

mod sipper;
pub use sipper::SipHasher24;

#[cfg(test)]
mod test_sip;
