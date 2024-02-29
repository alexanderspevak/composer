use regex::Regex;
use std::fmt;

#[derive(Clone)]
pub struct ComposerVersion {
    original: String,
    standardized: (u32, u32, u32),
}

const DEFAULT: &str = "1.0.0";

impl ComposerVersion {
    fn shorten_version(version: &str) -> String {
        let parts: Vec<&str> = version.split('.').collect();
        let sum: u32 = parts[2].parse::<u32>().unwrap() + parts[3].parse::<u32>().unwrap();
        format!("{}.{}.{}", parts[0], parts[1], sum)
    }

    // can pass only regex where is <version> grouping
    fn get_full_version_from_captured_version(original: &str, regex: Regex) -> Option<String> {
        if let Some(captures) = regex.captures(original) {
            let version = &captures["version"];

            let version =
                ComposerVersion::get_standard_version_from_shortened_version(version).unwrap();

            return Some(version.to_string());
        }
        None
    }

    fn clean_from_date(original_version: &str) -> Option<String> {
        let regex_patterns = [
            r"^(?P<date>\d{2}-\d{2}-\d{4})\.(?P<rest>\d+)$",
            r"^(?P<date>\d{4}-\d{2}\.\d{2})-(?P<rest>.+)$",
            r"^(?P<date>\d{4}-(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)-\d{2})-(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}-(0?[1-9]|1[0-2])-(0?[1-9]|[12]\d|3[01]))\.(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}\.(0?[1-9]|1[0-2])\.(0?[1-9]|[12]\d|3[01]))\.(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}\.(0?[1-9]|1[0-2])\.(0?[1-9]|[12]\d|3[01]))-(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}\.(0?[1-9]|[12]\d|3[01])\.(0?[1-9]|1[0-2]))-(?P<rest>.+)$",
            r#"^(?P<date>\d{4}\.\d{2}-\d{2})-(?P<rest>\d+)$"#,
            r#"^(?P<date>\d{4}-\d{2}-\d{2})_(?P<rest>\d+)$"#,
            r"^(?P<date>(?:19|20)\d{2}-(?:0?[1-9]|1[0-2])-(?:0?[1-9]|[12]\d|3[01]))-(?P<rest>\w+|\d+)$",
        ];

        regex_patterns.iter().find_map(|pattern| {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(original_version) {
                return Some(
                    captures
                        .name("rest")
                        .map(|m| m.as_str().to_owned())
                        .unwrap_or("1.0.0".to_string()),
                );
            }

            None
        })
    }

    fn get_standard_version_from_shortened_version(original: &str) -> Option<String> {
        let major_version_regex = Regex::new(r#"^\d+$"#).unwrap();

        if major_version_regex.captures(original).is_some() {
            return Some(format!("{original}.0.0"));
        }

        let major_minor_version_regex = Regex::new(r#"^\d+\.\d+$"#).unwrap();
        if major_minor_version_regex.captures(original).is_some() {
            return Some(format!("{original}.0"));
        }

        let major_minor_patch_version_regex = Regex::new(r#"^\d+\.\d+\.\d+$"#).unwrap();
        if major_minor_patch_version_regex.captures(original).is_some() {
            return Some(original.to_owned());
        }

        None
    }

    fn get_standardized_version(original: &str) -> String {
        let mut original_without_start_v = original;
        if let Some(stripped) = original.strip_prefix('v') {
            original_without_start_v = stripped;
        }

        if let Some(stripped) = original.strip_prefix('V') {
            original_without_start_v = stripped;
        }

        let just_dotted_date_version =
            Regex::new(r#"^(?:19\d{2}|20\d{2})\.\d{2}\.\d{2}$"#).unwrap();

        if just_dotted_date_version.is_match(original) {
            return String::from(DEFAULT);
        }

        let dashed_version = Regex::new(r#"v(\d+)"#).unwrap();

        if let Some(captures) = dashed_version.captures(original_without_start_v) {
            if let Some(version) =
                ComposerVersion::get_standard_version_from_shortened_version(&captures[1])
            {
                return version;
            }
        }

        let leading_zeros_version = Regex::new(r#"^0*(?P<version>\d+)$"#).unwrap();

        if let Some(version) = ComposerVersion::get_full_version_from_captured_version(
            original_without_start_v,
            leading_zeros_version,
        ) {
            return version;
        }

        if let Some(version) =
            ComposerVersion::get_standard_version_from_shortened_version(original_without_start_v)
        {
            return version;
        }

        let starts_with_full_version =
            Regex::new(r#"^(?P<version>\d+\.\d+\.\d+)-[\w.-]+$"#).unwrap();

        if let Some(captures) = starts_with_full_version.captures(original_without_start_v) {
            let version = &captures["version"];
            return version.to_string();
        }

        let starts_with_version = Regex::new(r"^(?P<version>\d+)(?:-(?P<meta>.*))?$").unwrap();
        if let Some(version) = ComposerVersion::get_full_version_from_captured_version(
            original_without_start_v,
            starts_with_version,
        ) {
            return version.to_string();
        }

        let ends_with_version = Regex::new(r#"^(?P<meta>.+)-v(?P<version>\d+)$"#).unwrap();
        if let Some(version) = ComposerVersion::get_full_version_from_captured_version(
            original_without_start_v,
            ends_with_version,
        ) {
            return version.to_string();
        }

        let long_version = Regex::new(r#"^(\d+)\.(\d+)\.(\d+)\.(\d+)$"#).unwrap();

        if long_version.is_match(original_without_start_v) {
            return ComposerVersion::shorten_version(original_without_start_v);
        }

        String::from(DEFAULT)
    }

    fn standardize(original_version: &str) -> String {
        let cleaned = ComposerVersion::clean_from_date(original_version);
        match cleaned {
            Some(cleaned) => ComposerVersion::get_standardized_version(&cleaned),
            None => ComposerVersion::get_standardized_version(original_version),
        }
    }

    pub fn new(original: &str) -> Self {
        let standardized = ComposerVersion::standardize(original);
        let nums = standardized
            .split('.')
            .map(|v| v.parse::<u32>().unwrap())
            .collect::<Vec<u32>>();

        ComposerVersion {
            original: original.to_string(),
            standardized: (nums[0], nums[1], nums[2]),
        }
    }

    pub fn get_original(&self) -> String {
        self.original.to_string()
    }

    pub fn bump_major(&mut self) {
        let (major, _, _) = self.standardized;
        self.standardized = (major + 1, 0, 0);
    }
    pub fn bump_minor(&mut self) {
        let (major, minor, _) = self.standardized;
        self.standardized = (major, minor + 1, 0);
    }
    pub fn bump_patch(&mut self) {
        let (major, minor, patch) = self.standardized;
        self.standardized = (major, minor, patch + 1);
    }
}

impl fmt::Display for ComposerVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (major, minor, patch) = self.standardized;
        write!(f, "{}.{}.{}", major, minor, patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod standardize {
        use super::*;
        #[test]
        fn test_dash_date_with_dot_separator() {
            let tested = "2021-11-24.3";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }

        #[test]
        fn test_dot_date_with_dot_separator() {
            let tested = "2023.04.25.3";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }

        #[test]
        fn test_version_with_sufix() {
            //this probably is date typo, but how can we handle cases like this, there is no v marking?
            let tested = "20211.11.24-1";
            let version = ComposerVersion::new(tested);

            assert_eq!("20211.11.24", version.to_string());
        }

        #[test]
        fn test_mixed_dash_dot_date_with_dash_separator() {
            let tested = "2022.03-01-2";
            let version = ComposerVersion::new(tested);

            assert_eq!("2.0.0", version.to_string());
        }

        #[test]
        fn test_just_doted_date() {
            //is it version, or is it date?? who knows
            let tested = "2023.05.03";
            let version = ComposerVersion::new(tested);

            assert_eq!(DEFAULT, version.to_string());
        }

        #[test]
        fn test_dash_date_with_dash_separator() {
            let tested = "2022-01-24-3";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }
        #[test]
        fn test_dash_date_with_dash_separator_and_v_prefix() {
            let tested = "2023-01-19-v3";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }
        #[test]
        fn test_dash_date_with_dash_separator_and_v_prefix_with_name_month() {
            let tested = "2023-Nov-02-v3";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }
        #[test]
        fn test_dash_date_with_dash_separator_and_meta_as_prefix() {
            // we can not deduce any version from this so we expect default 1.0.0
            let tested = "release-2022-01-25";
            let version = ComposerVersion::new(tested);

            assert_eq!(DEFAULT, version.to_string());
        }

        #[test]
        fn test_dot_date_with_dash_separator_and_meta() {
            let tested = "2023.10.25-V3-Revert-MV-G9-HF-SG-Red-V2";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }

        #[test]
        fn test_dot_date_with_dash_separator() {
            let tested = "2023.09.05-2";
            let version = ComposerVersion::new(tested);

            assert_eq!("2.0.0", version.to_string());
        }

        #[test]
        fn test_dot_date_with_dash_separator_yyyy_dd_mm() {
            let tested = "2023.22.03-2";
            let version = ComposerVersion::new(tested);

            assert_eq!("2.0.0", version.to_string());
        }

        #[test]
        fn test_dot_date_with_dash_separator_and_leading_0() {
            let tested = "2021.09.30-03";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }

        #[test]
        fn test_dot_date_with_dash_separator_and_leading_00() {
            let tested = "2021-07-15_003";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }

        #[test]
        fn test_valid() {
            let tested = "0.2.0";
            let version = ComposerVersion::new(tested);

            assert_eq!("0.2.0", version.to_string());
        }

        #[test]
        fn test_valid_with_meta() {
            let tested = "0.2.0-alpha.6";
            let version = ComposerVersion::new(tested);

            assert_eq!("0.2.0", version.to_string());
        }

        #[test]
        fn test_longer_than_standard() {
            let tested = "1.3.3.1";
            let version = ComposerVersion::new(tested);

            assert_eq!("1.3.4", version.to_string());
        }
        #[test]
        fn test_invalid_date_with_v_marking() {
            let tested = "2023-023-29-v3";
            let version = ComposerVersion::new(tested);

            assert_eq!("3.0.0", version.to_string());
        }
    }
    #[cfg(test)]
    mod bump {
        use crate::ComposerVersion;

        #[test]
        fn test_minor_bump() {
            let mut version = ComposerVersion::new("v1.0.1");
            version.bump_minor();
            assert_eq!("1.1.0", version.to_string());
        }
        #[test]
        fn test_patch_bump() {
            let mut version = ComposerVersion::new("v1.0.1");
            version.bump_patch();
            assert_eq!("1.0.2", version.to_string());
        }
        #[test]
        fn test_major_bump() {
            let mut version = ComposerVersion::new("v1.13.1");
            version.bump_major();
            assert_eq!("2.0.0", version.to_string());
        }
    }
}
