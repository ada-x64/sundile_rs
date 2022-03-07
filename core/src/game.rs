use std::time::*;
use legion::*;
use sundile_graphics::prelude::*;
use sundile_assets::prelude::*;

use crate::renderer::*;
use crate::renderer2d::*;
use crate::SceneMap;

pub struct Game<'a> {
    pub renderer: Renderer<'a>,
    pub renderer2d: Renderer2d<'a>,
    world: World,
    schedule: Schedule,
    resources: Resources,
    scenes: SceneMap, //TODO: Possibly move this outside of Game struct so DebugGui has ability to change scenes?
}

impl<'a> Game<'a> {
    pub fn new(render_target: &RenderTarget, assets: &Assets, scenes: SceneMap, viewport: Option<Viewport>) -> Self  {
        let renderer = Renderer::new(&render_target, &assets, viewport);
        let renderer2d = Renderer2d::new(&render_target, &assets,);

        let resources = Resources::default();
        // resources.insert(assets); // Don't send the entire assets struct here. Probably should access assets via some api.
        
        let mut world = World::default();
        scenes["default"](&mut world);

        Game {
            renderer,
            renderer2d,
            world,
            schedule: Schedule::builder().build(), //TODO: Replace this with an actual script.
            resources,
            scenes
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.renderer.update(dt);
        self.schedule.execute(&mut self.world, &mut self.resources,);
    }

    pub fn render(&mut self, render_target: &mut RenderTarget, assets: &Assets) {
        self.renderer.render(render_target, &self.world, assets);
        self.renderer2d.render(render_target);
    }

    pub fn handle_input(&mut self, e: &winit::event::DeviceEvent) {
        self.renderer.handle_input(&e);
    }

    //TODO: p3 -> find a way to do this without passing render_target?
    //TODO: Scenes should be assets loaded with AssetManager struct.
    pub fn set_scene<'s>(&mut self, scene: &'s str) {
        self.world.clear();
        self.scenes[scene](&mut self.world);
    }
}