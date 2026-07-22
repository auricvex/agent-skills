// simple-crate: A minimal Rust library for demonstrating SDD workflow.
// Conventions:
// - All public APIs must have doc comments.
// - Error handling uses thiserror (add it when needed).
// - Tests go in the same file as the code they test (inline tests).

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
