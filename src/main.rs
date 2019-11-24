mod throttler;

use std::time::Duration;
use std::ops::Mul;

use sdl2::event::Event;
use sdl2::rect::{Rect, Point};
use sdl2::pixels::Color;
use sdl2::keyboard::Keycode;

use wrapped2d::b2;
use wrapped2d::user_data::NoUserData;
use wrapped2d::collision::shapes::*;
use wrapped2d::common::math::*;

use crate::throttler::Throttler;


const SCREEN_WIDTH: u32 = 600;
const SCREEN_HEIGHT: u32 = 600;

const SQUARE_WIDTH: u32 = 1;
const SQUARE_HEIGHT: u32 = 1;

const ZOOM: f32 = 40.0;

const WORLD_WIDTH: f32 = SCREEN_WIDTH as f32 / ZOOM;
const WORLD_HEIGHT: f32 = SCREEN_HEIGHT as f32 / ZOOM;

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
    
    let mut force_x = 0.0;
    let mut force_y = 0.0;
    let mut linear_damping = 0.97;
    let mut angular_damping = 0.97;

    let gravity = b2::Vec2 { x: 0.0, y: 0.0 };
    let mut world = b2::World::<NoUserData>::new(&gravity);

    let mut def = b2::BodyDef {
        body_type: b2::BodyType::Dynamic,
        position: b2::Vec2 { x: start_pos.0 as f32, y: start_pos.1 as f32 },
        linear_damping,
        angular_damping,
        .. b2::BodyDef::new()
    };

    // Create ship
    let body_handle = world.create_body(&def);
    let mut body_fixture = b2::FixtureDef {
        friction: 1.0,
        restitution: 1.0,
        density: 1.0,
        is_sensor: false,
        .. b2::FixtureDef::new()
    };

    let fixture_handle;
 
    {
        let mut body = world.body_mut(body_handle);
            
        let shape = b2::PolygonShape::new_box(0.5, 0.5);
        fixture_handle = body.create_fixture(&shape, &mut body_fixture);
    }

    // Create walls
    let mut wall_def = b2::BodyDef {
        position: b2::Vec2 { x: 0.0, y: 0.0 },
        .. b2::BodyDef::new()
    };
    let mut wall_fixture = b2::FixtureDef {
        restitution: 1.4,
        density: 0.0,
        is_sensor: false,
        .. b2::FixtureDef::new()
    };
 
    let wall_edge = 0.5;
    let wall_points = [Vec2::from([wall_edge, wall_edge]),
                       Vec2::from([WORLD_WIDTH - wall_edge, wall_edge]),
                       Vec2::from([WORLD_WIDTH - wall_edge, WORLD_HEIGHT - wall_edge]),
                       Vec2::from([wall_edge, WORLD_HEIGHT - wall_edge]),
                       Vec2::from([wall_edge, wall_edge])];

    for wall_points in wall_points.windows(2) {
        let wall_start = wall_points[0];
        let wall_end = wall_points[1];

        let wall_handle = world.create_body(&wall_def);
        let mut body = world.body_mut(wall_handle);

        let shape = b2::EdgeShape::new_with(&wall_start, &wall_end);
        body.create_fixture(&shape, &mut wall_fixture);
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
                    if keycode == Some(Keycode::A) {
                        force_x = 1.0;
                    }
                    if keycode == Some(Keycode::S) {
                        force_y = 1.0;
                    }
                    if keycode == Some(Keycode::Q) {
                        running = false;
                    }
                }

                Event::KeyUp{keycode, keymod, ..} => {
                    if keycode == Some(Keycode::A) {
                        force_x = 0.0;
                    } 
                    if keycode == Some(Keycode::S) {
                        force_y = 0.0;
                    }
                }

                Event::MouseMotion{x, y, ..} => {
                }

                Event::MouseButtonDown{mouse_btn, x, y, ..} => {
                }

                Event::MouseButtonUp{mouse_btn, ..} => {
                }

                _ => {}
            }
        }

        world.step(TIME_STEP, VELOCITY_ITERATIONS, POSITION_ITERATIONS);

        let left_force;
        let right_force;
        let left_point;
        let right_point;
        {
            let mut body = world.body_mut(body_handle);
            left_force = body.world_vector(&b2::Vec2 { x: 0.0, y: 2.0 * force_x });
            right_force = body.world_vector(&b2::Vec2 { x: 0.0, y: 2.0 * force_y });
            left_point = body.world_point(&b2::Vec2 { x: -0.5, y: -0.5 });
            right_point = body.world_point(&b2::Vec2 { x: 0.5, y: -0.5 });

            body.set_linear_damping(linear_damping);
            body.apply_force(&left_force, &left_point, true);
            body.apply_force(&right_force, &right_point, true);
        }

        for contact in world.contacts() {
            let manifold = contact.world_manifold();
            let mut body = world.body_mut(body_handle);
            body.apply_force(&manifold.normal, &manifold.points[0], true);
        }

        canvas.set_draw_color(black);
        canvas.clear();

        let body = world.body(body_handle);
        let body_transform = Transform {
            pos: *body.position(),
            rot: Rot::from_angle(body.angle()),
        };

        canvas.set_draw_color(white);
        let body_pos = body.position();
        let fixture = body.fixture(fixture_handle);
        let shape_type = fixture.shape_type();
        let shape = fixture.shape();
        match &*shape {
            UnknownShape::Polygon(polygon) => {
                let first_vertex = body_transform.mul(*polygon.vertex(0));
                let first_point = sdl2_point(first_vertex);;
                let mut prev_point = first_point;
                for index in 0..polygon.vertex_count() {
                    let vertex = body_transform.mul(*polygon.vertex(index));

                    let point = sdl2_point(vertex);
                    canvas.draw_line(prev_point, point);
                    prev_point = point;
                }

                let last_vertex = polygon.vertex(polygon.vertex_count() - 1);
                let last_vertex = body_transform.mul(*last_vertex);
                let last_point = sdl2_point(last_vertex);
                canvas.draw_line(first_point, last_point);
            }

            _ => panic!("Unexpected shape!"),
        }

        canvas.set_draw_color(red);
        let sdl_left = sdl2_point(left_point);
        let sdl_right = sdl2_point(right_point);

        canvas.set_draw_color(green);
        let sdl_force_left = sdl2_point(b2::Vec2 { x: left_point.x + -left_force.x,
                                                   y: left_point.y + -left_force.y });
        let sdl_force_right = sdl2_point(b2::Vec2 { x: right_point.x + -right_force.x,
                                                    y: right_point.y + -right_force.y });
        canvas.draw_point(sdl_left);
        canvas.draw_point(sdl_right);
        canvas.draw_line(sdl_left, sdl_force_left);
        canvas.draw_line(sdl_right, sdl_force_right);

        // draw edge
        for wall_point in wall_points.windows(2) {
            let wall_start = wall_point[0];
            let wall_end = wall_point[1];
            let edge_start = sdl2_point(wall_start);
            let edge_end = sdl2_point(wall_end);
            canvas.draw_line(edge_start, edge_end);
        }

        canvas.present();

        throttler.wait();
    }
}
