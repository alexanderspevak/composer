use regex::Regex;
use std::fmt;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Clone)]
pub struct ComposerVersion {
    original: String,
    standardized: (u32, u32, u32),
}

impl ComposerVersion {
    fn shorten_version(version: &str) -> String {
        let parts: Vec<&str> = version.split('.').collect();
        let sum: u32 = parts[2].parse::<u32>().unwrap() + parts[3].parse::<u32>().unwrap();
        format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], sum)
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

    //I do not care about standardizing the date, as it is not defined in the task
    fn clean_from_date(original_version: &str) -> Option<String> {
        let regex_patterns = [
            r"^(?P<date>\d{2}-\d{2}-\d{4})\.(?P<rest>\d+)$",
            r"^(?P<date>\d{4}-\d{2}\.\d{2})-(?P<rest>.+)$",
            r"^(?P<rest>\w+)-(?P<date>\d{4}-\d{2}-\d{2})$",
            r"^(?P<date>\d{4}-(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)-\d{2})-(?P<rest>.+)$",
            r"^(?P<date>(0?[1-9]|1[0-2])-(0?[1-9]|[12]\d|3[01])-(19|20)\d{2})\.(?P<version>\d+)$",
            r"^(?P<date>(19|20)\d{2}-(0?[1-9]|1[0-2])-(0?[1-9]|[12]\d|3[01]))\.(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}-(0?[1-9]|[12]\d|3[01])-(0?[1-9]|1[0-2]))\.(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}\.(0?[1-9]|1[0-2])\.(0?[1-9]|[12]\d|3[01]))\.(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}\.(0?[1-9]|[12]\d|3[01])\.(0?[1-9]|1[0-2]))\.(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}-(0?[1-9]|1[0-2])-(0?[1-9]|[12]\d|3[01]))\.(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}-(0?[1-9]|[12]\d|3[01])-(0?[1-9]|1[0-2]))\.(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}\.(0?[1-9]|1[0-2])\.(0?[1-9]|[12]\d|3[01]))-(?P<rest>.+)$",
            r"^(?P<date>(19|20)\d{2}\.(0?[1-9]|[12]\d|3[01])\.(0?[1-9]|1[0-2]))-(?P<rest>.+)$",
            r#"^(?P<date>\d{4}\.\d{2}-\d{2})-(?P<rest>\d+)$"#,
            r"^(?P<date>(19|20)\d{2}-(0?[1-9]|1[0-2])-(0?[1-9]|[12]\d|3[01]))-(?P<rest>\d+)$",
            r"(?P<date>(19|20)\d{2}\.\d{2}\.\d{2})",
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

        String::from("100.0.0")
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
