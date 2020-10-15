use std::cell::RefCell;
use std::rc::Rc;
// use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

// extern crate console_error_panic_hook;

mod cell;
mod grid;

static VERT_SHADER_SRC: &str = r#"#version 100
    attribute vec2 attr_position;
    attribute float attr_color;
    varying float out_color;
    void main() {
        gl_Position = vec4(attr_position, 0.0, 1.0);
        out_color = attr_color;
    }
"#;

static FRAG_SHADER_SRC: &str = r#"#version 100
    precision mediump float;
    varying float out_color;
    void main() {
        gl_FragColor = vec4(out_color, out_color, out_color, 1.0);
    }
"#;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("`window` does not have a `document`");
    let body = document.body().expect("`document` does not have a `body`");

    let overlay = document.create_element("div")?;
    overlay.set_class_name("overlay");

    let slider_text = document.create_element("p")?;
    slider_text.set_inner_html("Delay: 200ms");
    overlay.append_child(slider_text.as_ref())?;

    let slider = document.create_element("input")?.dyn_into::<web_sys::HtmlInputElement>()?;
    slider.set_attribute("type", "range")?;
    slider.set_attribute("min", "0")?;
    slider.set_attribute("max", "1000")?;
    slider.set_attribute("value", "200")?;
    overlay.append_child(slider.as_ref())?;

    body.append_child(overlay.as_ref())?;

    let win_size: (u32, u32) = (window.inner_width()?.as_f64().unwrap() as u32, window.inner_height()?.as_f64().unwrap() as u32);

    let canvas = document.create_element("canvas")?.dyn_into::<web_sys::HtmlCanvasElement>()?;
    canvas.set_width(win_size.0);
    canvas.set_height(win_size.1);
    body.append_child(canvas.as_ref())?;

    let context = canvas.get_context("webgl")?.expect("Browser does not support webgl").dyn_into::<WebGlRenderingContext>()?;

    let vert_shader = compile_shader(&context, WebGlRenderingContext::VERTEX_SHADER, VERT_SHADER_SRC)?;
    let frag_shader = compile_shader(&context, WebGlRenderingContext::FRAGMENT_SHADER, FRAG_SHADER_SRC)?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));
    context.delete_shader(Some(&vert_shader));
    context.delete_shader(Some(&frag_shader));

    let mut grid = grid::Grid::new(win_size.0/20, win_size.1/20);
    grid.init_gl(&context, &program, win_size.0 as f32, win_size.1 as f32);

    let animate_callback = Rc::new(RefCell::new(None));
    let animate_callback2 = animate_callback.clone();

    let timeout_callback = Rc::new(RefCell::new(None));
    let timeout_callback2 = timeout_callback.clone();

    let window1 = web_sys::window().expect("no global `window` exists");
    let window2 = web_sys::window().expect("no global `window` exists");

    *timeout_callback2.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        request_animation_frame(&window1, animate_callback.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    *animate_callback2.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        grid.step();

        context.clear_color(0.5, 0.5, 0.5, 1.0);
        context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        grid.draw(&context);

        let speed = slider.value().parse::<i32>().unwrap();
        slider_text.set_inner_html(&format!("Delay: {0}ms", speed));

        set_timeout(&window2, timeout_callback.borrow().as_ref().unwrap(), speed);
    }) as Box<dyn FnMut()>));

    request_animation_frame(&window, animate_callback2.borrow().as_ref().unwrap());

    Ok(())
}

fn compile_shader(context: &WebGlRenderingContext, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = context.create_shader(shader_type).ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, &source);
    context.compile_shader(&shader);

    if context.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
        Ok(shader)
    }
    else {
        Err(context.get_shader_info_log(&shader).unwrap_or_else(|| String::from("Unkown error creating shader")))
    }
}

fn link_program(context: &WebGlRenderingContext, vert_shader: &WebGlShader, frag_shader: &WebGlShader) -> Result<WebGlProgram, String> {
    let program = context.create_program().ok_or_else(|| String::from("Unable to create program object"))?;
    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
        Ok(program)
    }
    else {
        Err(context.get_program_info_log(&program).unwrap_or_else(|| String::from("Unknown error creating program")))
    }
}

fn request_animation_frame(window: &web_sys::Window, f: &Closure<dyn FnMut()>) -> i32 {
    window
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK")
}

fn set_timeout(window: &web_sys::Window, f: &Closure<dyn FnMut()>, timeout_ms: i32) -> i32 {
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            f.as_ref().unchecked_ref(),
            timeout_ms,
        )
        .expect("should register `setTimeout` OK")
}
