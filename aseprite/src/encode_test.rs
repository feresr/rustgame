use std::fs;
use aseprite_loader::binary::color_depth::ColorDepth;
use aseprite_loader::loader::AsepriteFile;
use crate::{sprite::Sprite, uncompress};

#[test]
fn encode_test() {
    // TODO:
    // Next steps, make it work with multiple layers
    // Make it work with RGBA (not only indexed)
    // Slices
    // Tags
    // Emit a debug png but also bake the png info into the final binary for consumption by the game engine

    // Parse ase file
    let asedata = fs::read("src/map.ase").unwrap();
    let aseprite = AsepriteFile::load(&asedata).unwrap();
    // let mut aseprite = Aseprite::new("src/map.ase");

    let output_png = fs::File::create("output.png").unwrap();

    // let png: Vec<u8> = vec![];
    // let cursor = Cursor::new(png);
    // let mut altas = Vec::new();
    let (packing_width, packing_height) = optimal_rectangle_pack(aseprite.frames.len());
    dbg!("{}-{}", packing_width, packing_height);


    let mut encoder = png::Encoder::new(
        output_png,
        aseprite.file.header.width as u32 * packing_width as u32,
        aseprite.file.header.height as u32 * packing_height as u32,
    );

    // Write the binary file
    Sprite::encode(&aseprite, packing_width, packing_height);

    // Now write the png
    // TODO: bake the png into the binary?

    // TODO check what the bbp is in the aseprite file
    let color = match aseprite.file.header.color_depth {
        ColorDepth::Rgba => png::ColorType::Rgba,
        ColorDepth::Grayscale => png::ColorType::Grayscale,
        ColorDepth::Indexed => png::ColorType::Indexed,
        ColorDepth::Unknown(error) => panic!("Unsupported color depth: {}", error),
    };

    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Best);

    // The file specification allows for multiple palettes per frame, but the editor does not, and neither do we.
    let palette = aseprite.file.palette.unwrap();

    let mut writer = encoder.write_header().unwrap();

    let mut packing_x = 0;
    let mut packing_y = packing_height - 1;

    // Write output file
    let mut all_image_data = vec![
        [0u8; 4];
        (aseprite.file.header.width * aseprite.file.header.height) as usize
            * aseprite.file.header.frames as usize
    ]
    .concat();
    for frame in aseprite.frames.iter() {
        for cel in &frame.cels {
            dbg!(&frame.duration);
            if let Some(image) = &mut aseprite.images.get(cel.image_index) {
                let image_data= image.data;
                let mut uncompressed = image_data.to_vec();
                if image.compressed {
                   uncompressed = uncompress(image_data);
                }
                match color {
                    png::ColorType::Indexed => {
                        // This has the frame size
                        let mut dst = vec![
                            [0u8; 4];
                            (aseprite.file.header.width * aseprite.file.header.height) as usize
                        ]
                        .concat();
                        let converted = uncompressed.iter().map(|pi| &palette.colors[*pi as usize]);
                        // This is a smaller image containing only the colored (no transparent) pixels
                        // It's usually smaller than the original image
                        let src: Vec<[u8; 4]> = converted
                            .map(|pe| [pe.red, pe.green, pe.blue, pe.alpha])
                            .collect();
                        let src = src.concat();

                        // Blit the image data (sprite - no transparent pixels) into the destination buffer (contains transparent pixels padding)
                        blit(
                            &src[..],
                            image.width as usize,
                            image.height as usize,
                            &mut dst[..],
                            aseprite.file.header.width as usize,
                            aseprite.file.header.height as usize,
                            cel.origin.0 as usize,
                            cel.origin.1 as usize,
                        );
                        // Paste frame image into final image atlas
                        dbg!(packing_y);
                        blit(
                            &dst[..],
                            aseprite.file.header.width as usize,
                            aseprite.file.header.height as usize,
                            &mut all_image_data[..],
                            (aseprite.file.header.width as u32 * packing_width as u32) as usize,
                            (aseprite.file.header.height as u32 * packing_height as u32) as usize,
                            (packing_x * aseprite.file.header.width as usize) as usize,
                            (packing_y * aseprite.file.header.height as usize) as usize,
                        );
                        if packing_x < packing_width - 1 {
                            packing_x += 1;
                        } else {
                            packing_x = 0;
                            packing_y = packing_y.saturating_sub(1);
                        }
                    }
                    png::ColorType::Rgba => {
                        all_image_data.extend_from_slice(&mut uncompressed);
                    }
                    _ => {
                        panic!("Unsupported color type")
                    }
                }
                // dbg!(&uncompressed);
                break;
            }
        }
    }

    writer.write_image_data(&all_image_data).unwrap();
    let result = writer.finish();
    if let Err(err) = &result {
        eprintln!("Error writing PNG file: {}", err);
    }
    assert!(result.is_ok());
    // output.write(&cursor.into_inner()).unwrap();
}

fn blit(
    src: &[u8],
    src_width: usize,
    src_height: usize,
    dst: &mut [u8],
    dst_width: usize,
    dst_height: usize,
    offset_x: usize,
    offset_y: usize,
) {
    let pixel_size = 4; // assuming RGB, change to 4 for RGBA

    for y in 0..src_height {
        for x in 0..src_width {
            let src_index = (y * src_width + x) * pixel_size;
            let dst_x = offset_x + x;
            let dst_y = offset_y + y;

            if dst_x < dst_width && dst_y < dst_height {
                let dst_index = (dst_y * dst_width + dst_x) * pixel_size;
                dst[dst_index..dst_index + pixel_size]
                    .copy_from_slice(&src[src_index..src_index + pixel_size]);
            }
        }
    }
}

fn optimal_rectangle_pack(n: usize) -> (usize, usize) {
    let mut best = (n, 1); // Start with the worst possible rectangle (n, 1)
    let mut min_area = n; // Start with the area of that worst rectangle

    for h in 1..n {
        let w = (n + h - 1) / h; // This is the same as ceil(n / h)
        let area = w * h;

        dbg!(&area);
        dbg!(&best.0);
        dbg!(&w);

        // Ensure the current configuration has a smaller area
        if area < min_area || (area == min_area && w < best.0) {
            best = (w, h);
            dbg!(&best);
            min_area = area;
        }
    }

    dbg!("returning");
    dbg!(&best);
    best
}
