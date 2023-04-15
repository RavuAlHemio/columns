mod model;
mod seg_display;


use std::collections::{BTreeSet, VecDeque};
use std::env;
use std::thread::sleep;
use std::time::Duration;

use rand::SeedableRng;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureAccess, TextureCreator};
use sdl2::video::Window;

use crate::model::{Block, BlockState, Field, FieldBlock};
use crate::seg_display::SegmentedDisplay;


const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const BLOCK_WIDTH_PX: u32 = 25;
const BLOCK_HEIGHT_PX: u32 = 25;
const FIELD_WIDTH_BLOCKS: u32 = 6;
const FIELD_HEIGHT_BLOCKS: u32 = 18;
const FIELD_OFFSET_TOP_PX: i32 = 50;
const FIELD_OFFSET_LEFT_PX: i32 = 325;
const FIELD_FRAME_OFFSET_PX: i32 = 2;
const BLOCK_COLOR_COUNT: usize = 6;
const BLOCK_CENTER_OFFSET: u32 = 5;
const MINIMUM_SEQUENCE: usize = 3;
const DISAPPEAR_BLINK_COUNT: usize = 32;
const PAUSE_BAR_WIDTH: u32 = 85;
const PAUSE_BAR_HEIGHT: u32 = 256;
const SCORE_OFFSET_LEFT_PX: i32 = 500;
const COLOR_STATS_BARS_LEFT_PX: i32 = 500;
const COLOR_STATS_BAR_WIDTH: u32 = 8;
const COLOR_STATS_BAR_SPACING: u32 = 2;
const DEFAULT_BLOCK_FALL_LIMIT: u64 = 32;
const SCORE_SPEEDUP_DIVISOR: u64 = 4;

const FIELD_BLOCK_COUNT: usize = (FIELD_WIDTH_BLOCKS * FIELD_HEIGHT_BLOCKS) as usize;
const NEW_BLOCK_COLUMN: u32 = FIELD_WIDTH_BLOCKS / 2;


const BLOCK_COLORS: [Color; BLOCK_COLOR_COUNT] = [
    Color::RED, Color::GREEN, Color::BLUE,
    Color::YELLOW, Color::CYAN, Color::MAGENTA,
];


#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum GameState {
    #[default] Play,
    Pause,
    Over,
}

const fn mul_div(val: u8, numerator: u8, denominator: u8) -> u8 {
    ((val as u16) * (numerator as u16) / (denominator as u16)) as u8
}
const fn mul_div_rgb(color: Color, numerator: u8, denominator: u8) -> Color {
    Color::RGB(
        mul_div(color.r, numerator, denominator),
        mul_div(color.g, numerator, denominator),
        mul_div(color.b, numerator, denominator),
    )
}
const fn brighten_rgb(color: Color, divisor: u8) -> Color {
    let base = mul_div(255, divisor-1, divisor);
    Color::RGB(
        base + mul_div(color.r, 1, divisor),
        base + mul_div(color.g, 1, divisor),
        base + mul_div(color.b, 1, divisor),
    )
}


fn draw(
    canvas: &mut Canvas<Window>,
    field: &Field,
    game_state: GameState,
    score: u64,
    color_stats: &[u32; BLOCK_COLOR_COUNT],
    block_textures: &[Texture],
) {
    canvas.set_draw_color((0, 0, 0));
    canvas.clear();

    canvas.set_draw_color((0xC0, 0xC0, 0xC0));
    canvas.draw_rect(Rect::new(
        FIELD_OFFSET_LEFT_PX - FIELD_FRAME_OFFSET_PX,
        FIELD_OFFSET_TOP_PX - FIELD_FRAME_OFFSET_PX,
        BLOCK_WIDTH_PX * FIELD_WIDTH_BLOCKS + u32::try_from(2*FIELD_FRAME_OFFSET_PX).unwrap(),
        BLOCK_HEIGHT_PX * FIELD_HEIGHT_BLOCKS + u32::try_from(2*FIELD_FRAME_OFFSET_PX).unwrap(),
    )).unwrap();

    let blocks_and_coords = field.blocks().iter().zip(Field::coords());
    for (field_block, (x, y)) in blocks_and_coords {
        if let FieldBlock::Block(block) = field_block {
            let base_color_index = usize::from(block.color_index);
            let color_index = if let Some(counter) = block.state.disappearing_counter() {
                if (counter & (1 << 3)) == 0 {
                    base_color_index
                } else {
                    base_color_index + BLOCK_COLOR_COUNT
                }
            } else {
                base_color_index
            };

            let actual_x = FIELD_OFFSET_LEFT_PX + i32::try_from(x * BLOCK_WIDTH_PX).unwrap();
            let actual_y = FIELD_OFFSET_TOP_PX + i32::try_from(y * BLOCK_HEIGHT_PX).unwrap();
            canvas.copy(
                &block_textures[color_index],
                None,
                Rect::new(
                    actual_x,
                    actual_y,
                    BLOCK_WIDTH_PX,
                    BLOCK_HEIGHT_PX,
                ),
            ).unwrap();
        }
    }

    // draw score
    let mut my_score = score;
    let mut score_digits = [0u8; 4];
    for i in (0..score_digits.len()).rev() {
        score_digits[i] = u8::try_from(my_score % 10).unwrap();
        my_score /= 10;
    }
    let segs = score_digits.iter()
        .enumerate()
        .map(|(i, &dig)| SegmentedDisplay::new(
            SCORE_OFFSET_LEFT_PX + i32::try_from(i).unwrap() * crate::seg_display::DIGIT_OFFSET,
            FIELD_OFFSET_TOP_PX,
            Color::RGB(0x00, 0x7F, 0x00),
            dig,
        ));
    for seg in segs {
        seg.draw(canvas);
    }

    // draw color stats
    for (i, &color_count) in color_stats.iter().enumerate() {
        if color_count == 0 {
            continue;
        }

        let x = COLOR_STATS_BARS_LEFT_PX + i32::try_from(i).unwrap() * i32::try_from(COLOR_STATS_BAR_WIDTH + COLOR_STATS_BAR_SPACING).unwrap();
        let y = FIELD_OFFSET_TOP_PX + i32::try_from(FIELD_HEIGHT_BLOCKS * BLOCK_HEIGHT_PX - color_count).unwrap();

        canvas.set_draw_color(BLOCK_COLORS[i]);
        canvas.fill_rect(Rect::new(x, y, COLOR_STATS_BAR_WIDTH, color_count)).unwrap();
    }

    match game_state {
        GameState::Play => {},
        GameState::Pause => {
            // draw two parallel vertical rectangles to indicate pause
            let total_width = PAUSE_BAR_WIDTH * 3;
            let x1: i32 = ((WINDOW_WIDTH - total_width) / 2).try_into().unwrap();
            let x2 = x1 + 2*i32::try_from(PAUSE_BAR_WIDTH).unwrap();
            let y: i32 = ((WINDOW_HEIGHT - PAUSE_BAR_HEIGHT) / 2).try_into().unwrap();

            let mut translucent_gray = Color::GRAY;
            translucent_gray.a = 0xCC;
            canvas.set_draw_color(translucent_gray);
            canvas.fill_rect(Rect::new(x1, y, PAUSE_BAR_WIDTH, PAUSE_BAR_HEIGHT)).unwrap();
            canvas.fill_rect(Rect::new(x2, y, PAUSE_BAR_WIDTH, PAUSE_BAR_HEIGHT)).unwrap();
        },
        GameState::Over => {
            // draw a square to indicate game over
            let total_width = PAUSE_BAR_WIDTH * 3;
            let x: i32 = ((WINDOW_WIDTH - total_width) / 2).try_into().unwrap();
            let y: i32 = ((WINDOW_HEIGHT - PAUSE_BAR_HEIGHT) / 2).try_into().unwrap();

            let mut translucent_gray = Color::GRAY;
            translucent_gray.a = 0xCC;
            canvas.set_draw_color(translucent_gray);
            canvas.fill_rect(Rect::new(x, y, total_width, PAUSE_BAR_HEIGHT)).unwrap();
        },
    }

    canvas.present();
}


fn make_block_textures<'a, T>(creator: &'a TextureCreator<T>) -> Vec<Texture<'a>> {
    let mut ret = Vec::with_capacity(2*BLOCK_COLOR_COUNT);
    for brightened in [false, true] {
        for base_color in BLOCK_COLORS {
            let color = if brightened {
                brighten_rgb(base_color, 8)
            } else {
                base_color
            };

            let mid_color = mul_div_rgb(color, 4, 6);
            let dark_color = mul_div_rgb(color, 3, 6);
            let pixel_count: usize = (BLOCK_WIDTH_PX * BLOCK_HEIGHT_PX)
                .try_into().unwrap();
            let width_usize: usize = BLOCK_WIDTH_PX.try_into().unwrap();

            // start texture with dark color
            // dddddddd
            // dddddddd
            // dddddddd
            // dddddddd
            let mut texture_colors = vec![dark_color; pixel_count];

            // change top triangle to light color
            // llllllll
            // lllllddd
            // lllddddd
            // lddddddd
            for y_u32 in 0..BLOCK_HEIGHT_PX {
                let y: usize = y_u32.try_into().unwrap();

                let end_px = BLOCK_WIDTH_PX - (y_u32 * BLOCK_WIDTH_PX / BLOCK_HEIGHT_PX);
                for x_u32 in 0..end_px {
                    let x: usize = x_u32.try_into().unwrap();
                    texture_colors[y*width_usize + x] = color;
                }
            }

            // place mid-color square around middle
            // llllllll
            // llmmmmdd
            // llmmmmdd
            // lddddddd
            for y_u32 in BLOCK_CENTER_OFFSET..(BLOCK_HEIGHT_PX-BLOCK_CENTER_OFFSET) {
                let y: usize = y_u32.try_into().unwrap();

                for x_u32 in BLOCK_CENTER_OFFSET..(BLOCK_WIDTH_PX-BLOCK_CENTER_OFFSET) {
                    let x: usize = x_u32.try_into().unwrap();
                    texture_colors[y*width_usize + x] = mid_color;
                }
            }

            // squeeze into texture
            let mut texture_data = Vec::with_capacity(texture_colors.len() * 4);
            for color in texture_colors {
                texture_data.push(color.r);
                texture_data.push(color.g);
                texture_data.push(color.b);
                texture_data.push(color.a);
            }

            let mut texture = creator.create_texture(
                Some(PixelFormatEnum::ABGR8888),
                TextureAccess::Static,
                BLOCK_WIDTH_PX, BLOCK_HEIGHT_PX,
            ).unwrap();
            texture.update(
                Rect::new(0, 0, BLOCK_WIDTH_PX, BLOCK_HEIGHT_PX),
                &texture_data,
                (BLOCK_WIDTH_PX * 4).try_into().unwrap(),
            ).unwrap();
            ret.push(texture);
        }
    }
    ret
}


fn sequence_continues(field: &Field, x: u32, y: u32, dx: i32, dy: i32) -> Option<(u32, u32)> {
    let this_color = match field.block_by_coord(x, y).as_block() {
        Some(block) => block.color_index,
        None => return None,
    };

    let next_x = i32::try_from(x).unwrap() + dx;
    let next_y = i32::try_from(y).unwrap() + dy;

    if next_x < 0 {
        return None;
    }
    if next_x >= FIELD_WIDTH_BLOCKS.try_into().unwrap() {
        return None;
    }

    if next_y < 0 {
        return None;
    }
    if next_y >= FIELD_HEIGHT_BLOCKS.try_into().unwrap() {
        return None;
    }

    let x2 = u32::try_from(next_x).unwrap();
    let y2 = u32::try_from(next_y).unwrap();

    let neighbor_color = match field.block_by_coord(x2, y2).as_block() {
        Some(block) => block.color_index,
        None => return None,
    };

    if this_color == neighbor_color {
        Some((x2, y2))
    } else {
        None
    }
}


fn find_sequence(field: &Field, x: u32, y: u32, dx: i32, dy: i32) -> Vec<(u32, u32)> {
    assert!(dx != 0 || dy != 0);
    assert!(x < FIELD_WIDTH_BLOCKS && y < FIELD_HEIGHT_BLOCKS);

    let mut ret = Vec::new();
    if field.block_by_coord(x, y).as_block().is_none() {
        return ret; // no block here
    };
    ret.push((x, y));
    loop {
        let (last_x, last_y) = *ret.last().unwrap();
        if let Some((x2, y2)) = sequence_continues(field, last_x, last_y, dx, dy) {
            ret.push((x2, y2));
        } else {
            break;
        }
    }

    ret
}


fn get_coordinates_of_sequences(field: &Field) -> Vec<Vec<(u32, u32)>> {
    let settled_blocks = field.block_coords_with_predicate(|bs| bs.is_stationary());

    let mut sequences = Vec::with_capacity(4);
    for &(x, y) in &settled_blocks {
        // when looking for new sequences, we only look in four directions;
        // to ensure we don't count a sequence multiple times, we ensure there isn't a sequence in
        // the other direction as well
        if sequence_continues(field, x, y, -1, 0).is_none() { // left
            sequences.push(find_sequence(field, x, y, 1, 0)); // right
        }
        if sequence_continues(field, x, y, -1, -1).is_none() { // up-left
            sequences.push(find_sequence(field, x, y, 1, 1)); // down-right
        }
        if sequence_continues(field, x, y, 0, -1).is_none() { // up
            sequences.push(find_sequence(field, x, y, 0, 1)); // down
        }
        if sequence_continues(field, x, y, 1, -1).is_none() { // up-right
            sequences.push(find_sequence(field, x, y, -1, 1)); // down-left
        }

        // ensure our sequences are long enough
        sequences.retain(|seq| seq.len() >= MINIMUM_SEQUENCE);
    }

    sequences
}


fn handle_gravity_blocks(field: &mut Field, gravity_block_coords: &[(u32, u32)]) {
    for &(x, y) in gravity_block_coords {
        if field.block_at_coord_hit_bottom_or_stationary_block(x, y) {
            // we are no longer being pulled by gravity
            // mark this block as stationary
            field.block_by_coord_mut(x, y)
                .as_block_mut().unwrap()
                .state = BlockState::Stationary;
        } else {
            // drop this block by 1
            let this_block = field.block_by_coord(x, y);
            *field.block_by_coord_mut(x, y + 1) = this_block.clone();
            *field.block_by_coord_mut(x, y) = FieldBlock::Background;
        }
    }
}


fn handle_sequences(field: &mut Field, score: &mut u64) -> bool {
    // find sequences
    let sequences = get_coordinates_of_sequences(&field);
    if sequences.len() == 0 {
        return false;
    }

    for sequence in &sequences {
        *score += u64::try_from(sequence.len() - (MINIMUM_SEQUENCE - 1)).unwrap();
    }

    // mark blocks from sequences as disappearing
    let sequence_coords: BTreeSet<(u32, u32)> = sequences
        .iter()
        .flat_map(|seq| seq)
        .map(|(x, y)| (*x, *y))
        .collect();
    for &(x, y) in &sequence_coords {
        field.block_by_coord_mut(x, y)
            .as_block_mut().unwrap()
            .state = BlockState::Disappearing(DISAPPEAR_BLINK_COUNT);
    }

    true
}


fn handle_disappearing_blocks(field: &mut Field, disappearing_block_coords: &[(u32, u32)]) {
    for &(x, y) in disappearing_block_coords {
        let current_count = match field.block_by_coord(x, y).as_block() {
            Some(b) => match b.state.disappearing_counter() {
                Some(dc) => dc,
                None => continue,
            },
            None => continue,
        };
        if current_count > 0 {
            // reduce count by 1
            field.block_by_coord_mut(x, y)
                .as_block_mut().unwrap()
                .state = BlockState::Disappearing(current_count - 1);
        } else {
            // disappear the block completely
            *field.block_by_coord_mut(x, y) = FieldBlock::Background;

            // mark all blocks above as pulled-by-gravity unless they are also disappearing
            for above_y in 0..y {
                if let Some(block) = field.block_by_coord_mut(x, above_y).as_block_mut() {
                    if !block.state.is_disappearing() {
                        block.state = BlockState::Gravity;
                    }
                }
            }
        }
    }
}


fn make_new_descending_block(
    field: &mut Field,
    color_distribution: &Uniform<u8>,
    rng: &mut StdRng,
    color_stats: &mut [u32; BLOCK_COLOR_COUNT],
) -> bool {
    // is there even space?
    let has_space_for_new_block =
        field.block_by_coord(NEW_BLOCK_COLUMN, 0).is_background()
        && field.block_by_coord(NEW_BLOCK_COLUMN, 1).is_background()
        && field.block_by_coord(NEW_BLOCK_COLUMN, 2).is_background()
    ;
    if !has_space_for_new_block {
        false
    } else {
        // pick out three colors at random
        let color0 = color_distribution.sample(rng);
        let color1 = color_distribution.sample(rng);
        let color2 = color_distribution.sample(rng);

        color_stats[usize::from(color0)] += 1;
        color_stats[usize::from(color1)] += 1;
        color_stats[usize::from(color2)] += 1;

        *field.block_by_coord_mut(NEW_BLOCK_COLUMN, 0) = FieldBlock::Block(Block {
            color_index: color0,
            state: BlockState::Descending,
        });
        *field.block_by_coord_mut(NEW_BLOCK_COLUMN, 1) = FieldBlock::Block(Block {
            color_index: color1,
            state: BlockState::Descending,
        });
        *field.block_by_coord_mut(NEW_BLOCK_COLUMN, 2) = FieldBlock::Block(Block {
            color_index: color2,
            state: BlockState::Descending,
        });
        true
    }
}


fn handle_descending_blocks(field: &mut Field, descending_block_coords: &[(u32, u32)]) {
    for &(x, y) in descending_block_coords {
        let this_block = field.block_by_coord(x, y);

        if field.block_at_coord_hit_bottom_or_stationary_block(x, y) {
            // we are no longer descending
            field.block_by_coord_mut(x, y)
                .as_block_mut().unwrap()
                .state = BlockState::Stationary;
        } else {
            *field.block_by_coord_mut(x, y + 1) = this_block.clone();
            *field.block_by_coord_mut(x, y) = FieldBlock::Background;
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut rng = if args.len() > 1 {
        let seed_integer: u128 = args[1].parse()
            .expect("failed to parse RNG seed");
        let mut rng_seed = [0u8; 32];
        rng_seed[0..128/8].copy_from_slice(&seed_integer.to_be_bytes());
        StdRng::from_seed(rng_seed)
    } else {
        StdRng::from_entropy()
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Columns", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let color_distribution = Uniform::new(0, u8::try_from(BLOCK_COLOR_COUNT).unwrap());
    let mut color_stats = [0u32; BLOCK_COLOR_COUNT];
    let mut block_fall_counter = 0;
    let mut block_fall_limit = DEFAULT_BLOCK_FALL_LIMIT;

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_blend_mode(BlendMode::Blend);
    let texture_maker = canvas.texture_creator();
    let block_textures = make_block_textures(&texture_maker);

    let mut field = Field::new();
    let mut game_state = GameState::Play;
    let mut score = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'main_loop: loop {
        let mut force_draw = false;

        // handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                Event::KeyDown { keycode: Some(kc), .. } => {
                    match kc {
                        Keycode::Escape => break 'main_loop,
                        Keycode::Left => if game_state == GameState::Play {
                            // try moving falling blocks left
                            let descending_block_coords = field
                                .block_coords_with_predicate(|bs| bs.is_descending());
                            let can_move = descending_block_coords.iter()
                                .all(|&(x, y)|
                                    x > 0
                                    && field.block_by_coord(x - 1, y).is_background()
                                );
                            if can_move {
                                for (x, y) in descending_block_coords {
                                    *field.block_by_coord_mut(x - 1, y) = field.block_by_coord(x, y).clone();
                                    *field.block_by_coord_mut(x, y) = FieldBlock::Background;
                                }
                            }
                        },
                        Keycode::Right => if game_state == GameState::Play {
                            // try moving falling blocks right
                            let descending_block_coords = field
                                .block_coords_with_predicate(|bs| bs.is_descending());
                            let can_move = descending_block_coords.iter()
                                .all(|&(x, y)|
                                    x < FIELD_WIDTH_BLOCKS - 1
                                    && field.block_by_coord(x + 1, y).is_background()
                                );
                            if can_move {
                                for (x, y) in descending_block_coords {
                                    *field.block_by_coord_mut(x + 1, y) = field.block_by_coord(x, y).clone();
                                    *field.block_by_coord_mut(x, y) = FieldBlock::Background;
                                }
                            }
                        },
                        Keycode::Up => if game_state == GameState::Play {
                            // cycle through colors
                            let descending_block_coords = field
                                .block_coords_with_predicate(|bs| bs.is_descending());
                            let mut queue = VecDeque::with_capacity(descending_block_coords.len());
                            for &(x, y) in &descending_block_coords {
                                queue.push_back(
                                    field.block_by_coord(x, y)
                                        .as_block().unwrap()
                                        .color_index
                                );
                            }
                            if let Some(first_color) = queue.pop_front() {
                                queue.push_back(first_color);
                            }
                            for (&(x, y), &new_color) in descending_block_coords.iter().zip(queue.iter()) {
                                field.block_by_coord_mut(x, y)
                                    .as_block_mut().unwrap()
                                    .color_index = new_color;
                            }
                        },
                        Keycode::Down => if game_state == GameState::Play {
                            // hand over descending blocks to gravity
                            let descending_block_coords = field
                                .block_coords_with_predicate(|bs| bs.is_descending());
                            for &(x, y) in descending_block_coords.iter() {
                                field.block_by_coord_mut(x, y)
                                    .as_block_mut().unwrap()
                                    .state = BlockState::Gravity;
                            }
                        },
                        Keycode::F2 => {
                            // restart game
                            for field_block in field.blocks_mut() {
                                *field_block = FieldBlock::Background;
                            }
                            for color_stat in &mut color_stats {
                                *color_stat = 0;
                            }
                            score = 0;
                            block_fall_limit = DEFAULT_BLOCK_FALL_LIMIT;
                            game_state = GameState::Play;
                        },
                        Keycode::F3 => {
                            // pause/unpause
                            game_state = match game_state {
                                GameState::Over => GameState::Over,
                                GameState::Play => GameState::Pause,
                                GameState::Pause => GameState::Play,
                            };

                            // force redraw to make sure the "pause" symbol appears
                            force_draw = true;
                        },
                        _ => {},
                    }
                },
                Event::Window { win_event: WindowEvent::Exposed, .. } => {
                    force_draw = true;
                },
                _ => {},
            }
        }

        if game_state == GameState::Play {
            let disappearing_block_coords = field
                .block_coords_with_predicate(|bs| bs.is_disappearing());
            if disappearing_block_coords.len() > 0 {
                // count down
                handle_disappearing_blocks(&mut field, &disappearing_block_coords);

                // continue immediately
                block_fall_counter = block_fall_limit;
            } else {
                let gravity_block_coords = field
                    .block_coords_with_predicate(|bs| bs.is_pulled_by_gravity());
                if gravity_block_coords.len() > 0 {
                    handle_gravity_blocks(&mut field, &gravity_block_coords);

                    // continue immediately
                    block_fall_counter = block_fall_limit;
                } else {
                    if block_fall_counter == block_fall_limit {
                        // handle descending blocks
                        block_fall_counter = 0;

                        let descending_block_coords = field
                            .block_coords_with_predicate(|bs| bs.is_descending());
                        handle_descending_blocks(&mut field, &descending_block_coords);

                        if descending_block_coords.len() == 0 {
                            // no more descending blocks

                            // any sequences?
                            let old_score_divided = score / SCORE_SPEEDUP_DIVISOR;
                            let sequences_found = handle_sequences(&mut field, &mut score);
                            if sequences_found {
                                if block_fall_limit > 1 {
                                    let new_score_divided = score / SCORE_SPEEDUP_DIVISOR;
                                    if new_score_divided > old_score_divided {
                                        // increase speed by lowering the limit
                                        block_fall_limit -= 1;
                                    }
                                }

                                // continue immediately
                                block_fall_counter = block_fall_limit - 1;
                            } else {
                                if !make_new_descending_block(&mut field, &color_distribution, &mut rng, &mut color_stats) {
                                    // GAME OVER
                                    game_state = GameState::Over;

                                    // force redraw to make sure the "stop" symbol appears
                                    force_draw = true;
                                }
                            }
                        }
                    }
                    block_fall_counter += 1;
                }
            }
        }

        if game_state == GameState::Play || force_draw {
            draw(&mut canvas, &field, game_state, score, &color_stats, &block_textures);
            canvas.present();
        }

        sleep(Duration::new(0, 1_000_000_000 / 60))
    }
}
