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

        // Essaye de parser un entier
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
