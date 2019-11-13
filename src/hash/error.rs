//From Ring Crate


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unspecified;

impl Unspecified {
    fn description_() -> &'static str {
        "mohan::error::Unspecified"
    }
}

// This is required for the implementation of `std::error::Error`.
impl core::fmt::Display for Unspecified {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(Self::description_())
    }
}