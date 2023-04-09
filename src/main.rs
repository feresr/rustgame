extern crate engine;
extern crate nalgebra_glm as glm;

use bevy_ecs::prelude::*;
use engine::{
    graphics::{batch::*, common::*, material::*, shader::*, target::*, texture::*},
    Keyboard, Mouse, DebugOptions,
};

#[derive(Component)]
struct Circle {}
#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
    r: f32,
}
#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
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
            uniform float time;
            void main()\n
            {\n
                vec2 c = (2.0 * gl_FragCoord.xy - u_resolution.xy) / u_resolution.x; 
                c = c * 25.0;
                if (length(sin(c + vec2(time,time))) < 0.20) {
                    FragColor = vec4(0.2);
                } else {
                    FragColor = vec4(0.05);
                }
            }";

#[derive(Resource)]
struct Assets {
    textures: Vec<Texture>,
}

fn main() {
    engine::start(&|world, update, render| {
        world.spawn((
            Position {
                x: 0.0,
                y: 0.0,
                r: 1.2,
            },
            Velocity {
                x: 0.0001,
                y: 0.00005,
            },
            Circle {},
        ));

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

        world.spawn(DebugOptions {
            camera_pos: [0.0, 0.0, 12.0],
            camera_target: [0.0, 0.0, 0.0],
            cube_size: 1.0,
            perspective: false,
            fov: 0.5,
            pause: false,
            render_background: true,
            render_cube_1: true,
            render_cube_2: true,
        });

        let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2);
        let mut material = Material::new(shader);
        let mut sampler = TextureSampler::default();
        sampler.filter = TextureFilter::Linear;

        let mut path = std::env::current_exe().unwrap();
        path.pop(); // removes executable name from path
        let path = path.display().to_string();
        let texture = Texture::from_path(String::from(path.clone() + "/brick.png").as_str());
        let texture2 = Texture::from_path(String::from(path + "/coin.png").as_str());
        let assets = Assets {
            textures: vec![texture, texture2],
        };

        material.set_texture("u_mytex", &assets.textures[0]);
        material.set_sampler("u_mytex", &sampler);

        world.insert_resource(assets);
        world.insert_resource(material);
        world.insert_resource(sampler);
        world.insert_non_send_resource(Target::new(200, 100, &[TextureFormat::RGBA]));

        update.add_system(updating);
        update.add_system(camera_system);
        update.add_system(render_system);
        render.add_system(rendering);
    });
}

// This system moves each entity with a Position and Velocity component
// NonSend means (use main thread)
fn rendering(
    mut batch: NonSendMut<'_, Batch>,
    mut query: Query<'_, '_, (&mut Position, &Circle)>,
    mut options: Query<'_, '_, &DebugOptions>,
    mut material: ResMut<'_, Material>,
    sampler: Res<'_, TextureSampler>,
    assets: Res<'_, Assets>,
) {
    batch.set_sampler(&sampler);
    let mut cube_size = 0.0;

    for slider in &mut options {
        cube_size = slider.cube_size;
    }

    let options = options.single();

    for (position, _) in &mut query {
        //     // todo: this material unifomr is overwritten (since the material is shared)
        material.set_valuef("time", position.r);
        let rect1 = RectF {
            x: -20.0,
            y: -20.0,
            w: 40.0,
            h: 40.0,
        };

        // Draw background
        if options.render_background {
            batch.push_matrix(glm::Mat4::new_translation(&glm::vec3(0.0, 0.0, -4.0)));
            batch.push_material(&material);
            batch.rect(&rect1, (0.0, 1.0, 1.0));
            batch.pop_material();
            batch.pop_matrix();
        }

        let mat = glm::Mat4::identity();
        let rot = glm::Mat4::from_scaled_axis(&glm::vec3(1.2, 1.0, 0.0) * position.r);
        let mat = rot * mat;
        if options.render_cube_1 {
            batch.push_matrix(mat);
            batch.set_texture(&assets.textures[1]);
            let c = 0.4 + (position.r * 5.0).cos() * 0.1;
            batch.cube((0.0, 0.0), cube_size, (c, c, c));
            // batch.circle((0.0, 0.0), 0.9, 38, (0.5, 0.1, 0.9));
            batch.pop_matrix();
        }

        if options.render_cube_2 {
            let mat = mat.append_translation(&glm::vec3(3.0, 2.0, 0.0));
            batch.push_matrix(mat);
            batch.set_texture(&assets.textures[0]);
            batch.cube((0.0, 0.0), cube_size, (0.0, 0.0, 0.0));
            batch.circle((0.0, 0.0), 0.9, 38, (0.5, 0.1, 0.9));
            batch.pop_matrix();
        }
    }
}

fn updating(
    mut query: Query<'_, '_, (&mut Position, &Velocity)>,
    options: Query<'_, '_, &DebugOptions>,
) {
    let options = options.get_single().unwrap();
    if options.pause {
        return;
    }
    for (mut position, velocity) in &mut query {
        position.r += 0.01 + velocity.x;
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn render_system(
    mut batch: NonSendMut<'_, Batch>,
    camera: Query<'_, '_, &Camera>,
    options: Query<'_, '_, &DebugOptions>,
) {
    let options = options.get_single().unwrap();
    let camera = camera.get_single().unwrap();

    let ratio = (SCREEN.width as f32) / SCREEN.height as f32;
    let width = 5.0;
    let height = (width / ratio) / 2.0;

    let view = glm::look_at(&camera.pos, &(camera.pos + camera.dir), &camera.up);
    if options.perspective {
        let perspective = glm::perspective(ratio, options.fov, 0.1, 100.0);
        let perspective = perspective * view;
        batch.render(&SCREEN, &perspective);
    } else {
        let ortho: glm::Mat4 = glm::ortho(-width / 2.0, width / 2.0, -height, height, 0.1, 100.0);
        let ortho = ortho * view;
        batch.render(&SCREEN, &ortho);
    }
    batch.clear();
}

fn camera_system(
    mouse: NonSend<'_, Mouse>,
    keyboard: NonSend<'_, Keyboard>,
    mut camera: Query<'_, '_, &mut Camera>,
    mut options: Query<'_, '_, &mut DebugOptions>,
) {
    if !mouse.pressing {
        let options = options.iter().next().unwrap();
        // update camera to match debug panel (options)
        let mut camera = camera.iter_mut().next().unwrap();
        camera.pos = glm::vec3(
            options.camera_pos[0],
            options.camera_pos[1],
            options.camera_pos[2],
        );
        camera.target = glm::vec3(
            options.camera_target[0],
            options.camera_target[1],
            options.camera_target[2],
        );
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

        let mut options = options.get_single_mut().unwrap();
        options.camera_pos = camera.pos.try_into().unwrap();
        options.camera_target = camera.target.try_into().unwrap();
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

    let mut options = options.get_single_mut().unwrap();
    options.camera_pos = camera.pos.try_into().unwrap();
}
