use gameboy::{SCREEN_W, SCREEN_H};
use glium::index::PrimitiveType;
use glium::{Program, VertexBuffer, Rect, Surface, IndexBuffer};
use glium::texture::{Texture2d, UncompressedFloatFormat, MipmapsOption};
use glium::backend::Facade;
use cgmath::SquareMatrix;
use cgmath;

const VERT_SHADER_SRC: &'static str = include_str!("shaders/vert.glsl");
const FRAG_SHADER_SRC: &'static str = include_str!("shaders/frag.glsl");
const ASPECT_RATIO: f32 = SCREEN_W as f32 / SCREEN_H as f32;

#[derive(Copy, Clone)]
pub struct Vertex {
    pos: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, pos, tex_coords);

pub struct Renderer {
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    texture: Texture2d,
    prog: Program,
    colour_lut: cgmath::Matrix4<f32>,
    perspective: cgmath::Matrix4<f32>,
}

impl Renderer {
    // TODO: Add error handling
    pub fn new<F: Facade>(display: &F) -> Renderer {
        let vertexes = [Vertex {
                            pos: [0.5, 0.5],
                            tex_coords: [1.0, 0.0],
                        },
                        Vertex {
                            pos: [0.5, -0.5],
                            tex_coords: [1.0, 1.0],
                        },
                        Vertex {
                            pos: [-0.5, 0.5],
                            tex_coords: [0.0, 0.0],
                        },
                        Vertex {
                            pos: [-0.5, -0.5],
                            tex_coords: [0.0, 1.0],
                        }];
        let vbo = VertexBuffer::new(display, &vertexes).unwrap();
        let ibo = IndexBuffer::new(display, PrimitiveType::TrianglesList, &[2, 3, 1, 2, 0, 1])
            .unwrap();
        // A non power of 2 texture isn't ideal, but shouldn't hurt perf too much...
        let mut texture = Texture2d::empty_with_format(display,
                                                       UncompressedFloatFormat::U8,
                                                       MipmapsOption::NoMipmap,
                                                       SCREEN_W as u32,
                                                       SCREEN_H as u32)
            .unwrap();
        let program = Program::from_source(display, VERT_SHADER_SRC, FRAG_SHADER_SRC, None)
            .unwrap();

        // TODO: Allow switching of colours.
        let colour_lut = cgmath::Matrix4::new(255.0,
                                              255.0,
                                              255.0,
                                              255.0,
                                              192.0,
                                              192.0,
                                              192.0,
                                              192.0,
                                              128.0,
                                              128.0,
                                              128.0,
                                              128.0,
                                              64.0,
                                              64.0,
                                              64.0,
                                              64.0) / 255.0;

        Renderer {
            vertex_buffer: vbo,
            index_buffer: ibo,
            texture: texture,
            prog: program,
            colour_lut: colour_lut,
            perspective: cgmath::Matrix4::<f32>::identity(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let new_aspect_ratio = width as f32 / height as f32;
        let scale = ASPECT_RATIO / new_aspect_ratio;
        self.perspective.x.x = scale;
    }

    pub fn update_texture(&mut self, pixels: &[u8]) {
        // FIXME: HACK
        let v: Vec<Vec<_>> =
            pixels.chunks(160).map(|chunk| chunk.iter().cloned().collect()).collect();
        self.texture.write(Rect {
                               bottom: 0,
                               left: 0,
                               width: SCREEN_W as u32,
                               height: SCREEN_H as u32,
                           },
                           v);
    }

    pub fn render<S: Surface>(&self, frame: &mut S) {
        // TODO MOVE ME
        let uniforms = uniform! {
            perspective: Into::<[[f32; 4]; 4]>::into(self.perspective),
            colour_lut: Into::<[[f32; 4]; 4]>::into(self.colour_lut),
            tex: &self.texture,
        };

        frame.draw(&self.vertex_buffer,
                  &self.index_buffer,
                  &self.prog,
                  &uniforms,
                  &Default::default())
            .unwrap();
    }
}
