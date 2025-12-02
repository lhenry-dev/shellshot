use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Dimension {
    Auto,
    Value(u16),
}

impl FromStr for Dimension {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("auto") {
            return Ok(Dimension::Auto);
        }

        s.parse::<u16>().map(Dimension::Value).map_err(|_| {
            format!("Invalid dimension value: {s}. Must be 'auto' or a positive integer")
        })
    }
}

impl Dimension {
    pub fn to_u16(&self, default: u16) -> u16 {
        match self {
            Dimension::Value(v) => *v,
            Dimension::Auto => default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_from_str_auto() {
        let dim = Dimension::from_str("auto").unwrap();
        matches!(dim, Dimension::Auto);
    }

    #[test]
    fn test_from_str_numeric() {
        let dim = Dimension::from_str("42").unwrap();
        match dim {
            Dimension::Value(v) => assert_eq!(v, 42),
            _ => panic!("Expected Dimension::Value"),
        }
    }

    #[test]
    fn test_from_str_invalid() {
        let err = Dimension::from_str("abc").unwrap_err();
        assert!(err.contains("Invalid dimension value"));
    }

    #[test]
    fn test_to_u16_value() {
        let dim = Dimension::Value(50);
        assert_eq!(dim.to_u16(100), 50);
    }

    #[test]
    fn test_to_u16_auto() {
        let dim = Dimension::Auto;
        assert_eq!(dim.to_u16(80), 80);
    }
}
