use std::collections::HashMap;
use std::sync::RwLock;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    PtBr,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::PtBr => "pt_BR",
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::En => "English",
            Language::PtBr => "Português (BR)",
        }
    }

    pub fn detect_system() -> Self {
        for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(val) = std::env::var(var) {
                let lower = val.to_lowercase();
                if lower.starts_with("pt") {
                    return Language::PtBr;
                } else if lower.starts_with("en") {
                    return Language::En;
                }
            }
        }
        Language::En
    }
}

pub struct I18n {
    current_lang: Language,
    translations: HashMap<String, HashMap<String, String>>,
}

static INSTANCE: std::sync::OnceLock<RwLock<I18n>> = std::sync::OnceLock::new();

impl I18n {
    pub fn init() {
        let lang = Language::detect_system();
        let mut translations = HashMap::new();

        let en_json = include_str!("../../i18n/en.json");
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(en_json) {
            translations.insert("en".to_string(), map);
        }

        let pt_json = include_str!("../../i18n/pt_BR.json");
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(pt_json) {
            translations.insert("pt_BR".to_string(), map);
        }

        info!("i18n initialized with system language: {:?}", lang);

        let i18n = I18n {
            current_lang: lang,
            translations,
        };

        let _ = INSTANCE.set(RwLock::new(i18n));
    }

    #[allow(dead_code)]
    pub fn set_language(lang: Language) {
        if let Some(lock) = INSTANCE.get() {
            if let Ok(mut guard) = lock.write() {
                guard.current_lang = lang;
            }
        }
    }

    #[allow(dead_code)]
    pub fn current_language() -> Language {
        INSTANCE
            .get()
            .and_then(|l| l.read().ok())
            .map(|g| g.current_lang)
            .unwrap_or(Language::En)
    }

    pub fn get(key: &str) -> String {
        if let Some(lock) = INSTANCE.get() {
            if let Ok(guard) = lock.read() {
                let lang_code = guard.current_lang.code();
                if let Some(map) = guard.translations.get(lang_code) {
                    if let Some(val) = map.get(key) {
                        return val.clone();
                    }
                }
                // Fallback to English
                if let Some(map) = guard.translations.get("en") {
                    if let Some(val) = map.get(key) {
                        return val.clone();
                    }
                }
            }
        }
        key.to_string()
    }
}

pub fn t(key: &str) -> String {
    I18n::get(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_i18n_translations() {
        I18n::init();
        I18n::set_language(Language::En);
        assert_eq!(t("app_title"), "Desktop Manager");
        assert_eq!(t("new_entry"), "New Entry");

        I18n::set_language(Language::PtBr);
        assert_eq!(t("app_title"), "Gerenciador de Desktop");
        assert_eq!(t("new_entry"), "Nova Entrada");
    }

    #[test]
    fn test_all_keys_in_sync() {
        let en: HashMap<String, String> =
            serde_json::from_str(include_str!("../../i18n/en.json")).unwrap();
        let pt: HashMap<String, String> =
            serde_json::from_str(include_str!("../../i18n/pt_BR.json")).unwrap();

        for key in en.keys() {
            assert!(
                pt.contains_key(key),
                "Missing translation in pt_BR.json for key: '{key}'"
            );
        }

        for key in pt.keys() {
            assert!(
                en.contains_key(key),
                "Missing translation in en.json for key: '{key}'"
            );
        }
    }
}
