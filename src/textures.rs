use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

const FACE_TEXTURE_SIZE: u32 = 64;
const FACE_BODY_LUMA: u8 = 228;
const FACE_EYE_LUMA: u8 = 255;

const VIGNETTE_TEXTURE_SIZE: u32 = 256;
const VIGNETTE_BORDER_PX: f32 = 40.0;
const VIGNETTE_INNER_RADIUS: f32 = 0.55;
const VIGNETTE_MAX_ALPHA: f32 = 0.14;

#[derive(Resource, Clone)]
pub(crate) struct GeneratedTextures {
    face: Handle<Image>,
    vignette: Handle<Image>,
}

impl FromWorld for GeneratedTextures {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        Self {
            face: images.add(create_face_texture()),
            vignette: images.add(create_vignette_texture()),
        }
    }
}

impl GeneratedTextures {
    pub(crate) fn face_sprite(&self, size: f32, color: Color) -> Sprite {
        Sprite {
            image: self.face.clone(),
            custom_size: Some(Vec2::splat(size)),
            color,
            ..default()
        }
    }

    pub(crate) fn vignette_node(&self) -> ImageNode {
        ImageNode::new(self.vignette.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(VIGNETTE_BORDER_PX),
            ..default()
        }))
    }
}

fn create_face_texture() -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: FACE_TEXTURE_SIZE,
            height: FACE_TEXTURE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[FACE_BODY_LUMA, FACE_BODY_LUMA, FACE_BODY_LUMA, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let size = FACE_TEXTURE_SIZE as f32;
    let eye_size = size * 0.12;
    let eye_x = size * 0.64;
    let top_eye_y = size * 0.26;
    let bottom_eye_y = size * 0.62;

    for y in 0..FACE_TEXTURE_SIZE {
        for x in 0..FACE_TEXTURE_SIZE {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            if (is_eye_pixel(px, py, eye_x, top_eye_y, eye_size)
                || is_eye_pixel(px, py, eye_x, bottom_eye_y, eye_size))
                && let Some(pixel) = image.pixel_bytes_mut(UVec3::new(x, y, 0))
            {
                pixel[0] = FACE_EYE_LUMA;
                pixel[1] = FACE_EYE_LUMA;
                pixel[2] = FACE_EYE_LUMA;
            }
        }
    }

    image
}

fn create_vignette_texture() -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: VIGNETTE_TEXTURE_SIZE,
            height: VIGNETTE_TEXTURE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let texture_size = VIGNETTE_TEXTURE_SIZE as f32;
    let half_diagonal = 2.0_f32.sqrt();

    for y in 0..VIGNETTE_TEXTURE_SIZE {
        for x in 0..VIGNETTE_TEXTURE_SIZE {
            let normalized_x = ((x as f32 + 0.5) / texture_size) * 2.0 - 1.0;
            let normalized_y = ((y as f32 + 0.5) / texture_size) * 2.0 - 1.0;
            let distance = Vec2::new(normalized_x, normalized_y).length() / half_diagonal;
            let edge_factor = ((distance - VIGNETTE_INNER_RADIUS) / (1.0 - VIGNETTE_INNER_RADIUS))
                .clamp(0.0, 1.0);
            let eased_edge = EaseFunction::SmootherStep.sample_clamped(edge_factor);
            let alpha = eased_edge * VIGNETTE_MAX_ALPHA;

            if let Some(pixel) = image.pixel_bytes_mut(UVec3::new(x, y, 0)) {
                pixel[3] = (alpha * u8::MAX as f32) as u8;
            }
        }
    }

    image
}

fn is_eye_pixel(px: f32, py: f32, min_x: f32, min_y: f32, size: f32) -> bool {
    px >= min_x && px < min_x + size && py >= min_y && py < min_y + size
}
