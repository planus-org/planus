#[macro_use]
pub mod macros;

pub mod planus;
pub mod planus_api;
pub mod planus_test;

pub mod hexdump;
mod unit_tests;

#[cfg(test)]
pub mod tests {
    use anyhow::Result;
    use std::path::Path;

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
