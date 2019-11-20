mod throttler;

use std::time::Duration;

use sdl2::event::Event;
use sdl2::rect::{Rect, Point};
use sdl2::pixels::Color;

use wrapped2d::b2;
use wrapped2d::user_data::NoUserData;
use wrapped2d::collisions::shapes::*;

use crate::throttler::Throttler;


const SCREEN_WIDTH: u32 = 600;
const SCREEN_HEIGHT: u32 = 600;

const SQUARE_WIDTH: u32 = 1;
const SQUARE_HEIGHT: u32 = 1;

const ZOOM: f32 = 40.0;

const TIME_STEP: f32 = 1.0 / 30.0;
const VELOCITY_ITERATIONS: i32 = 8;
const POSITION_ITERATIONS: i32 = 3;

type World = b2::World<NoUserData>;


fn sdl2_point(point: b2::Vec2) -> Point {
    return Point::new((point.x * ZOOM) as i32, (point.y * ZOOM) as i32);
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let window = video.window("Assemble", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered().build().map_err(|e| e.to_string()).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut canvas = window.into_canvas()
        .accelerated().build().map_err(|e| e.to_string()).unwrap();

    let start_pos = (10, 10);
    
    let mut force_multiplier = 0.0;
    let mut linear_damping = 0.97;

    let gravity = b2::Vec2 { x: 0.0, y: 0.0 };
    let mut world = b2::World::<NoUserData>::new(&gravity);

    let mut def = b2::BodyDef {
        body_type: b2::BodyType::Dynamic,
        position: b2::Vec2 { x: start_pos.0 as f32, y: start_pos.1 as f32 },
        linear_damping: linear_damping,
        .. b2::BodyDef::new()
    };

    let body_handle = world.create_body(&def);
    let fixture_handle;
 
    {
        let mut body = world.body_mut(body_handle);

        let shape = b2::PolygonShape::new_box(0.5, 0.5);
        fixture_handle = body.create_fast_fixture(&shape, 2.0);
    }

    let throttler = Throttler::new(Duration::from_millis(1000 / 30));

    let black = Color::RGBA(0, 0, 0, 255);
    let white = Color::RGBA(255, 255, 255, 255);
    let red = Color::RGBA(255, 0, 0, 255);
    let green = Color::RGBA(0, 255, 0, 255);

    // Main Game Loop
    let mut running = true;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{ .. }=> {
                    running = false;
                }

                Event::KeyDown{keycode, keymod, ..} => {
                    force_multiplier = 1.0;
                }

                Event::KeyUp{keycode, keymod, ..} => {
                    force_multiplier = 0.0;
                }

                Event::MouseMotion{x, y, ..} => {
                    //linear_damping = (x as f32 / SCREEN_WIDTH as f32);
                }

                Event::MouseButtonDown{mouse_btn, x, y, ..} => {
                }

                Event::MouseButtonUp{mouse_btn, ..} => {
                }

                _ => {}
            }
        }

        let force;
        let left_point;
        let right_point;
        {
            let mut body = world.body_mut(body_handle);
            force = body.world_vector(&b2::Vec2 { x: 0.0 * force_multiplier, y: 1.0 * force_multiplier });
            left_point = body.world_point(&b2::Vec2 { x: 0.0, y: 0.0 });
            right_point = body.world_point(&b2::Vec2 { x: 0.5, y: 0.0 });

            body.set_linear_damping(linear_damping);
            body.apply_force(&force, &left_point, true);
            body.apply_force(&force, &right_point, true);
        }

        world.step(TIME_STEP, VELOCITY_ITERATIONS, POSITION_ITERATIONS);

        canvas.set_draw_color(black);
        canvas.clear();

        let body = world.body(body_handle);

        let body_pos = body.position();
        let fixture = body.fixture(fixture_handle);
        let shape_type = fixture.shape_type();
        let shape = fixture.shape();
        match shape {
            UnknownShape::Polygon(polygon) => {
                canvas.set_draw_color(white);
                let body_point = sdl2_point(*body_pos);
                canvas.draw_rect(Rect::new(body_point.x,
                                           body_point.y,
                                           SQUARE_WIDTH * ZOOM as u32,
                                           SQUARE_HEIGHT * ZOOM as u32));

            }

            _ => panic!("Unexpected shape!"),
        }

        canvas.set_draw_color(red);
        let sdl_left = sdl2_point(left_point);
        let sdl_right = sdl2_point(right_point);

        canvas.set_draw_color(green);
        let sdl_force_left = sdl2_point(b2::Vec2 { x: left_point.x + force.x,
                                              y: left_point.y + force.y });
        let sdl_force_right = sdl2_point(b2::Vec2 { x: right_point.x + force.x,
                                                       y: right_point.y + force.y });
        canvas.draw_point(sdl_left);
        canvas.draw_point(sdl_right);
        canvas.draw_line(sdl_left, sdl_force_left);
        canvas.draw_line(sdl_right, sdl_force_right);

        canvas.present();

        throttler.wait();
    }
}
