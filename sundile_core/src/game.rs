use std::time::*;
use legion::*;
use graphics::prelude::*;

use crate::renderer::*;
use crate::renderer2d::*;

pub struct Game<'a> {
    pub renderer: Renderer,
    pub renderer2d: Renderer2d<'a>,
    world: World,
    schedule: Schedule,
    resources: Resources,
}

impl<'a> Game<'a> {
    pub fn new<F>(render_target: &RenderTarget, viewport: Option<Viewport>, systems_fn: F) -> Self
        where F: Fn() -> Schedule {
        let assets = assets::load_from_bin(&render_target).expect("Unable to load assets!");

        let renderer = Renderer::new(&render_target, &assets, viewport);
        let renderer2d = Renderer2d::new(&render_target, &assets,);

        let mut resources = Resources::default();
        resources.insert(assets);
        
        Game {
            renderer,
            renderer2d,
            world: World::default(),
            schedule: systems_fn(),
            resources,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.renderer.update(dt);
        self.schedule.execute(&mut self.world, &mut self.resources,);
    }

    pub fn render(&mut self, render_target: &mut RenderTarget,) {
        self.renderer.render(render_target, &self.world);
        self.renderer2d.render(render_target);
    }

    pub fn handle_input(&mut self, e: &winit::event::DeviceEvent) {
        self.renderer.handle_input(&e);
    }

    //TODO: p3 -> find a way to do this without passing render_target?
    //TODO: Scenes should be assets loaded with AssetManager struct.
    pub fn init_scene(&mut self, _scene_id: usize, ) {
        self.world.clear();
    }
}