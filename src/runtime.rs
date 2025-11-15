


pub struct Runtime;
impl Runtime {
    pub fn get_source_code(mod_path: &str) -> String {
        let runtime_path = format!("{}/index.js", mod_path);

        let runtime_content = std::fs::read_to_string(runtime_path).unwrap();

        return runtime_content;
    }
}