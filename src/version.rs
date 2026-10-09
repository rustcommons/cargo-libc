use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl FromStr for Version {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        let parts: Vec<&str> = text.split('.').collect();
        if !(2..=3).contains(&parts.len()) {
            return Err(format!(
                "invalid version {text:?}: expected MAJOR.MINOR or MAJOR.MINOR.PATCH"
            ));
        }
        let number = |part: &str| -> Result<u32, String> {
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return Err(format!("invalid numeric version {text:?}"));
            }
            part.parse()
                .map_err(|_| format!("version component too large in {text:?}"))
        };
        Ok(Self {
            major: number(parts[0])?,
            minor: number(parts[1])?,
            patch: if parts.len() == 3 {
                number(parts[2])?
            } else {
                0
            },
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.patch == 0 {
            write!(f, "{}.{}", self.major, self.minor)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert!("2.9".parse::<Version>().unwrap() < "2.22".parse().unwrap());
        assert!("2.2.5".parse::<Version>().unwrap() < "2.17".parse().unwrap());
        assert_eq!(
            "2.22.0".parse::<Version>().unwrap(),
            "2.22".parse().unwrap()
        );
    }

    #[test]
    fn rejects_non_numeric_and_partial_versions() {
        for value in [
            "",
            "2",
            "2.",
            ".22",
            "2.22.1.4",
            "+2.22",
            "GLIBC_2.22",
            "2.22x",
            "4294967296.22",
        ] {
            assert!(value.parse::<Version>().is_err(), "{value}");
        }
    }
}
