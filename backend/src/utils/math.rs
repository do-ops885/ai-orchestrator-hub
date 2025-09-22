//! Mathematical utilities for the AI Orchestrator Hub

/// Calculates the factorial of a non-negative integer.
///
/// Factorial is defined as:
/// - 0! = 1
/// - n! = n * (n-1) * ... * 1 for n > 0
///
/// # Arguments
/// * `n` - A non-negative integer (u64)
///
/// # Returns
/// * `Ok(u128)` - The factorial value if successful
/// * `Err(String)` - Error message if overflow occurs (n > 34 for u128)
///
/// # Examples
/// ```
/// use ai_orchestrator_hub::utils::math::factorial;
///
/// let result = factorial(5);
/// assert_eq!(result, Ok(120));
///
/// let zero = factorial(0);
/// assert_eq!(zero, Ok(1));
/// ```
pub fn factorial(n: u64) -> Result<u128, String> {
    if n == 0 || n == 1 {
        return Ok(1);
    }
    let mut result: u128 = 1;
    for i in 1..=n {
        match result.checked_mul(u128::from(i)) {
            Some(val) => result = val,
            None => return Err(format!("Factorial of {n} would overflow u128")),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial_zero() {
        assert_eq!(factorial(0), Ok(1));
    }

    #[test]
    fn test_factorial_one() {
        assert_eq!(factorial(1), Ok(1));
    }

    #[test]
    fn test_factorial_small() {
        assert_eq!(factorial(5), Ok(120));
        assert_eq!(factorial(10), Ok(3_628_800));
    }

    #[test]
    fn test_factorial_large() {
        assert_eq!(factorial(20), Ok(2_432_902_008_176_640_000));
    }

    #[test]
    fn test_factorial_overflow() {
        assert!(factorial(35).is_err());
    }
}
