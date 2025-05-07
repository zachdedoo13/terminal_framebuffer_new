use crossterm::terminal::size;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;
use terminal_renders::color::ColorRGB;
use terminal_renders::renderers::{RGBChar, FULL};
use terminal_renders::term_framebuffer::{
   terminal_cleanup_alternate_screen, terminal_setup_alternate_screen, TerminalFramebuffer,
   TerminalState,
};
use terminal_renders::utils::FrameRateTracker;

fn main() {
    sleep(Duration::from_millis(50));
    let mut fb = TerminalFramebuffer::<RGBChar>::new().unwrap();

    terminal_setup_alternate_screen().unwrap();

    ctrlc::set_handler(|| {
        terminal_cleanup_alternate_screen().unwrap();
        std::process::exit(1);
    })
    .unwrap();

    let mut terminal_state = TerminalState::new();
    TerminalState::enable_mouse().unwrap();

    let mut fps = FrameRateTracker::start(50);

    loop {
        terminal_state.update().unwrap();
        fps.update();

        let s = terminal_state.mouse_position.0 as f32 / size().unwrap().0 as f32;
        fb.check_size().unwrap();
        fb.iterate_uv_par(move |x, y| RGBChar {
            col: ColorRGB::from_f32(x, y, s),
            char: FULL,
        });

        let msg = format!(
            "FPS {:.2} Focused {}",
            1.0 / fps.get_frametime().as_secs_f64(),
            terminal_state.focused
        )
        .chars()
        .map(|c| RGBChar {
            col: ColorRGB::WHITE,
            char: c,
        })
        .collect::<Vec<RGBChar>>();
        overwrite_first_x(fb.raw_data(), msg);

        fb.render_wrapping().unwrap();
        stdout().flush().unwrap();
    }
}

fn overwrite_first_x<T>(dest: &mut Vec<T>, src: Vec<T>) {
    if src.len() > dest.len() {
        panic!("Source vector is larger than the destination vector.");
    }
    if src.is_empty() {
        return;
    }
    for (i, item) in src.into_iter().enumerate() {
        dest[i] = item;
    }
}
