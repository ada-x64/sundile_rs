use std::time::*;
use legion::*;

use sundile_graphics::prelude::*;
use sundile_assets::prelude::*;
use sundile_scripting::prelude::*;

use crate::renderer::*;
use crate::renderer2d::*;

pub struct Game<'a> {
    pub renderer: Renderer<'a>,
    pub renderer2d: Renderer2d<'a>,
    pub paused: bool,
    world: World,
    schedule: Schedule,
    resources: Resources,
    scenes: SceneMap, //TODO: Possibly move this outside of Game struct so DebugGui has ability to change scenes?
}

impl<'a> Game<'a> {
    //TODO: mut Assets temporary?
    pub fn new(render_target: &RenderTarget, assets: &'a mut AssetTypeMap, scenes: SceneMap, viewport: Option<Viewport>, paused: bool) -> Self  {
        let renderer = Renderer::new(&render_target, assets, viewport);
        let renderer2d = Renderer2d::new(&render_target, &assets,);

        let resources = Resources::default();
        // resources.insert(assets); // Don't send the entire assets struct here. Probably should access assets via some api.
        
        let mut world = World::default();
        scenes["default"](&mut world);

        Game {
            renderer,
            renderer2d,
            paused,
            world,
            schedule: Schedule::builder().build(), //TODO: Replace this with an actual script.
            resources,
            scenes,
        }
    }

    pub fn update(&mut self, dt: Duration) {
        if self.paused {return;}
        self.renderer.update(dt);
        self.schedule.execute(&mut self.world, &mut self.resources,);
    }

    pub fn render(&mut self, render_target: &mut RenderTarget, assets: &AssetTypeMap) {
        if self.paused {return;}
        self.renderer.render(render_target, &self.world, assets);
        self.renderer2d.render(render_target);
    }

    pub fn handle_input(&mut self, e: &winit::event::DeviceEvent) {
        if self.paused {return;}
        self.renderer.handle_input(&e);
    }

    //TODO: Scenes should be assets loaded with AssetManager struct.
    pub fn set_scene<'s>(&mut self, scene: &'s str) {
        self.world.clear();
        self.scenes[scene](&mut self.world);
    }
}