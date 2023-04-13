mod model;


use std::thread::sleep;
use std::time::Duration;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::model::FieldBlock;


const BLOCK_WIDTH_PX: u32 = 25;
const BLOCK_HEIGHT_PX: u32 = 25;
const FIELD_WIDTH_BLOCKS: u32 = 6;
const FIELD_HEIGHT_BLOCKS: u32 = 18;
const OFFSET_TOP_PX: i32 = 50;
const OFFSET_LEFT_PX: i32 = 325;
const BLOCK_COLOR_COUNT: usize = 6;

const FIELD_BLOCK_COUNT: usize = (FIELD_WIDTH_BLOCKS * FIELD_HEIGHT_BLOCKS) as usize;


const BLOCK_COLORS: [Color; BLOCK_COLOR_COUNT] = [
    Color::RED, Color::GREEN, Color::BLUE,
    Color::YELLOW, Color::CYAN, Color::MAGENTA,
];
const BRIGHT_COLORS: [Color; BLOCK_COLOR_COUNT] = [
    brighten_rgb(Color::RED), brighten_rgb(Color::GREEN), brighten_rgb(Color::BLUE),
    brighten_rgb(Color::YELLOW), brighten_rgb(Color::CYAN), brighten_rgb(Color::MAGENTA),
];



const fn brighten_rgb(color: Color) -> Color {
    Color::RGB(
        127 + color.r/255,
        127 + color.g/255,
        127 + color.b/255,
    )
}


fn draw(canvas: &mut Canvas<Window>, field: &[FieldBlock; FIELD_BLOCK_COUNT]) {
    canvas.set_draw_color((0, 0, 0));
    canvas.clear();

    canvas.set_draw_color((0xC0, 0xC0, 0xC0));
    canvas.draw_rect(Rect::new(
        OFFSET_LEFT_PX,
        OFFSET_TOP_PX,
        BLOCK_WIDTH_PX * FIELD_WIDTH_BLOCKS,
        BLOCK_HEIGHT_PX * FIELD_HEIGHT_BLOCKS,
    )).unwrap();

    let mut i = 0;
    for y in 0..FIELD_HEIGHT_BLOCKS {
        for x in 0..FIELD_WIDTH_BLOCKS {
            if let FieldBlock::Block { color_index, .. } = field[i] {
                canvas.set_draw_color(BLOCK_COLORS[usize::from(color_index)]);
                canvas.fill_rect(Rect::new(
                    OFFSET_LEFT_PX + i32::try_from(x * BLOCK_WIDTH_PX).unwrap(),
                    OFFSET_TOP_PX + i32::try_from(y * BLOCK_HEIGHT_PX).unwrap(),
                    BLOCK_WIDTH_PX,
                    BLOCK_HEIGHT_PX,
                )).unwrap();
            }
            i += 1;
        }
    }

    canvas.present();
}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Columns", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut rng_seed = [0u8; 32];
    rng_seed[0] = 23;
    rng_seed[1] = 42;
    rng_seed[2] = 69;
    let mut rng = StdRng::from_seed(rng_seed);
    let mut block_fall_counter = 0;
    let mut block_fall_limit = 64;

    let mut canvas = window.into_canvas().build().unwrap();
    let mut field = [FieldBlock::Background; FIELD_BLOCK_COUNT];
    field[12] = FieldBlock::Block { color_index: 1, falling: false };

    let mut event_pump = sdl_context.event_pump().unwrap();
    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                _ => {},
            }
        }

        draw(&mut canvas, &field);

        // handle falling blocks
        if block_fall_counter == block_fall_limit {
            block_fall_counter = 0;

            // any falling blocks?
            let mut has_falling_blocks = false;
            let falling_blocks = field
                .iter_mut()
                .rev()
                .filter(|b| b.is_falling_block());
            for falling_block in falling_blocks {
                
            }
        }
        block_fall_counter += 1;

        // more game loop

        canvas.present();
        sleep(Duration::new(0, 1_000_000_000 / 60))
    }
}
