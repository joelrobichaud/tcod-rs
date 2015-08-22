extern crate tcod;
extern crate tcod_sys as ffi;
extern crate rand;

use tcod::console::*;
use tcod::input::{Key, KeyCode, KEY_PRESS, MOUSE, check_for_event, Event};
use tcod::system;
use tcod::colors;
use tcod::colors::Color;
use tcod::chars;
use rand::Rng;
use rand::ThreadRng;
use std::char::from_u32;

const SAMPLE_SCREEN_WIDTH : i32 = 46;
const SAMPLE_SCREEN_HEIGHT : i32 = 20;

const SAMPLE_SCREEN_X : i32 = 20;
const SAMPLE_SCREEN_Y : i32 = 10;

struct ColorsState {
    cols : [Color; 4],
    dirr : [i8; 4],
    dirg : [i8; 4],
    dirb : [i8; 4],
}

impl ColorsState {
    fn new() -> ColorsState {
        ColorsState {
            cols : [Color {r:50,  g:40, b:150},
                    Color {r:240, g:85, b:5},
                    Color {r:50,  g:35, b:240},
                    Color {r:10,  g:200, b:130}],
            dirr : [1, -1, 1, 1],
            dirg : [1, -1, -1, 1],
            dirb : [1, 1, 1, -1]
        }
    }
}

fn render_colors(console: &mut Offscreen, first: bool, state : &mut ColorsState) -> () {
    enum Dir {
        TopLeft = 0,
        TopRight,
        BottomLeft,
        BottomRight,
    };

    let mut cols = state.cols;
    let mut dirr = state.dirr;
    let mut dirg = state.dirg;
    let mut dirb = state.dirb;

    let rng : &mut ThreadRng = &mut rand::thread_rng();

    if first {
        system::set_fps(0);
        console.clear()
    }

    for c in 0..4 {
        let component = rng.gen_range(0, 3);
        match component {
            0 => {
                let delta : i16 = (5 * dirr[c]) as i16;
                cols[c].r = (cols[c].r as i16 + delta) as u8;
                
                if cols[c].r == 255 {
                    dirr[c] = -1
                } else if cols[c].r == 0 {
                    dirr[c] = 1
                }
            },
            1 => {
                let delta : i16 = (5 * dirg[c]) as i16;
                cols[c].g = (cols[c].g as i16 + delta) as u8;
                
                if cols[c].g == 255 {
                    dirg[c] = -1
                } else if cols[c].g == 0 {
                    dirg[c] = 1
                }
            },
            2 => {
                let delta : i16 = (5 * dirb[c]) as i16;
                cols[c].b = (cols[c].b as i16 + delta) as u8;
                
                if cols[c].b == 255 {
                    dirb[c] = -1
                } else if cols[c].b == 0 {
                    dirb[c] = 1
                }
            },
            _ => panic!("Random number generator is broken!")
        }
    }

    // ==== scan the whole screen, interpolating corner colors ====
	for x in 0..SAMPLE_SCREEN_WIDTH {
		let xcoef = (x as f32) / ((SAMPLE_SCREEN_WIDTH-1) as f32);
        
		// get the current column top and bottom colors
		let top = colors::lerp(cols[Dir::TopLeft as usize], cols[Dir::TopRight as usize], xcoef);
		let bottom = colors::lerp(cols[Dir::BottomLeft as usize], cols[Dir::BottomRight as usize], xcoef);
		for y in 0..SAMPLE_SCREEN_HEIGHT {
			let ycoef = (y as f32) / ((SAMPLE_SCREEN_HEIGHT-1) as f32);
            
			// get the current cell color
			let cur_color = colors::lerp(top, bottom, ycoef);
			console.set_char_background(x, y, cur_color, BackgroundFlag::Set);
		}
	}

    // ==== print the text with a random color ====
	// get the background color at the text position
	let mut text_color = console.get_char_background(SAMPLE_SCREEN_WIDTH/2, 5);
	// and invert it
	text_color.r = 255 - text_color.r;
	text_color.g = 255 - text_color.g;
	text_color.b = 255 - text_color.b;
	// put random text (for performance tests) 
	for x in 0..SAMPLE_SCREEN_WIDTH {
        for y in 0..SAMPLE_SCREEN_HEIGHT {
			let mut c;
			let mut col = console.get_char_background(x, y);
			col = colors::lerp(col, colors::BLACK, 0.5);
			// use colored character 255 on first and last lines
			if y == 0 || y == SAMPLE_SCREEN_HEIGHT-1 {
				c = std::char::from_u32(0x00ff).unwrap();
			} else {
                let r = rng.gen_range('a' as u32, 'z' as u32);
				c = from_u32(r).unwrap();
			}
			
			console.set_default_foreground(col);
			console.put_char(x, y, c, BackgroundFlag::None);
		}
	}

    console.set_default_foreground(text_color);
	// the background behind the text is slightly darkened using the Multiply flag
	console.set_default_background(colors::GREY);
	console.print_rect_ex(SAMPLE_SCREEN_WIDTH/2, 5, SAMPLE_SCREEN_WIDTH-2, SAMPLE_SCREEN_HEIGHT-1,
		                  BackgroundFlag::Multiply, TextAlignment::Center,
		                  "The Doryen library uses 24 bits colors, for both background and foreground.");
}

struct OffscreenState {
    secondary : Offscreen,
    screenshot : Offscreen,
    init : bool,
    counter : i32,
    x : i32,
    y : i32,
    xdir : i32,
    ydir : i32
}

impl OffscreenState {
    fn new() -> OffscreenState {
        OffscreenState {
            secondary : Offscreen::new(SAMPLE_SCREEN_WIDTH/2, SAMPLE_SCREEN_HEIGHT/2),
            screenshot : Offscreen::new(SAMPLE_SCREEN_WIDTH, SAMPLE_SCREEN_HEIGHT),
            init : false,
            counter : 0,
            x : 0,
            y : 0,
            xdir : 1,
            ydir : 1
        }
    }
}

fn render_offscreen(console: &mut Offscreen,
                    first: bool,
                    state: &mut OffscreenState) -> () {
    if !state.init {
        state.init = true;
        state.secondary.print_frame(0, 0, SAMPLE_SCREEN_WIDTH/2, SAMPLE_SCREEN_HEIGHT/2,
                                    false, BackgroundFlag::Set, Some("Offscreen console"));
        state.secondary.print_rect_ex(SAMPLE_SCREEN_WIDTH/4, 2, SAMPLE_SCREEN_WIDTH/2-2,
                                      SAMPLE_SCREEN_HEIGHT/2, BackgroundFlag::None, TextAlignment::Center,
                                      "You can render to an offscreen console and blit in on another one, simulating alpha transparency.");
    }

    if first {
        system::set_fps(30);
        blit(console, (0, 0), (SAMPLE_SCREEN_WIDTH, SAMPLE_SCREEN_HEIGHT),
             &mut state.screenshot, (0, 0), 1.0, 1.0);
    }

    state.counter += 1;
    if state.counter % 20 == 0 {
        state.x += state.xdir;
        state.y += state.ydir;
        if state.x == (SAMPLE_SCREEN_WIDTH/2 + 5) { state.xdir = -1 }
        else if state.x == -5 { state.xdir = 1 }
        if state.y == (SAMPLE_SCREEN_HEIGHT/2 + 5) { state.ydir = -1 }
        else if state.y == -5 { state.ydir = 1 }
    }

    blit(&state.screenshot, (0, 0), (SAMPLE_SCREEN_WIDTH, SAMPLE_SCREEN_HEIGHT),
         console, (0, 0), 1.0, 1.0);
    blit(&state.secondary, (0, 0), (SAMPLE_SCREEN_WIDTH/2, SAMPLE_SCREEN_HEIGHT/2),
         console, (state.x, state.y), 1.0, 0.75);
}

/*
fn render_lines(_console: &mut Offscreen, _first: bool) -> () {}
fn render_noise(_console: &mut Offscreen, _first: bool) -> () {}
fn render_fov(_console: &mut Offscreen, _first: bool) -> () {}
fn render_path(_console: &mut Offscreen, _first: bool) -> () {}
fn render_bsp(_console: &mut Offscreen, _first: bool) -> () {}
fn render_image(_console: &mut Offscreen, _first: bool) -> () {}
fn render_mouse(_console: &mut Offscreen, _first: bool) -> () {}
fn render_name(_console: &mut Offscreen, _first: bool) -> () {}
fn render_sdl(_console: &mut Offscreen, _first: bool) -> () {}
*/

static RENDERER_NAME : [&'static str; 3] = ["F1 GLSL   ","F2 OPENGL ","F3 SDL    "];

fn main() {
    let mut colors_state = ColorsState::new();
    let mut offscreen_state = OffscreenState::new();
    let samples = vec!["  True colors      ".to_string(),
        "  Offscreen console".to_string(),
        "  Line drawing     ".to_string(),
        "  Noise            ".to_string(),
        "  Field of view    ".to_string(),
        "  Path finding     ".to_string(),
        "  Bsp toolkit      ".to_string(),
        "  Image toolkit    ".to_string(),
        "  Mouse support    ".to_string(),
        "  Name generator   ".to_string(),
        "  SDL callback     ".to_string()];
    let mut cur_sample = 0;
    let mut first = true;
    let (mut fullscreen_width, mut fullscreen_height) = (0, 0);
    let mut font = "consolas10x10_gs_tc.png".to_string();
    let mut font_type = FontType::Greyscale;
    let mut font_layout = FontLayout::Tcod;
    let (mut nb_char_horiz, mut nb_char_vertic) = (0, 0);
    let mut fullscreen = false;
    let mut console = Offscreen::new(SAMPLE_SCREEN_WIDTH, SAMPLE_SCREEN_HEIGHT);
    
    let renderer = Renderer::SDL;

    let mut args = std::env::args();

    loop {
        use std::i32;
        use std::str::FromStr;

        match args.next() {
            None => break,
            Some(opt) => match opt.as_ref() {
                "-font" => {
                    let n = args.next();
                    if n.is_some() {
                        font = n.unwrap()
                    }
                },
                "-font-nb-char" => {
                    let horiz = args.next();
                    let vertic = args.next();
                    if horiz.is_some() {
                        nb_char_horiz = i32::from_str(horiz.unwrap().as_ref()).unwrap()
                    }
                    if vertic.is_some() {
                        nb_char_vertic = i32::from_str(vertic.unwrap().as_ref()).unwrap()
                    }
                }
                "-fullscreen-resolution" => {
                    let width  = args.next();
                    let height = args.next();
                    if width.is_some() {
                        fullscreen_width = i32::from_str(width.unwrap().as_ref()).unwrap()
                    }
                    if height.is_some() {
                        fullscreen_height = i32::from_str(height.unwrap().as_ref()).unwrap()
                    }
                }
                "-fullscreen" => fullscreen = true,
                "-font-in-row" => font_layout = FontLayout::AsciiInRow,
                "-font-greyscale" => font_type = FontType::Greyscale,
                "-font-tcod" => font_layout = FontLayout::Tcod,
                "-help" | "-?" => {
                    println!("options :");
			        println!("-font <filename> : use a custom font");
			        println!("-font-nb-char <nb_char_horiz> <nb_char_vertic> : number of characters in the font");
			        println!("-font-in-row : the font layout is in row instead of columns");
			        println!("-font-tcod : the font uses TCOD layout instead of ASCII");
			        println!("-font-greyscale : antialiased font using greyscale bitmap");
			        println!("-fullscreen : start in fullscreen");
			        println!("-fullscreen-resolution <screen_width> <screen_height> : force fullscreen resolution");
			        println!("-renderer <num> : set renderer. 0 : GLSL 1 : OPENGL 2 : SDL");
                    std::process::exit(0)
                }
                _ => continue
            }
        }
    }

    if fullscreen_width > 0 {
		system::force_fullscreen_resolution(fullscreen_width, fullscreen_height);
    }
    let mut root = Root::initializer()
        .size(80, 50)
        .title("libtcod Rust sample")
        .fullscreen(fullscreen)
        .renderer(renderer)
        .font(font, font_layout)
        .font_type(font_type)
        .font_dimensions(nb_char_horiz, nb_char_vertic)
        .init();
    let mut credits_end = false;

    while !root.window_closed() {
        if !credits_end {
            credits_end = root.render_credits(60, 43, false);
        }
        // print the list of samples
        for i in 0..samples.len() {
            if i == cur_sample {
                root.set_default_foreground(colors::WHITE);
                root.set_default_background(colors::LIGHT_BLUE);
            } else {
                root.set_default_foreground(colors::GREY);
                root.set_default_background(colors::BLACK);
            }
            let y : i32 = 46 - (samples.len() as i32 - i as i32);
            let fun = &samples[i]; //.name;
            root.print_ex(2, y, BackgroundFlag::Set, TextAlignment::Left, fun);
        }

        // print the help message
        root.set_default_foreground(colors::GREY);
        root.print_ex(79, 46, BackgroundFlag::None, TextAlignment::Right,
                      format!("last frame : {:3.0} ms ({:3} fps)",
                      system::get_last_frame_length() * 1000.0,
                              system::get_fps()));
        let time = system::get_elapsed_time();
        root.print_ex(79, 47, BackgroundFlag::None, TextAlignment::Right,
                      format!("elapsed {:8}ms {:4.2}s",
                              time.num_milliseconds(),
                              time.num_milliseconds() as f32/ 1000.0));
        root.print(2, 47, format!("{}{} : select a sample",
                                  chars::ARROW_N, chars::ARROW_S));
        let fullscreen_text = if root.is_fullscreen() {"windowed mode"}
                              else {"fullscren_mode"};
        root.print(2, 48, format!("ALT-ENTER : switch to {}", fullscreen_text));

        match cur_sample {
            0 => render_colors(&mut console, first, &mut colors_state),
            1 => render_offscreen(&mut console, first, &mut offscreen_state),
            2...10 => {}
            _ => panic!("Wrong menu item")
        }
        first = false;
        blit(&console, (0, 0), (SAMPLE_SCREEN_WIDTH, SAMPLE_SCREEN_HEIGHT),
             &mut root, (SAMPLE_SCREEN_X, SAMPLE_SCREEN_Y), 1.0, 1.0);

        // erase the renderer in debug mode (needed because the root
        // console is not cleared each frame)
		root.print(1, 1, "        ");

        root.set_default_foreground(colors::GREY);
        root.set_default_background(colors::BLACK);
        root.print_ex(42, 46-(ffi::TCOD_NB_RENDERERS as i32 + 1),
                      BackgroundFlag::Set, TextAlignment::Left,
                      "Renderer :");
        for i in 0..(ffi::TCOD_NB_RENDERERS as i32) {
            if i == system::get_renderer() as i32{
                root.set_default_foreground(colors::WHITE);
                root.set_default_background(colors::LIGHT_BLUE);
            } else {
                root.set_default_foreground(colors::GREY);
                root.set_default_background(colors::BLACK);
            }
            root.print_ex(42, 46 - (ffi::TCOD_NB_RENDERERS as i32 - i),
                          BackgroundFlag::Set, TextAlignment::Left,
                          RENDERER_NAME[i as usize]);
        }
        
        root.flush();
        let event = check_for_event(KEY_PRESS | MOUSE);
        match event {
            None => {continue;}
            Some((_flag, Event::Key(state))) => {
                match state.key {
                    Key::Special(KeyCode::Down) => {
                        cur_sample = (cur_sample + 1) % samples.len();
                        first = true
                    }
                    Key::Special(KeyCode::Up) => {
                        if cur_sample == 0 { cur_sample = samples.len()-1; }
                        else { cur_sample -= 1; }
                        first = true
                    }
                    Key::Special(KeyCode::Enter) if state.left_alt => {
                        let fullscreen = root.is_fullscreen();
                        root.set_fullscreen(!fullscreen)
                    }
                    Key::Special(KeyCode::PrintScreen) => {
                        // TODO
                    }
                    Key::Special(KeyCode::Escape) => { break }
                    Key::Special(KeyCode::F1) => {
                        system::set_renderer(Renderer::GLSL)
                    }
                    Key::Special(KeyCode::F2) => {
                        system::set_renderer(Renderer::OpenGL)
                    }
                    Key::Special(KeyCode::F3) => {
                        system::set_renderer(Renderer::SDL)
                    }
                    _ => {continue;}
                }
            }
            _ => {continue;}
        }
    }
}
