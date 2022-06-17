mod utils;
extern crate nanorand;

use nanorand::{WyRand, Rng};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug, Copy, Clone)]
struct Velocity {
	x: f64,
	y: f64,
	x_delta: f64,
	y_delta: f64,
}

#[derive(Debug, Copy, Clone)]
struct Particle {
		loc: (f64, f64),
		vel: Velocity,
		size: f64,
		color: (u8, u8, u8, u8),
		color_target: (u8, u8, u8, u8),
}

struct World {
	width: u32,
	height: u32,
	particles: Vec<Particle>,
}

impl World {
	fn new(width: u32, height: u32, max_particles: usize) -> World {
		let width = width;
		let height = height;
		let mut rng = WyRand::new();

		let mut particles = Vec::new();
		while particles.len() < max_particles {
			particles.push(Particle {
				loc: (
					rng.generate_range(0..width) as f64,
					rng.generate_range(0..height) as f64
				),
				vel: Velocity {
					x: rng.generate_range(0i16..25000) as f64 / 500_000.,
					y: rng.generate_range(-25000i16..0) as f64 / 500_000.,
					x_delta: rng.generate_range(-25000i16..=25000) as f64 / 500_000.,
					y_delta: rng.generate_range(-25000i16..=25000) as f64 / 500_000.,
				},
				size: rng.generate_range(2..7) as f64,
				color: (255, 255, 255, 128),
				color_target: (255, 255, 255, 128),
			})
		}

		World {
			width: width,
			height: height,
			particles: particles,
		}
	}

	fn update(&mut self) {
		for p in &mut self.particles {
			p.update(self.width, self.height);
		}
	}

	fn render(&self, tgt: &web_sys::CanvasRenderingContext2d) {
		tgt.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
		for p in &self.particles {
			tgt.begin_path();
			tgt.set_fill_style(&wasm_bindgen::JsValue::from_str(&p.color_hex()));
			tgt.move_to(p.loc.0.floor(), p.loc.1.floor());
			tgt.arc(p.loc.0.floor(), p.loc.1.floor(), p.size, 0.0, std::f64::consts::PI * 2.0).unwrap();
			tgt.fill();
			tgt.close_path();
		}
	}
}

impl Particle {
	fn update(&mut self, width: u32, height: u32) {
		let mut rng = WyRand::new();
		// Color stuff
		if self.color.3 < self.color_target.3 {
			self.color.3 += 1;
		} else if self.color.3 > self.color_target.3 {
			self.color.3 -= 1;
		} else if self.color.3 == self.color_target.3 {
			self.color_target.3 = rng.generate();
		}

		// Location wrap-around
		if self.loc.0 > width as f64 + 15.0 {
			self.loc.0 = -15.0;
		} else if self.loc.0 < -15.0 {
			self.loc.0 = width as f64 + 15.0;
		}
		if self.loc.1 > height as f64 + 15.0 {
			self.loc.1 = -15.0;
		} else if self.loc.1 < -15.0 {
			self.loc.1 = height as f64 + 15.0;
		}
		self.vel.update(&mut rng);
		self.loc.0 += self.vel.x;
		self.loc.1 += self.vel.y;
	}

	fn color_hex(&self) -> String {
		let (r, g, b, a) = self.color;
		format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
	}
}

impl Velocity {
	fn update(&mut self, rng: &mut WyRand){
		self.x += self.x_delta;
		self.x_delta = rng.generate_range(-25000i16..=25000) as f64 / 500_000.;
		self.y += self.y_delta;
		self.y_delta = rng.generate_range(-25000i16..=25000) as f64 / 500_000.;
		// Clamping, sorta.
		// The particles get too fast without the following.
		// Still allows for particles to go fast though...
		// Also makes particles tend to go to the upper right.
		if self.x > 1.0 {
			self.x -= 0.002;
		} else if self.x < 0.0 {
			self.x += 0.002;
		}
		if self.y > 0.0 {
			self.y -= 0.002;
		} else if self.y < 1.0 {
			self.y += 0.002;
		}
	}
}

fn window() -> web_sys::Window {
	web_sys::window().unwrap()
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
	window().request_animation_frame(f.as_ref().unchecked_ref()).unwrap();
}

#[wasm_bindgen(start)]
pub fn start() {
	utils::set_panic_hook();
	let window = web_sys::window().unwrap();
	let document = window.document().unwrap();
	let canvas = document.get_element_by_id("canvas").unwrap();
	let canvas: web_sys::HtmlCanvasElement = canvas
		.dyn_into::<web_sys::HtmlCanvasElement>()
		.map_err(|_| ())
		.unwrap();

	let context = canvas
		.get_context("2d")
		.unwrap().unwrap()
		.dyn_into::<web_sys::CanvasRenderingContext2d>()
		.unwrap();

	canvas.set_height(window.inner_height().unwrap().as_f64().unwrap() as u32);
	canvas.set_width(window.inner_width().unwrap().as_f64().unwrap() as u32);
	let mut world = World::new(canvas.width(), canvas.height(), 500);
	let mut frame = 0;

	let f = Rc::new(RefCell::new(None));
	let g = f.clone();
	world.update();
	world.render(&context);
	*g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
		let window_height = window.inner_height().unwrap().as_f64().unwrap() as u32;
		let window_width = window.inner_width().unwrap().as_f64().unwrap() as u32;
		if frame % 60 == 0 {
			if canvas.height() != window_height {
				canvas.set_height(window_height);
				world.height = window_height;
			}
			if canvas.width() != window_width {
				canvas.set_width(window_width);
				world.width = window_width;
			}
		}
		frame += 1;
		world.update();
		world.render(&context);
		request_animation_frame(f.borrow().as_ref().unwrap());
	}) as Box<dyn FnMut()>));
	request_animation_frame(g.borrow().as_ref().unwrap());
}
