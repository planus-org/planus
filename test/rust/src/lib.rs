#![recursion_limit = "512"]

#[macro_use]
pub mod macros;

pub mod planus_api;
pub mod planus_test;
pub mod planus_test_no_flatc;

pub mod hexdump;

mod table_reader;

#[cfg(test)]
pub mod tests {
    use std::path::Path;

    use anyhow::Result;

    pub fn compare_regenerate_file(
        path: impl AsRef<Path>,
        new_val: &[u8],
        should_regenerate: bool,
    ) -> Result<()> {
        let path = path.as_ref();

        if path.exists() && !should_regenerate {
            let data = std::fs::read(path)?;
            similar_asserts::assert_eq!(data, new_val);
        } else {
            std::fs::write(path, new_val)?;
        }

        Ok(())
    }

    pub fn compare_regenerate_file_str(
        path: impl AsRef<Path>,
        new_val: &str,
        should_regenerate: bool,
    ) -> Result<()> {
        let path = path.as_ref();

        if path.exists() && !should_regenerate {
            let data = std::fs::read_to_string(path)?;
            similar_asserts::assert_str_eq!(data, new_val);
        } else {
            std::fs::write(path, &new_val)?;
        }

        Ok(())
    }
}
