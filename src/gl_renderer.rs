extern crate gl;

pub const VERTEX_SHADER_SOURCE: &str = r#"
#version 330

layout (location = 0) in vec3 a_Pos;
layout (location = 1) in vec2 a_UV;

out vec2 o_UV;

uniform mat4 projection = mat4(1.0);

void main() {
    o_UV = a_UV; 
    gl_Position = projection * vec4(a_Pos, 1.0);
}
"#;

pub const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330

out vec4 FragColor;

in vec2 o_UV;

uniform sampler2D texture1;

void main() {
    FragColor = texture(texture1, o_UV) * vec4(1.0, 1.0, 1.0, 1.0);
}
"#;

pub unsafe fn create_shader(vertex_source: &str, fragment_source: &str) -> u32
{
    let mut success = gl::FALSE as gl::types::GLint;
    let mut info_log = Vec::with_capacity(512);
    info_log.set_len(512 - 1);

    let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
    let c_str_vertex = std::ffi::CString::new(vertex_source.as_bytes()).unwrap();
    gl::ShaderSource(vertex_shader, 1, &c_str_vertex.as_ptr(), std::ptr::null());
    gl::CompileShader(vertex_shader);

    gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
    if !success != gl::TRUE as gl::types::GLint {
        gl::GetShaderInfoLog(vertex_shader, 512, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut gl::types::GLchar); 
        let result = std::str::from_utf8(&info_log);
        if result.is_ok() {
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", std::str::from_utf8(&info_log).unwrap());
        }
    }

    let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
    let c_str_fragment = std::ffi::CString::new(fragment_source.as_bytes()).unwrap();
    gl::ShaderSource(fragment_shader, 1, &c_str_fragment .as_ptr(), std::ptr::null());
    gl::CompileShader(fragment_shader);

    gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
    if !success != gl::TRUE as gl::types::GLint {
        gl::GetShaderInfoLog(fragment_shader, 512, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut gl::types::GLchar); 
        let result = std::str::from_utf8(&info_log);
        if result.is_ok() {
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", std::str::from_utf8(&info_log).unwrap());
        }      
    }

    let shader_program = gl::CreateProgram();
    gl::AttachShader(shader_program, vertex_shader);
    gl::AttachShader(shader_program, fragment_shader);
    gl::LinkProgram(shader_program);
    let mut success = gl::FALSE as gl::types::GLint;
    gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
    let mut info_log = Vec::with_capacity(512);
    info_log.set_len(512 - 1);
    if success != gl::TRUE as gl::types::GLint {
        gl::GetProgramInfoLog(shader_program, 512, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut gl::types::GLchar);
        println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", std::str::from_utf8(&info_log).unwrap());
    }

    gl::DeleteShader(vertex_shader);
    gl::DeleteShader(fragment_shader);

    return shader_program ;
}

pub unsafe fn create_geometry() -> (u32, u32, u32) {
    let vertices: [gl::types::GLfloat; 20] = [
         0.5,  0.5, 0.0,    1.0, 1.0,
         0.5, -0.5, 0.0,    1.0, 0.0,
        -0.5, -0.5, 0.0,    0.0, 0.0,
        -0.5,  0.5, 0.0,    0.0, 1.0
    ];

    let indices: [gl::types::GLuint; 6] = [
        0, 1, 3,
        1, 2, 3
    ];

    let (mut vbo, mut vao, mut ebo) = (0, 0, 0);
    gl::GenVertexArrays(1, &mut vao);
    gl::GenBuffers(1, &mut vbo);
    gl::GenBuffers(1, &mut ebo);

    gl::BindVertexArray(vao);

    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        (vertices.len() * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
        vertices.as_ptr() as *const std::ffi::c_void,
        gl::STATIC_DRAW
    );

    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        (indices.len() * std::mem::size_of::<gl::types::GLuint>()) as gl::types::GLsizeiptr,
        indices.as_ptr() as *const std::ffi::c_void,
        gl::STATIC_DRAW
    );

    let stride = 5 * std::mem::size_of::<gl::types::GLfloat>() as gl::types::GLsizei;
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * std::mem::size_of::<gl::types::GLfloat>()) as *const std::ffi::c_void);
    gl::EnableVertexAttribArray(1);
    
    (vbo, vao, ebo)
}

pub unsafe fn generate_texture() -> u32 {
    let mut texture = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_2D, texture);
    
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    texture
}

pub unsafe fn create_texture(texture: u32, width: i32, height: i32, format: u32, format_type: u32, internal_format: i32, data: *const std::ffi::c_void) {
    gl::BindTexture(gl::TEXTURE_2D, texture);
    gl::TexImage2D(gl::TEXTURE_2D, 0, internal_format, width, height, 0, format, format_type, data);
    gl::GenerateMipmap(gl::TEXTURE_2D);
}
