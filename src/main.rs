extern crate glfw;
extern crate cgmath;
extern crate gl;
extern crate image;
use cgmath::Matrix;
use ffmpeg_sys_next::{avformat_open_input, avformat_find_stream_info};
use glfw::{Action, Context};

const VERTEX_SHADER_SOURCE: &str = r#"
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

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 330

out vec4 FragColor;

in vec2 o_UV;

uniform sampler2D texture1;

void main() {
    FragColor = texture(texture1, o_UV) * vec4(1.0, 1.0, 1.0, 1.0);
}
"#;

unsafe fn create_shader(vertex_source: &str, fragment_source: &str) -> u32
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

fn main() {
    let mut width: i32 = 800;
    let mut height: i32 = 600;
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw.create_window(width as u32, height as u32, "RustyVideoPlayer", glfw::WindowMode::Windowed)
    .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader_program, vao, vbo, ebo, texture) = unsafe {
        let shader_program = create_shader(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        
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
        
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        // let img = image::open(& std::path::Path::new("res/example.jpg")).expect("Failed to load texture"); 

        let mut format_context = ffmpeg_sys_next::avformat_alloc_context();
        assert!(!format_context.is_null(), "ERROR could not allocate memory for Format Context");

        if avformat_open_input(&mut format_context, std::ffi::CString::new("res/Top5polishperson.mp4").unwrap().as_ptr(), std::ptr::null_mut(), std::ptr::null_mut()) != 0 {
            assert!(false, "ERROR could not open the file");
        }
        println!("Format {}, duration {} us", std::ffi::CStr::from_ptr((*(*format_context).iformat).long_name).to_str().unwrap(), (*format_context).duration);


        let mut video_codec_parameters: *mut ffmpeg_sys_next::AVCodecParameters = std::ptr::null_mut();
        let mut video_codec: *mut ffmpeg_sys_next::AVCodec = std::ptr::null_mut();
        let mut video_stream_index: i32 = -1;

        for i in 0..(*format_context).nb_streams {
            let local_codec_parameters = (*(*(*format_context).streams).offset(i as isize)).codecpar;
            let local_codec = ffmpeg_sys_next::avcodec_find_decoder((*local_codec_parameters).codec_id);
            if local_codec.is_null() {
                println!("ERROR unsupported codec!");
                continue;
            }

            if (*local_codec_parameters).codec_type == ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_VIDEO {
                video_stream_index = i as i32;
                video_codec_parameters = local_codec_parameters;
                video_codec = local_codec;
                println!("Video resolution: {} x {}", (*local_codec_parameters).width, (*local_codec_parameters).height);
                break; 
            }
            else if (*local_codec_parameters).codec_type == ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_AUDIO {
                println!("Audio channels: {}, sample rate: {}", (*local_codec_parameters).channels, (*local_codec_parameters).sample_rate); 
            }
            println!("\tID: {:?}, bitrate: {}", (*local_codec).id, (*local_codec_parameters).bit_rate);
        }

        assert!(video_stream_index != -1, "File does not contain a video stream!");

        let mut codec_context = ffmpeg_sys_next::avcodec_alloc_context3(video_codec);
        assert!(!codec_context.is_null(), "Failed to allocate memory for AVCodecContext");

        if ffmpeg_sys_next::avcodec_parameters_to_context(codec_context, video_codec_parameters) < 0 {
            assert!(false, "failed to copy codec params to codec context");
        }

        if ffmpeg_sys_next::avcodec_open2(codec_context, video_codec, std::ptr::null_mut()) < 0 {
            assert!(false, "failed to open codec through avcodec_open2");
        }

        let mut frame = ffmpeg_sys_next::av_frame_alloc();
        assert!(!frame.is_null(), "failed to allocate memory for AVFrame");
        let packet = ffmpeg_sys_next::av_packet_alloc();
        assert!(!packet.is_null(), "failed to allocate memory for AVPacket");

        let mut response = 0;

        while ffmpeg_sys_next::av_read_frame(format_context, packet) >= 0 {
            if (*packet).stream_index != video_stream_index {
                continue;
            }

            response = ffmpeg_sys_next::avcodec_send_packet(codec_context, packet);
            if response < 0 {
                println!("Failed to decode packet");
                break;
            }

            response = ffmpeg_sys_next::avcodec_receive_frame(codec_context, frame);
            if (response == ffmpeg_sys_next::AVERROR(ffmpeg_sys_next::EAGAIN)) || (response == ffmpeg_sys_next::AVERROR_EOF) {
                continue;
            }
            else if response < 0 {
                println!("Failed to decode packet");
                break;
            }

                
            ffmpeg_sys_next::av_packet_unref(packet);
            break;
        }

        let mut data: Vec<u8>  = Vec::with_capacity(((*frame).width * (*frame).height * 4) as usize);

        let sws_scale_ctx = ffmpeg_sys_next::sws_getContext(
            (*frame).width,
            (*frame).height,
            (*codec_context).pix_fmt,
            (*frame).width,
            (*frame).height,
            ffmpeg_sys_next::AVPixelFormat::AV_PIX_FMT_RGB0,
            ffmpeg_sys_next::SWS_BILINEAR,
            std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()
        );

        assert!(!sws_scale_ctx.is_null(), "Couldn't initialize sw scaler");

        let dest = [data.as_mut_ptr(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()];
        let dest_linesize = [(*frame).width * 4, 0, 0, 0]; 
        ffmpeg_sys_next::sws_scale(
            sws_scale_ctx,
            (*frame).data.as_ptr() as *const *const u8,
            (*frame).linesize.as_ptr(),
            0,
            (*frame).height,
            dest.as_ptr(),
            dest_linesize.as_ptr()
        );

        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, (*frame).width, (*frame).height, 0, gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr() as *const std::ffi::c_void); 

        ffmpeg_sys_next::sws_freeContext(sws_scale_ctx);
        ffmpeg_sys_next::avformat_close_input(&mut format_context);
        ffmpeg_sys_next::avformat_free_context(format_context);
        ffmpeg_sys_next::avcodec_free_context(&mut codec_context);
        ffmpeg_sys_next::av_free_packet(packet);
        ffmpeg_sys_next::av_frame_free(&mut frame);

        // gl::GenerateMipmap(gl::TEXTURE_2D);

        (shader_program, vao, vbo, ebo, texture)
    };
    
    let uniform_name = std::ffi::CString::new("projection").unwrap();

    while !window.should_close() {
        handle_window_event(&mut width, &mut height, &mut window, &events);     
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 0.1);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);

            let mut projection_matrix = cgmath::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
            let mut model = cgmath::Matrix4::from_translation(cgmath::vec3(0.0, 0.0, 0.0));
            model = model * cgmath::Matrix4::from_translation(cgmath::vec3(0.5 * (width as f32), 0.5 * (height as f32), 0.0));
            model = model * cgmath::Matrix4::from_nonuniform_scale(1.0 * width as f32, height as f32 , 1.0);
            projection_matrix = projection_matrix * model;

            let vertex_proyection_location = gl::GetUniformLocation(shader_program, uniform_name.as_ptr());
            gl::UniformMatrix4fv(vertex_proyection_location, 1, gl::FALSE, projection_matrix.as_ptr());

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }

        window.swap_buffers();
        glfw.poll_events();
    }

    unsafe {
        gl::DeleteProgram(shader_program);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ebo);
        gl::DeleteTextures(1, &texture);
    }
}

fn handle_window_event(width_win: &mut i32, height_win: &mut i32, window: &mut glfw::Window, events: &std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(&events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe {
                    *width_win = width;
                    *height_win = height;
                    gl::Viewport(0, 0, width, height);
                }
            }
            glfw::WindowEvent::Key(glfw::Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
