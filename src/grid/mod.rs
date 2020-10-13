use wasm_bindgen::prelude::*;
use std::mem;
use rand::Rng;
use web_sys::{WebGlBuffer, WebGl2RenderingContext};
use super::cell::Cell;

#[wasm_bindgen]
pub struct Grid {
    width: u32,
    height: u32,
    cells: Vec<u8>, // Each cell is one bit => `cells` is 8 cells aligned

    vertices: Vec<f32>,
    idxs: Vec<u32>,
    vbo: Option<WebGlBuffer>,
    ebo: Option<WebGlBuffer>
}

impl Grid {
    /// Creates a new Grid with Cells that are randomly either Cell::Dead or
    /// Cell::Alive.
    pub fn new(width: u32, height: u32) -> Grid {
        let mut rng = rand::thread_rng();

        let mut cells = Vec::with_capacity((width as usize) * (height as usize));
        for _ in 0..cells.capacity() {
            cells.push(rng.gen());
        }

        Grid {
            width: width,
            height: height,
            cells: cells,

            vertices: Vec::<f32>::new(),
            idxs: Vec::<u32>::new(),
            vbo: None,
            ebo: None
        }
    }

    /// Initializes the WebGL aspect of the Grid
    pub fn init_gl(&mut self, context: &WebGl2RenderingContext, win_width: f32, win_height: f32) {
        let cell_size = (win_width/(self.width as f32)).min(win_height/(self.height as f32));
        let cell_margin = 0.08 * cell_size;

        let x_padding = (win_width - (self.width as f32) * cell_size)/2.0;
        let y_padding = (win_height - (self.height as f32) * cell_size)/2.0;

        self.vertices = Vec::with_capacity((self.width as usize) * (self.height as usize) * 4 * (2 + 1)); // 4 vertices per grid cell, 2 coordinates per vertex, 1 color (black or white) per vertex
        for x in 0..self.width {
            for y in 0..self.height {
                let x1 = (x as f32) * cell_size;
                let x2 = ((x+1) as f32) * cell_size;
                let y1 = (y as f32) * cell_size;
                let y2 = ((y+1) as f32) * cell_size;
                let color = match self.get_cell(x as i32, y as i32) {
                    Cell::Dead => 1.0,
                    Cell::Alive => 0.0
                };

                self.vertices.push((2.0 * (x1 + x_padding + cell_margin)/win_width) - 1.0);
                self.vertices.push((2.0 * (y1 + y_padding + cell_margin)/win_height) - 1.0);
                self.vertices.push(color);

                self.vertices.push((2.0 * (x2 + x_padding - cell_margin)/win_width) - 1.0);
                self.vertices.push((2.0 * (y1 + y_padding + cell_margin)/win_height) - 1.0);
                self.vertices.push(color);

                self.vertices.push((2.0 * (x2 + x_padding - cell_margin)/win_width) - 1.0);
                self.vertices.push((2.0 * (y2 + y_padding - cell_margin)/win_height) - 1.0);
                self.vertices.push(color);

                self.vertices.push((2.0 * (x1 + x_padding + cell_margin)/win_width) - 1.0);
                self.vertices.push((2.0 * (y2 + y_padding - cell_margin)/win_height) - 1.0);
                self.vertices.push(color);
            }
        }

        // Indices
        for i in 0..(self.width as usize) * (self.height as usize) {
            // First triangle
            self.idxs.push((i*4) as u32);
            self.idxs.push(((i*4)+1) as u32);
            self.idxs.push(((i*4)+2) as u32);

            // Second triangle
            self.idxs.push((i*4) as u32);
            self.idxs.push(((i*4)+2) as u32);
            self.idxs.push(((i*4)+3) as u32);
        }

        // Create VBO and EBO
        self.vbo = context.create_buffer();
        self.ebo = context.create_buffer();

        // Bind and set VBO
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vbo.as_ref());
        unsafe {
            let vbo_array = js_sys::Float32Array::view(&self.vertices);
            context.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &vbo_array, WebGl2RenderingContext::DYNAMIC_DRAW);
        }

        // Bind and set EBO
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.ebo.as_ref());
        unsafe {
            let ebo_array = js_sys::Uint32Array::view(&self.idxs);
            context.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, &ebo_array, WebGl2RenderingContext::DYNAMIC_DRAW);
        }

        // Vertex position
        context.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>() as i32, 0);
        context.enable_vertex_attrib_array(0);

        // Vertex color
        context.vertex_attrib_pointer_with_i32(1, 1, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>() as i32, 2 * mem::size_of::<f32>() as i32);
        context.enable_vertex_attrib_array(1);

        // Unbind VBO
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
    }

    fn update_vertex_data(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                let color = match self.get_cell(x as i32, y as i32) {
                    Cell::Dead => 1.0,
                    Cell::Alive => 0.0
                };

                let idx: usize = ((x*self.height + y) * 4 * (2+1)) as usize;
                self.vertices[idx+2] = color;
                self.vertices[idx+5] = color;
                self.vertices[idx+8] = color;
                self.vertices[idx+11] = color;
            }
        }
    }

    /// Draws the Grid on to the context
    pub fn draw(&mut self, context: &WebGl2RenderingContext) {
        // Update the vertices array
        self.update_vertex_data();
        
        // Bind and set VBO
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, self.vbo.as_ref());
        unsafe {
            let vbo_array = js_sys::Float32Array::view(&self.vertices);
            context.buffer_sub_data_with_i32_and_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, 0, &vbo_array);
        }
        
        // Bind and set EBO
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, self.ebo.as_ref());
        unsafe {
            let ebo_array = js_sys::Uint32Array::view(&self.idxs);
            context.buffer_sub_data_with_i32_and_array_buffer_view(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, 0, &ebo_array);
        }

        // Vertex position
        context.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>() as i32, 0);
        context.enable_vertex_attrib_array(0);

        // Vertex color
        context.vertex_attrib_pointer_with_i32(1, 1, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>() as i32, 2 * mem::size_of::<f32>() as i32);
        context.enable_vertex_attrib_array(1);

        // Unbind VBO
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

        // Draw
        context.draw_elements_with_i32(WebGl2RenderingContext::TRIANGLES, self.idxs.len() as i32, WebGl2RenderingContext::UNSIGNED_INT, 0);
    }

    /// Origin (0, 0) is at the top left with x increasing to the right, and y
    /// increasing down. This function automatically wraps around the sides
    /// (top/bottom, left/right).
    pub fn get_cell(&self, mut x: i32, mut y: i32) -> Cell {
        x += self.width as i32; // Gets rid of negative x values
        y += self.height as i32; // Gets rid of negative y values
        let idx: usize = ((x % (self.width as i32)) + ((y % (self.height as i32)) * self.width as i32)) as usize;
        ((self.cells[(idx as usize)/8] & (1 << (idx % 8))) >> (idx % 8)).into() // 8 bits in a byte
    }

    /// The number of living neighbours around (x, y) with the origin (0, 0) at
    /// the top left (x increasing to the right, and y increasing down).
    fn num_live_neighbours(&self, x: u32, y: u32) -> u32 {
        let mut count = 0;
        // TODO: There are a few optimizations we can do here that don't involve looping
        for i in -1..=1 {
            for j in -1..=1 {
                if (i == 0) && (j == 0) { continue };
                count += self.get_cell((x as i32) + i, (y as i32) + j) as u32;
            }
        }
        count
    }

    /// Steps through one iteration of Conway's Game of Life
    pub fn step(&mut self) {
        let mut new_cells = self.cells.clone();
        for x in 0..self.width {
            for y in 0..self.height {
                let new_cell = match (self.get_cell(x as i32, y as i32), self.num_live_neighbours(x, y)) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (curr_state, _) => curr_state // Default behaviour is to remain in the state you were in
                };
                let idx = (x % self.width) + (y % self.height) * self.width;
                new_cells[(idx as usize)/8] = (new_cells[(idx as usize)/8] & !(1 << (idx % 8))) | (u8::from(new_cell) << (idx % 8)); // Update the cell
            }
        }
        // Update the grid
        self.cells = new_cells;
    }
}
