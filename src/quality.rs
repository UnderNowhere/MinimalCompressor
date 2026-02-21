use std::fmt;

/// All available quality presets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quality {
    Low,    // Screen
    Middle, // Ebook
    Good,   // Printer
    High,   // Prepress
}

impl Quality {
    /// All available quality presets.
    pub const ALL: &'static [Self] = &[Self::Low, Self::Middle, Self::Good, Self::High];

    pub fn as_gs_pdfsettings(&self) -> String {
        match self {
            Quality::Low => String::from("screen"),
            Quality::Middle => String::from("ebook"),
            Quality::Good => String::from("printer"),
            Quality::High => String::from("prepress"),
        }
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Quality::Low => write!(f, "Low"),
            Quality::Middle => write!(f, "Middle"),
            Quality::Good => write!(f, "Good"),
            Quality::High => write!(f, "High"),
        }
    }
}
