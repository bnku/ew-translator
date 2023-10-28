use super::settings::TARGET_LANG;
use urlencoding::encode;

extern crate reqwest;

#[tokio::main]
pub async fn google(phrase: String) -> Result<String, reqwest::Error> {
    let url = format!(
        "{}{}{}{}",
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl=",
        *TARGET_LANG.read().unwrap(),
        "&dt=t&q=",
        encode(&phrase)
    );

    let json = reqwest::get(url).await?.text().await?;
    // println!("{}",String::from(&json));

    let mut result = String::from("");
    for line in gjson::get(&json, "0.#.0").array() {
        result.push_str(line.str());
    }
    Ok(result)
}
