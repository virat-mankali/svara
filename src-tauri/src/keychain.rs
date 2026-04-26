use keyring::Entry;

const SERVICE: &str = "svara";
const GROQ_KEY: &str = "groq_api_key";

pub fn save_api_key(key: &str) -> anyhow::Result<()> {
    Entry::new(SERVICE, GROQ_KEY)?.set_password(key)?;
    Ok(())
}

pub fn get_api_key() -> anyhow::Result<String> {
    Ok(Entry::new(SERVICE, GROQ_KEY)?.get_password()?)
}
