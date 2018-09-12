extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;
extern crate image;
extern crate cgmath;

use self::gl::types::*;

use std::sync::mpsc::Receiver;
use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::path::Path;

mod shader;
use shader::Shader;

use image::GenericImage;
use image::ImageBuffer;

use std::fs;


fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("[ERROR] Call with texture name");
        std::process::exit(1);
    }

    // let image = &args[1];
    // println!("{}", args[1]);
    let cube_image_name = &args[1];
    let target_dir = format!("resources/textures/{}", cube_image_name);
    fs::create_dir_all(target_dir);

    let open_image_src = format!("resources/textures/{}.jpg", cube_image_name);
    println!("{}", open_image_src);
    
    let img = image::open(&Path::new("resources/textures/cube.jpg")).expect("Failed to load texture");
    let image_size: u32 = img.width() / 4 as u32;

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw.create_window(img.width(), img.height(), "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (our_shader, vbo, vao, ebo, texture) = unsafe {
        // build and compile our shader program
        // ------------------------------------
        let our_shader = Shader::new(
            "src/shaders/shader.vert.glsl",
            "src/shaders/shader.frag.glsl");

        // set up vertex data (and buffer(s)) and configure vertex attributes
        // ------------------------------------------------------------------
        // HINT: type annotation is crucial since default for float literals is f64
        let vertices: [f32; 32] = [
            // positions       // colors        // texture coords
             1.0,  1.0, 0.0,   1.0, 0.0, 0.0,   1.0, 1.0, // top right
             1.0, -1.0, 0.0,   0.0, 1.0, 0.0,   1.0, 0.0, // bottom right
            -1.0, -1.0, 0.0,   0.0, 0.0, 1.0,   0.0, 0.0, // bottom left
            -1.0,  1.0, 0.0,   1.0, 1.0, 0.0,   0.0, 1.0  // top left
        ];
        let indices = [
            0, 1, 3,  // first Triangle
            1, 2, 3   // second Triangle
        ];
        let (mut vbo, mut vao, mut ebo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                       (indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &indices[0] as *const i32 as *const c_void,
                       gl::STATIC_DRAW);

        let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
        // position attribute
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        // color attribute
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(1);
        // texture coord attribute
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const c_void);
        gl::EnableVertexAttribArray(2);

        // load and create a texture
        // -------------------------
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
        // set the texture wrapping parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        // set texture filtering parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        // load image, create texture and generate mipmaps
        
        let data = img.raw_pixels();
        gl::TexImage2D(gl::TEXTURE_2D,
                       0,
                       gl::RGB as i32,
                       img.width() as i32,
                       img.height() as i32,
                       0,
                       gl::RGB,
                       gl::UNSIGNED_BYTE,
                       &data[0] as *const u8 as *const c_void);
        // println!("{}, {}", img.width(), img.height());

        gl::GenerateMipmap(gl::TEXTURE_2D);

        (our_shader, vbo, vao, ebo, texture)
    };

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, image_size, cube_image_name);
        }

        // render
        // ------
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // bind Texture
            gl::BindTexture(gl::TEXTURE_2D, texture);

            // render container
            our_shader.useProgram();
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }

    // optional: de-allocate all resources once they've outlived their purpose:
    // ------------------------------------------------------------------------
    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ebo);
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, img_size: u32, cube_image_name: &String ) {
    match event {
        glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
        }
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
            println!("size: {}, {}", img_size, cube_image_name);

            let target_dir = format!("resources/textures/{}", cube_image_name);
            let image_names = vec!["negx", "posx", "negy", "posy", "negz", "posz"];

            for i in 0..image_names.len() {
                let image_src_name = format!("{}/{}.jpg", target_dir, image_names[i]);
                println!("{}", image_src_name);

                unsafe{

                    let mut imgbuf: image::RgbImage = ImageBuffer::new(img_size as u32, img_size as u32);
                    let imgbuf_ptr = imgbuf.as_mut_ptr();
                    let mut x:u32 = 0;
                    let mut y:u32 = 0;

                    match i {
                        0 => {x = 0; y = img_size;},
                        1 => {x = img_size * 2; y = img_size;},
                        2 => {x = img_size; y = img_size * 2;},
                        3 => {x = img_size; y = 0;},
                        4 => {x = img_size; y = img_size;},
                        _ => {x = img_size * 3; y = img_size;},
                    }

                    gl::ReadPixels(x as GLint, y as GLint, img_size as GLint, img_size as GLint, gl::RGB, gl::UNSIGNED_BYTE, imgbuf_ptr as *mut c_void );

                    imgbuf.save(image_src_name).unwrap();

                }
            }

            
        }
        _ => {}
    }
}