mod loader;
//
use loader::AssetManager;

mod dung_gen;
use dung_gen::DungGen;

mod graphics;

mod components;
mod systems;
use components::*;
use crate::components::components::*;
use crate::systems::systems::*;

use std::f32::consts::PI;
use rand::seq::SliceRandom;
use std::ops::{Add, Mul};
use std::process::exit;

use specs::prelude::*;
use specs::{DispatcherBuilder, WorldExt, Builder, System, AccessorCow, RunningTime};
use specs::Component;
use specs::prelude::*;

use winit::event_loop::{EventLoop, ControlFlow};
use winit::dpi::{PhysicalSize};
use winit::event::{Event, WindowEvent};

use std::{mem, slice};
use crate::graphics::{Vertex, create_vertices, Model, Mesh};
use wgpu::{TextureViewDimension, CompareFunction, PrimitiveTopology, BufferDescriptor};
use cgmath::{Vector2, Vector3};


fn main() {
    let mut ass_man = AssetManager::new();
    let ds = ass_man.load_display_settings();


    let event_loop = EventLoop::new();

    let size = PhysicalSize { width: ds.screen_width, height: ds.screen_height };

    let mut builder = winit::window::WindowBuilder::new()
        .with_title("deeper")
        .with_inner_size(size);
    let window = builder.build(&event_loop).unwrap();

    let surface = wgpu::Surface::create(&window);

    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::PRIMARY,
        }
    ).unwrap();

    let (device, mut queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false
            },
            limits: Default::default()
        }
    );

    let context = graphics::Context::new(&device);

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: graphics::COLOR_FORMAT,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Vsync,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let mut init_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0});

    let vertex_size = mem::size_of::<Vertex>();
    let (vertex_data, index_data) = create_vertices();

    let indexed_vertices = {
        let mut verts = vec!();
        for idx in &index_data {
            verts.push(vertex_data[*idx as usize]);
        }
        verts
    };

    let vertex_buf =
        //device.create_buffer_with_data(vertex_data.as_bytes(), wgpu::BufferUsage::VERTEX);
        device.create_buffer_mapped::<Vertex>(
            indexed_vertices.len(),
            wgpu::BufferUsage::VERTEX,
        ).fill_from_slice(indexed_vertices.as_slice());


    let size = 256u32;

    let texture_extent = wgpu::Extent3d {
        width: size,
        height: size,
        depth: 1,
    };

    let texels = graphics::create_texels(size as usize);

    let temp_buf =
        device.create_buffer_mapped::<u8>(texels.len(), wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(texels.as_slice());

   init_encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            row_pitch: 4 * size,
            image_height: 0,
        },
        wgpu::TextureCopyView {
            texture: &context.texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        },
    texture_extent,
    );

    let init_command_buf = init_encoder.finish();

    queue.submit(&[init_command_buf]);

    // End graphics shit

    let dungeon = DungGen::new()
        .width(60)
        .height(60)
        .n_rooms(10)
        .room_min(5)
        .room_range(5)
        .generate();

    let player_start = dungeon
        .room_centers
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone();

    let mut world = World::new();

    register_components(&mut world);

    let mut model_array = vec![
        Model {
            meshes: vec![Mesh {
                num_vertices: indexed_vertices.len(), vertex_buffer: vertex_buf, offset: [0.0, 0.0, 0.0]
            }]
        },
    ];

    // initialize dispacher with all game systems
    let mut dispatcher = DispatcherBuilder::new()
        .with(DunGenSystem { dungeon }, "DunGenSystem", &[])
        .with(PlayerSystem::new(), "PlayerSystem", &[])
        .with(Physics2DSystem, "Physics2DSystem", &["PlayerSystem"])
        .with(
            MovementSystem,
            "MovementSystem",
            &["Physics2DSystem", "PlayerSystem"],
        )
        .with(
            SphericalFollowSystem,
            "SphericalFollowSystem",
            &["MovementSystem"],
        )
        .with_thread_local(GraphicsSystem::new(
            context, model_array, sc_desc, swap_chain, device, queue
        ))
        .build();

    let player = world
        .create_entity()
        .with(Position(Vector2::new(player_start.0 as f32, player_start.1 as f32)))
        .with(DynamicBody)
        .with(CircleCollider { radius: 0.5 })
        .with(Velocity::new())
        .with(Model3D::from_index(0).with_scale(0.5))
        .build();

    let player_camera = world
        .create_entity()
        .with(components::components::Camera {
            up: Vector3::unit_z(),
            fov: 25.0,
        })
        .with(Target(player))
        .with(Position3D(Vector3::new(0.0, 0.0, 0.0)))
        .with(SphericalOffset::new())
        .build();

    world.insert(Player::from_entity(player));
    world.insert(ActiveCamera(player_camera));
    world.insert(PlayerCamera(player_camera));

    // Setup world
    dispatcher.setup(&mut world);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                dispatcher.dispatch(&mut world);
                window.request_redraw();
                println!("boop");
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                unimplemented!();
            },
            Event::RedrawRequested(_) => {},
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

