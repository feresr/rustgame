extern crate engine;
extern crate nalgebra_glm as glm;

use bevy_ecs::prelude::*;
use engine::{
    graphics::{batch::*, common::*, material::*, shader::*, target::*, texture::*},
    Keyboard, Mouse,
};

#[derive(Component)]
struct Background {
    offset: f32,
    radius: f32,
    time: f32,
}

#[derive(Component)]
struct Paddle {
    x: f32,
    width: f32,
    height: f32,
}

#[derive(Component)]
struct Ball {
    x: f32,
    y: f32,
    r: f32,
    dx: f32,
    dy: f32,
}

#[derive(Component)]
struct Camera {
    target: glm::Vec3,
    pos: glm::Vec3,
    dir: glm::Vec3,
    right: glm::Vec3,
    up: glm::Vec3,
}

const VERTEX_SHADER_SOURCE: &str = "#version 330 core\n
            layout (location = 0) in vec3 aPos;\n
            layout (location = 1) in vec3 aColor;\n
            layout (location = 2) in vec2 aTexCoord;\n
            uniform mat4 u_matrix;\n
            out vec2 TexCoord;\n
            out vec3 a_color;\n
            void main()\n
            {\n
               gl_Position = u_matrix * vec4(aPos, 1.0);\n
               TexCoord = aTexCoord;
               a_color = aColor;
            }";
const FRAGMENT_SHADER_SOURCE_2: &str = "#version 330 core\n
            in vec2 TexCoord;\n
            out vec4 FragColor;\n
            uniform sampler2D u_texture;\n
            uniform ivec2 u_resolution;\n
            uniform float offset;
            uniform float radius;
            uniform float time;
            void main()\n
            {\n
                vec2 c = (2.0 * gl_FragCoord.xy - u_resolution.xy) / u_resolution.x; 
                c = c * 25.0;
                if (length(sin(c + vec2(offset,offset))) < radius) {
                    FragColor = vec4(0.8);
                } else {
                    FragColor = vec4(min(0.5, sin(time)), min(0.5, cos(time)), min(0.5, sin(time * 2.0)), 1.0);
                }
            }";

#[derive(Resource)]
struct Assets {
    textures: Vec<Texture>,
}

fn main() {
    engine::start(&|world, update, render| {
        let pos = glm::vec3(0.0, 0.0, 12.0);
        let target = glm::vec3(0.0, 0.0, 0.0);
        let dir = glm::normalize(&(target - pos));
        let up = glm::vec3(0.0, 1.0, 0.0);
        let right = glm::normalize(&(glm::cross(&up, &dir)));
        let camera_up = glm::cross(&dir, &right);
        world.spawn(Camera {
            target,
            pos,
            dir,
            right,
            up: camera_up,
        });

        world.spawn(Paddle {
            x: 0.0,
            width: 200.0,
            height: 20.0,
        });
        world.spawn(Ball {
            x: SCREEN.width as f32 / 2.0,
            y: SCREEN.height as f32 / 2.0,
            r: 10.0,
            dx: 2.0,
            dy: 6.0,
        });

        // Background Rect / shader
        world.spawn((Background {
            offset: 1.2,
            radius: 0.20,
            time: 0.0,
        },));

        let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2);
        world.insert_resource(Material::new(shader));
        world.insert_resource(TextureSampler::default());
        world.insert_non_send_resource(Target::new(200, 100, &[TextureFormat::RGBA]));

        update.add_systems((
            paddle_system,
            ball_system.after(paddle_system),
            background_animation,
            camera_system,
        ));
        render.add_systems((background_render, paddle_render, ball_render, render_system).chain());
    });
}
// THis is wrong, no item has both "query and paddle traits"
fn collision_system(mut query: Query<'_, '_, (&mut Ball, &Paddle)>) {
    println!("Checking collision -----------------");
    for (mut ball, paddle) in &mut query {}
}

fn rect_collision(a: &RectF, b: &RectF) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

fn ball_render(mut batch: NonSendMut<'_, Batch>, mut query: Query<'_, '_, &mut Ball>) {
    for ball in &mut query {
        batch.circle((ball.x, ball.y), ball.r, 32, (0.0, 1.0, 1.0));
    }
}
fn paddle_render(mut batch: NonSendMut<'_, Batch>, mut query: Query<'_, '_, (&mut Paddle)>) {
    for paddle in &mut query {
        // println!("Paddle x: {}", paddle.x);
        let rect = RectF {
            x: paddle.x - paddle.width / 2.0,
            y: 20.0,
            w: paddle.width,
            h: paddle.height,
        };
        batch.rect(&rect, (1.0, 0.0, 1.0));
    }
}

fn background_render(
    mut batch: NonSendMut<'_, Batch>,
    mut query: Query<'_, '_, &mut Background>,
    mut material: ResMut<'_, Material>,
    sampler: Res<'_, TextureSampler>,
) {
    // Render the background quad
    batch.set_sampler(&sampler);
    for mut bkg in &mut query {
        //     // todo: this material unifomr is overwritten (since the material is shared)
        material.set_valuef("offset", bkg.offset);
        material.set_valuef("radius", bkg.radius);
        material.set_valuef("time", bkg.time);
        bkg.time += 0.003;
        let rect1 = RectF {
            x: 0f32,
            y: 0f32,
            w: SCREEN.width as f32,
            h: SCREEN.height as f32,
        };

        batch.push_matrix(glm::Mat4::new_translation(&glm::vec3(0.0, 0.0, -0.1)));
        batch.push_material(&material);
        batch.rect(&rect1, (1.0, 1.0, 1.0));
        batch.pop_material();
        batch.pop_matrix();
    }
}

fn paddle_system(mut query: Query<'_, '_, &mut Paddle>, mouse: NonSend<'_, Mouse>) {
    for mut paddle in &mut query {
        paddle.x = mouse.positon.0 as f32;
    }
}
fn ball_system(mut query: Query<'_, '_, &mut Ball>, paddle: Query<'_, '_, &Paddle>) {
    for mut ball in &mut query {
        for paddle in &paddle {
            let paddle_rect = RectF {
                x: paddle.x - paddle.width / 2.0,
                y: 20.0,
                w: paddle.width,
                h: paddle.height,
            };
            let ball_rect = RectF {
                x: ball.x - ball.r,
                y: ball.y - ball.r,
                w: ball.r * 2.0,
                h: ball.r * 2.0,
            };
            if rect_collision(&paddle_rect, &ball_rect) {
                ball.dy = -ball.dy;
                let middle = paddle.x;
                println!("Ball x: {}, Paddle x: {}", ball.x, middle);
                if ball.x > middle {
                    println!("-MAS");
                    ball.dx += 2.0;
                } else {
                    println!("-MINUS");
                    ball.dx -= 2.0;
                }
            } else {
            }
        }

        if (ball.x + ball.r) > SCREEN.width as f32 || (ball.x - ball.r) < 0.0 {
            ball.dx = -ball.dx;
        }
        if (ball.y + ball.r) > SCREEN.height as f32 || (ball.y - ball.r) < 0.0 {
            ball.dy = -ball.dy;
        }
        ball.x += ball.dx;
        ball.y += ball.dy;
    }
}

fn background_animation(mut query: Query<'_, '_, &mut Background>) {
    for mut bkg in &mut query {
        bkg.offset += 0.01;
    }
}

// Actually calls render
// NonSend means (use main thread)
fn render_system(mut batch: NonSendMut<'_, Batch>, camera: Query<'_, '_, &Camera>) {
    let camera = camera.get_single().unwrap();

    // let ratio = (SCREEN.width as f32) / SCREEN.height as f32;
    let width = SCREEN.width as f32;
    let height = SCREEN.height as f32;

    let view = glm::look_at(&camera.pos, &(camera.pos + camera.dir), &camera.up);
    // Background is at z 0
    // Camera is at z 1 - Looking at 0
    let ortho: glm::Mat4 = glm::ortho(0.0, width, 0f32, height, 0.0f32, 2f32);
    let ortho = ortho * view;
    batch.render(&SCREEN, &ortho);
}

fn camera_system(
    mouse: NonSend<'_, Mouse>,
    keyboard: NonSend<'_, Keyboard>,
    mut camera: Query<'_, '_, &mut Camera>,
) {
    if !mouse.pressing {
        // let options = options.iter().next().unwrap();
        // update camera to match debug panel (options)
        let mut camera = camera.iter_mut().next().unwrap();
        camera.pos = glm::vec3(0f32, 0f32, 1f32);
        camera.target = glm::vec3(0f32, 0f32, 0f32);
        return;
    }

    let dx = mouse.change.0 as f32 / 10.0;
    let dy = mouse.change.1 as f32 / 10.0;
    let mut camera = camera.get_single_mut().unwrap();
    if keyboard.shift {
        let right = camera.right;
        camera.pos += right * dx * 0.1;
        camera.target += right * dx * 0.1;

        let top = camera.up;
        camera.pos += top * dy * 0.1;
        camera.target += top * dy * 0.1;
        return;
    }

    // update camera position and also debug panel to match.
    let distance_from_target = glm::length(&(camera.target - camera.pos));

    // move the camera in its `right` and `up` vector
    let right = camera.right;
    camera.pos += right * dx;
    let up = camera.up;
    camera.pos += up * dy;
    // The resulting position is not at the same distance from the target as before,
    // to fix this, get a unit vector from the target to the new camera pos.
    let normalized = glm::normalize(&(camera.pos - camera.target));
    // the new position is target + (unit vector pointing at new position) * original distance
    camera.pos = camera.target + normalized * distance_from_target;

    // recalculate camera direction right and up.
    camera.dir = glm::normalize(&(camera.target - camera.pos));
    let right = glm::normalize(&(glm::cross(&glm::vec3(0.0, 1.0, 0.0), &camera.dir)));
    camera.right = right;
    camera.up = glm::cross(&camera.dir, &camera.right);
}
