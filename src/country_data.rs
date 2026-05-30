use std::collections::HashMap;
use std::sync::OnceLock;

static DATA: &str = include_str!("../data/country_names.json");

pub struct CountryDb {
    data: HashMap<String, HashMap<String, String>>,
}

impl CountryDb {
    pub fn new() -> Self {
        let data: HashMap<String, HashMap<String, String>> =
            serde_json::from_str(DATA).expect("Failed to parse country_names.json");
        CountryDb { data }
    }

    pub fn lookup(&self, lang: &str, code: &str) -> Option<&str> {
        self.data
            .get(lang)
            .and_then(|countries| countries.get(code))
            .map(|s| s.as_str())
    }
}

pub fn db() -> &'static CountryDb {
    static DB: OnceLock<CountryDb> = OnceLock::new();
    DB.get_or_init(CountryDb::new)
}
