use crate::{byte_stream::ByteStream, globals::Globals};



pub struct LunaCompile;

impl LunaCompile {
    pub fn compile(header_data: &crate::header::HeaderData, source_code: String) -> ByteStream {
        let data = ByteStream::new();
        let header = Self::prepare_header(data, &header_data);

        return Self::bake_source_code(header, source_code);

    }
    fn prepare_header(data: ByteStream, header_data: &crate::header::HeaderData) -> ByteStream {
        let mut data = data;
        data.write_vint(Globals::LUNA_VERSION);
        
        data.write_string(&header_data.id);
        data.write_vint(header_data.version);
        data.write_string(&header_data.version_name);

        data.write_string(&header_data.display_name.clone().unwrap_or("null".to_string()));
        data.write_string(&header_data.description.clone().unwrap_or("null".to_string()));
        data.write_string(&header_data.author.clone().unwrap_or("null".to_string()));

        return data;
    }

    fn bake_source_code(mut data: ByteStream, source_code: String) -> ByteStream {
        data.write_string(&source_code);
        return data;
    }
}