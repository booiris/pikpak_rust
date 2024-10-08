use anyhow::{Context, Result};
use path_clean::clean;

pub fn slash(path: &str) -> Result<String> {
    let path = clean(path);
    let path = path.to_str().context("[slash] parse path error")?;
    if path.is_empty() {
        return Ok("".to_string());
    }
    if let Some(res) = path.strip_prefix('/') {
        return Ok(res.to_string());
    }
    Ok(path.to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slash() {
        assert_eq!(slash("/").unwrap(), "");
        assert_eq!(slash("/a").unwrap(), "a");
        assert_eq!(slash("a").unwrap(), "a");
        assert_eq!(slash("a/").unwrap(), "a");
        assert_eq!(slash("/a/").unwrap(), "a");
        assert_eq!(slash("/a/b").unwrap(), "a/b");
        assert_eq!(slash("a/b").unwrap(), "a/b");
        assert_eq!(slash("a/b/").unwrap(), "a/b");
        assert_eq!(slash("/a/b/").unwrap(), "a/b");
    }
}
