use std::fs;

use crate::Aseprite;

#[test]
fn test_aseprite() {
    
    // TODO: 
    // Next steps, make it work with multiple layers
    // Make it work with RGBA (not only indexed)
    // Slices
    // Tags
    // Emit a debug png but also bake the png info into the final binary for consumption by the game engine
    
    // Parse ase file
    let mut aseprite = Aseprite::new("src/player.ase");
    let output = fs::File::create("output.png").unwrap();

    // let mut altas = Vec::new();
    let (packing_width, packing_height) = optimal_rectangle_pack(aseprite.frame_count as usize);
    dbg!("{}-{}", packing_width, packing_height);
    let mut encoder = png::Encoder::new(
        output,
        aseprite.header.width as u32 * packing_width as u32,
        aseprite.header.height as u32 * packing_height as u32,
    );

    // TODO check what the bbp is in the aseprite file
    //
    let color = match aseprite.header.color_depth {
        32 => png::ColorType::Rgba,
        16 => png::ColorType::Grayscale,
        8 => png::ColorType::Indexed,
        _ => panic!("Unsupported color depth"),
    };
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Best);

    // The file specification allows for multiple palettes per frame, but the editor does not, and neither do we.
    let frame = aseprite.frames.first_mut().unwrap();
    let palette = Vec::from(frame.palettes.first().unwrap().entries.clone());
    // for frame in aseprite.frames.iter_mut() {
    //     for entry in &frame.palettes.first().unwrap().entries {
    //         palette.push(entry.red);
    //         palette.push(entry.green);
    //         palette.push(entry.blue);
    //         // palette.push(entry.alpha);
    //     }
    //     encoder.set_palette(palette.clone());
    // }
    // dbg!(&palette);

    let mut writer = encoder.write_header().unwrap();
    
    let mut packing_x = 0;
    let mut packing_y = 0;

    // Write output file
    let mut all_image_data = vec![
        [0u8; 4];
        (aseprite.header.width * aseprite.header.height) as usize
            * aseprite.header.frames as usize
    ].concat();
    for frame in aseprite.frames.iter_mut() {
        for cel in &mut frame.cels {
            if let Some(image_data) = &mut cel.image_data {
                let mut uncompressed = image_data.uncompress();
                match color {
                    png::ColorType::Indexed => {
                        let mut dst = vec![
                            [0u8; 4];
                            (aseprite.header.width * aseprite.header.height) as usize
                        ]
                        .concat();
                        let converted = uncompressed.iter().map(|pi| &palette[*pi as usize]);
                        let src: Vec<[u8; 4]> = converted
                            .map(|pe| [pe.red, pe.green, pe.blue, pe.alpha])
                            .collect();
                        let src = src.concat();

                        blit(
                            &src[..],
                            image_data.width as usize,
                            image_data.height as usize,
                            &mut dst[..],
                            aseprite.header.width as usize,
                            aseprite.header.height as usize,
                            cel.position_x as usize,
                            cel.position_y as usize,
                        );
                        blit(
                            &dst[..],
                            aseprite.header.width as usize,
                            aseprite.header.height as usize,
                            &mut all_image_data[..],
                            (aseprite.header.width as u32 * packing_width as u32) as usize,
                            (aseprite.header.height as u32 * packing_height as u32) as usize,
                            (packing_x * aseprite.header.width as usize) as usize,
                            (packing_y * aseprite.header.height as usize) as usize,
                        );
                        if packing_x < packing_width {
                            packing_x += 1;
                        } else {
                            packing_x = 0;
                            packing_y += 1;
                        }
                    }
                    png::ColorType::Rgba => {
                        all_image_data.append(&mut uncompressed);
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
    let mut best = (n, 1);  // Start with the worst possible rectangle (n, 1)
    let mut min_area = n;   // Start with the area of that worst rectangle

    for h in 1..n {
        let w = (n + h - 1) / h;  // This is the same as ceil(n / h)
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
