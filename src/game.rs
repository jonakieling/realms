use ggez;
use ggez::GameResult;
use ggez::{conf, event::*, ContextBuilder};
use ggez::Context;
use ggez::graphics;

pub struct World { }

impl EventHandler for World {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
    	Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
    	Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
    }

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: Keycode, _keymod: Mod, _repeat: bool) {
    }
}

pub fn run() {
	let screen_height = 512;
    let screen_width = 768;

    let cb = ContextBuilder::new("realms", "jonakieling")
        .window_setup(conf::WindowSetup::default().title("realms"))
        .window_mode(conf::WindowMode::default().dimensions(screen_width, screen_height));

    let ctx = &mut cb.build().unwrap();
    graphics::set_background_color(ctx, graphics::BLACK);
    graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);

    if let Err(_) = ggez::event::run(ctx, &mut World {}) {
        println!("An error occured while running the ggez event loop");
    }
}