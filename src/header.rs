#[derive(serde::Deserialize)]
pub struct HeaderData {
    pub id: String,
    pub version: i32,
    pub version_name: String,

    // Optional fields

    pub display_name: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
}

pub struct Header;
impl Header {
    pub fn read_json(mod_path: &str) -> HeaderData {
        let header_path = format!("{}/header.json", mod_path);

        let header_content = std::fs::read_to_string(header_path).unwrap();
        let header_content: HeaderData = serde_json::from_str(&header_content).unwrap();

        return header_content;
    }
}