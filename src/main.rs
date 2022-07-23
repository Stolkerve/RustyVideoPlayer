#![allow(non_snake_case)]

extern crate glfw;
extern crate cgmath;
extern crate gl;

mod gl_renderer;
mod video_renderer;

use cgmath::Matrix;
use glfw::{Action, Context};

fn main() {
    let (mut width, mut height) = (800, 600);
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

    let (shader_program, vao, vbo, ebo, texture, mut video_ctx) = unsafe {
        let shader_program = gl_renderer::create_shader(gl_renderer::VERTEX_SHADER_SOURCE, gl_renderer::FRAGMENT_SHADER_SOURCE);
        
        let (vbo, vao, ebo) = gl_renderer::create_geometry();
        let texture = gl_renderer::generate_texture();
       
        let mut video_ctx: video_renderer::VideoContext = video_renderer::VideoContext {
            width: 0,
            height: 0,
            time_base: ffmpeg_sys_next::AVRational { num: 0, den: 0 }, 
            sws_scale_ctx: std::ptr::null_mut(),
            format_context: std::ptr::null_mut(),
            video_codec_parameters: std::ptr::null_mut(),
            video_codec: std::ptr::null_mut(),
            video_stream_index: -1,
            codec_context: std::ptr::null_mut(),
            frame: std::ptr::null_mut(),
            packet: std::ptr::null_mut()
        };

        video_renderer::load_video(&mut video_ctx, "res/FatCat.mp4");

        (shader_program, vao, vbo, ebo, texture, video_ctx)
    };

    let uniform_name = std::ffi::CString::new("projection").unwrap();
    let mut data: Vec<u8> = Vec::new();
    
    while !window.should_close() {
        handle_window_event(&mut width, &mut height, &mut window, &events);     
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 0.1);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);

            let mut projection_matrix = cgmath::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
            let mut model = cgmath::Matrix4::from_translation(cgmath::vec3(0.0, 0.0, 0.0));
            model = model * cgmath::Matrix4::from_translation(cgmath::vec3(0.5 * (width as f32), 0.5 * (height as f32), 0.0));
            model = model * cgmath::Matrix4::from_nonuniform_scale(1.0 * width as f32, height as f32 , 1.0);
            projection_matrix = projection_matrix * model;

            let mut pts: i64 = 0;
            video_renderer::read_video_frame(&mut video_ctx, &mut data, &mut pts);

            static mut FIRST_FRAME: bool = true;
            if FIRST_FRAME {
                glfw.set_time(0.0);
                gl_renderer::create_texture(texture, video_ctx.width, video_ctx.height, gl::RGBA, gl::UNSIGNED_BYTE, gl::RGB as i32, data.as_ptr() as *const std::ffi::c_void);
                FIRST_FRAME = false;
            }

            let pt_in_sec = pts as f64 * video_ctx.time_base.num as f64 / video_ctx.time_base.den as f64;

            if f64::ceil((*video_ctx.format_context).duration as f64 / 1000000.0) == f64::ceil(pt_in_sec)
            {
                break;
            }

            // wait
            while pt_in_sec > glfw.get_time()
            {
            }

            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, video_ctx.width, video_ctx.height, gl::RGBA, gl::UNSIGNED_BYTE, data.as_ptr() as *const std::ffi::c_void);

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

    unsafe {
        video_renderer::free_video_data(&mut video_ctx);
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
