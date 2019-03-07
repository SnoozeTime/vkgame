use shaderc::{Compiler, CompileOptions};
use std::error::Error;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<Error>> {

    let mut file = File::open("assets/shaders/gui.vert")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    let binary_result = compiler.compile_into_spirv(
        content.as_str(), shaderc::ShaderKind::Vertex,
        "shaderrr.glsl", "main", None).unwrap();

    dbg!(binary_result.as_binary_u8());
    
    Ok(())
}
