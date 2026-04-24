//! macOS single-modifier push-to-talk monitor backed by CGEventTap.
//!
//! Replaces `fn_monitor.rs`. See docs/feature-single-key-hotkey.md.

#![cfg(target_os = "macos")]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingleKey {
    Fn,
    LeftOption,
    RightOption,
    LeftCommand,
    RightCommand,
    LeftControl,
    RightControl,
    LeftShift,
    RightShift,
}

impl SingleKey {
    /// Parse a raw config string. Accepts `"Fn"`, `"RightOption"`,
    /// `"left_shift"`, `"right-control"`, etc. Case-insensitive.
    pub fn from_raw(raw: &str) -> Option<Self> {
        let normalized: String = raw
            .trim()
            .chars()
            .filter(|c| !matches!(*c, '_' | '-' | ' '))
            .flat_map(|c| c.to_lowercase())
            .collect();
        match normalized.as_str() {
            "fn" => Some(Self::Fn),
            "leftoption" => Some(Self::LeftOption),
            "rightoption" => Some(Self::RightOption),
            "leftcommand" => Some(Self::LeftCommand),
            "rightcommand" => Some(Self::RightCommand),
            "leftcontrol" => Some(Self::LeftControl),
            "rightcontrol" => Some(Self::RightControl),
            "leftshift" => Some(Self::LeftShift),
            "rightshift" => Some(Self::RightShift),
            _ => None,
        }
    }

    /// macOS virtual key code emitted by the physical modifier key.
    pub fn key_code(self) -> i64 {
        match self {
            Self::Fn => 63,
            Self::LeftOption => 58,
            Self::RightOption => 61,
            Self::LeftCommand => 55,
            Self::RightCommand => 54,
            Self::LeftControl => 59,
            Self::RightControl => 62,
            Self::LeftShift => 56,
            Self::RightShift => 60,
        }
    }

    /// Bit in `CGEventGetFlags` that indicates this specific key is held.
    /// Fn uses the device-independent `NSEventModifierFlagFunction`; left/right
    /// split keys use the private `NX_DEVICE*KEYMASK` bits in the low nibble.
    pub fn device_mask(self) -> u64 {
        match self {
            Self::Fn => 1 << 23,            // NSEventModifierFlagFunction
            Self::LeftControl => 0x0000_0001,
            Self::LeftShift => 0x0000_0002,
            Self::RightShift => 0x0000_0004,
            Self::LeftCommand => 0x0000_0008,
            Self::RightCommand => 0x0000_0010,
            Self::LeftOption => 0x0000_0020,
            Self::RightOption => 0x0000_0040,
            Self::RightControl => 0x0000_2000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw_accepts_canonical_names() {
        assert_eq!(SingleKey::from_raw("Fn"), Some(SingleKey::Fn));
        assert_eq!(SingleKey::from_raw("RightOption"), Some(SingleKey::RightOption));
        assert_eq!(SingleKey::from_raw("LeftOption"), Some(SingleKey::LeftOption));
        assert_eq!(SingleKey::from_raw("RightCommand"), Some(SingleKey::RightCommand));
        assert_eq!(SingleKey::from_raw("LeftCommand"), Some(SingleKey::LeftCommand));
        assert_eq!(SingleKey::from_raw("RightControl"), Some(SingleKey::RightControl));
        assert_eq!(SingleKey::from_raw("LeftControl"), Some(SingleKey::LeftControl));
        assert_eq!(SingleKey::from_raw("RightShift"), Some(SingleKey::RightShift));
        assert_eq!(SingleKey::from_raw("LeftShift"), Some(SingleKey::LeftShift));
    }

    #[test]
    fn from_raw_is_case_and_separator_insensitive() {
        assert_eq!(SingleKey::from_raw("fn"), Some(SingleKey::Fn));
        assert_eq!(SingleKey::from_raw("right_option"), Some(SingleKey::RightOption));
        assert_eq!(SingleKey::from_raw("right-option"), Some(SingleKey::RightOption));
        assert_eq!(SingleKey::from_raw("  RIGHT OPTION "), Some(SingleKey::RightOption));
    }

    #[test]
    fn from_raw_rejects_combos_and_unknown() {
        assert_eq!(SingleKey::from_raw("Option+Space"), None);
        assert_eq!(SingleKey::from_raw(""), None);
        assert_eq!(SingleKey::from_raw("Space"), None);
        assert_eq!(SingleKey::from_raw("RightAlt"), None); // we use "Option"
    }

    #[test]
    fn key_codes_are_distinct() {
        let all = [
            SingleKey::Fn,
            SingleKey::LeftOption, SingleKey::RightOption,
            SingleKey::LeftCommand, SingleKey::RightCommand,
            SingleKey::LeftControl, SingleKey::RightControl,
            SingleKey::LeftShift, SingleKey::RightShift,
        ];
        let mut codes: Vec<i64> = all.iter().map(|k| k.key_code()).collect();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), all.len(), "every SingleKey must have a unique key_code");
    }

    #[test]
    fn device_masks_cover_expected_bits() {
        assert_eq!(SingleKey::Fn.device_mask(), 1 << 23);
        assert_eq!(SingleKey::RightOption.device_mask(), 0x40);
        assert_eq!(SingleKey::LeftCommand.device_mask(), 0x08);
        assert_eq!(SingleKey::RightControl.device_mask(), 0x2000);
    }
}
