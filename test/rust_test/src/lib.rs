#[macro_use]
pub mod macros;

pub mod flatc;
pub mod planus;
pub mod planus_api;
pub mod planus_test;

mod conformance;
pub mod hexdump;
mod unit_tests;

#[cfg(test)]
pub mod tests {
    use anyhow::Result;
    use std::path::Path;

    pub fn compare_regenerate_file<P, S, D, R>(
        path: P,
        new_val: R,
        serialize: S,
        deserialize: D,
        should_regenerate: bool,
    ) -> Result<()>
    where
        P: AsRef<Path>,
        D: Fn(Vec<u8>) -> R,
        S: Fn(R) -> Vec<u8>,
        R: PartialEq + std::fmt::Debug,
    {
        let path = path.as_ref();

        if path.exists() && !should_regenerate {
            let data = std::fs::read(path)?;
            let prev_val = deserialize(data);
            similar_asserts::assert_eq!(prev_val, new_val);
        } else {
            let data = serialize(new_val);
            std::fs::write(path, &data)?;
        }

        Ok(())
    }
}
