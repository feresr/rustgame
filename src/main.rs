extern crate engine;
extern crate nalgebra_glm as glm;

use bevy_ecs::prelude::*;
use engine::{
    graphics::{batch::*, common::*, material::*, shader::*, target::*, texture::*},
    DebugOptions, Keyboard, Mouse,
};

#[derive(Component)]
struct Background {
    offset: f32,
    radius: f32,
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
            void main()\n
            {\n
                vec2 c = (2.0 * gl_FragCoord.xy - u_resolution.xy) / u_resolution.x; 
                c = c * 25.0;
                if (length(sin(c + vec2(offset,offset))) < radius) {
                    FragColor = vec4(0.3);
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

        // Background Rect / shader
        world.spawn((Background {
            offset: 1.2,
            radius: 0.20,
        },));

        let shader = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE_2);
        world.insert_resource(Material::new(shader));
        world.insert_resource(TextureSampler::default());
        world.insert_non_send_resource(Target::new(200, 100, &[TextureFormat::RGBA]));

        update.add_systems((background_animation, camera_system));
        render.add_systems((background_render, render_system).chain());
    });
}

fn background_render(
    mut batch: NonSendMut<'_, Batch>,
    mut query: Query<'_, '_, (&mut Background)>,
    mut material: ResMut<'_, Material>,
    sampler: Res<'_, TextureSampler>,
) {
    println!("DRAWING STUFF..");
    // Render the background quad
    batch.set_sampler(&sampler);
    for bkg in &mut query {
        //     // todo: this material unifomr is overwritten (since the material is shared)
        material.set_valuef("offset", bkg.offset);
        material.set_valuef("radius", bkg.radius);
        let rect1 = RectF {
            x: -40.0,
            y: -40.0,
            w: 80.0,
            h: 80.0,
        };

        batch.push_matrix(glm::Mat4::new_translation(&glm::vec3(0.0, 0.0, 1.0)));
        batch.push_material(&material);
        batch.rect(&rect1, (1.0, 1.0, 1.0));
        batch.pop_material();
        batch.pop_matrix();
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

    let ratio = (SCREEN.width as f32) / SCREEN.height as f32;
    let width = 5.0;
    let height = (width / ratio) / 2.0;

    let view = glm::look_at(&camera.pos, &(camera.pos + camera.dir), &camera.up);
    let ortho: glm::Mat4 = glm::ortho(-width / 2.0, width / 2.0, -height, height, -1f32, 1f32);
    let ortho = ortho * view;
    batch.render(&SCREEN, &ortho);
    batch.clear();
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
