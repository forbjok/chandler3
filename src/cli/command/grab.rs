use super::*;

pub fn grab(url: &str) -> Result<String, CommandError> {
    println!("URL: {}", url);

    Ok("GRAB".to_string())
}
