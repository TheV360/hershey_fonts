use std::fs::{read_dir, read_to_string};
use std::ffi::OsStr;

use minifb::{Scale, Window, WindowOptions, Key};
// use nanorand::WyRand;

use hershey_reader::*;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

mod bresenham;
use bresenham::*;

const CORNER: Vec2 = (WIDTH as Coord, HEIGHT as Coord);
const CENTER: Vec2 = (CORNER.0 / 2, CORNER.1 / 2);
			
// const COOL_COLORS: [u32; 8] = [0xFFFFFF, 0xFB4934, 0xFE8019, 0xFABD2F, 0xB8BB26, 0x8EC07C, 0x83A598, 0xD3869B];
const COOL_COLORS: [u32; 9] = [0xFFFFFF, 0xA89984, 0xCC241D, 0xD65D0E, 0xD79921, 0x98971A, 0x689D6A, 0x458588, 0xB16286];

fn main() -> std::io::Result<()> {
	let look_in = &std::env::args_os().nth(1)
		.or_else(|| std::env::var_os("HERSHEY_FONTS_DIR"))
		.unwrap_or_else(|| {
			println!("Falling back to default font directory (the one inside this repository)");
			"fonts/".into()
		});
	let mut fonts: Vec<(String, Vec<HersheyChar>)> = Vec::new();
	
	let dir = read_dir(look_in)
		.map_err(|dir| {
			println!(r"i need some fonts to view.
			supply a directory of .jhf fonts as my first arg?
			
			anyway here's the ugly error:");
			dir
		})?;
	
	for entry in dir {
		let entry = entry?;
		let path = entry.path();
		let mut success = false;
		
		if path.is_file() {
			if let Some(ext) = path.extension().and_then(OsStr::to_str) {
				if ext == "jhf" {
					let jhf = read_to_string(&path)?;
					let jhf = jhf.trim_end();
					
					let mut font = Vec::new();
					for (ln, line) in jhf.lines().enumerate() {
						match HersheyChar::new_from_str(line) {
							Ok(chr) => font.push(chr),
							Err(x) => panic!("`{}`\nIn file {}:{}, couldn't parse, {:?}", line, path.to_string_lossy(), ln, x),
						}
					}
					
					let filename = path.file_name()
						.map(|x| x.to_string_lossy().into_owned())
						.unwrap_or_else(||"oops".to_string());
					fonts.push((filename, font));
					success = true;
				}
			}
			
			let success = if success { "Loaded" } else { "Skipped" };
			println!("{} `{}`.", success, path.to_string_lossy());
		}
	}
	
	let mut buffer: Box<[u32]> = vec![0x201d1a; WIDTH * HEIGHT]
		.into_boxed_slice();
	
	let mut win = Window::new(
		"Hershey Font Viewer by V 360", WIDTH, HEIGHT,
		WindowOptions {
			scale: Scale::X1,
			..Default::default()
		}
	).expect("Failed to open window!");
	
	let mut specimen = r"
the quick brown fox jumped
over the lazy dog
THE QUICK BROWN FOX JUMPED
OVER THE LAZY DOG
(0123456789)
<HTML> ? @
	".trim().to_string();
	
	// hardcoded US A layout. sorry.
	fn key_to_char(k: Key, sh: bool) -> Option<char> {
		const NM_UP: &[u8] = b")!@#$%^&*(";
		let kc = k as u8;
		match (sh, k, kc) {
			(false, _,  0..= 9) => Some((b'0' +  kc      ) as char),
			(false, _, 10..=35) => Some((b'a' + (kc - 10)) as char),
			( true, _,  0..= 9) => Some(NM_UP[kc as usize] as char),
			( true, _, 10..=35) => Some((b'A' + (kc - 10)) as char),
			
			(_, Key::Space, _) => Some(' '),
			
			(false, Key::Apostrophe, _) => Some('\''),
			( true, Key::Apostrophe, _) => Some('"'),
			(false, Key::Backquote, _) => Some('`'),
			( true, Key::Backquote, _) => Some('~'),
			(false, Key::Backslash, _) => Some('\\'),
			( true, Key::Backslash, _) => Some('|'),
			(false, Key::Comma, _) => Some(','),
			( true, Key::Comma, _) => Some('<'),
			(false, Key::Equal, _) => Some('='),
			( true, Key::Equal, _) => Some('+'),
			(false, Key::LeftBracket, _) => Some('['),
			( true, Key::LeftBracket, _) => Some('{'),
			(false, Key::Minus, _) => Some('-'),
			( true, Key::Minus, _) => Some('_'),
			(false, Key::Period, _) => Some('.'),
			( true, Key::Period, _) => Some('>'),
			(false, Key::RightBracket, _) => Some(']'),
			( true, Key::RightBracket, _) => Some('}'),
			(false, Key::Semicolon, _) => Some(';'),
			( true, Key::Semicolon, _) => Some(':'),
			(false, Key::Slash, _) => Some('/'),
			( true, Key::Slash, _) => Some('?'),
			
			(_, Key::Backspace, _) => Some('\x08'),
			(_, Key::Delete, _) => Some('\x7F'),
			(_, Key::Enter, _) => Some('\n'),
			
			_ => None
		}
	}
	
	enum Page {
		Help,
		Specimen,
		Map
	}
	impl Page {
		fn back(self) -> Self {
			use Page::*;
			match self {
				Help => Map,
				Specimen => Help,
				Map => Specimen,
			}
		}
		fn next(self) -> Self {
			use Page::*;
			match self {
				Help => Specimen,
				Specimen => Map,
				Map => Help,
			}
		}
		fn get_name(&self) -> &'static str {
			use Page::*;
			match self {
				Help => "Help",
				Specimen => "Specimen",
				Map => "Character Map",
			}
		}
	}
	
	let ui_font = &fonts.iter()
		.find(|(name, _)| name == "futural.jhf")
		.unwrap_or_else(|| fonts.first().unwrap())
		.1;
	
	let mut specimen = r"
the quick brown fox jumped
over the lazy dog
THE QUICK BROWN FOX JUMPED
OVER THE LAZY DOG
(0123456789)
<HTML> ? @
	".trim().to_string();
	
	let mut cur_char = 0;
	let mut chrmap_view = 0; // put this into a Page-by-Page state later
	let mut cur_font = 0;
	let mut cur_page = Page::Help;
	let mut font_size = 1.0;
	let mut redraw = true; // hastily added
	
	// gonna use this for gfx buttons
	let mut mouse = (0, 0);
	let (mut mouse_click, mut mouse_down) = (false, false);
	let mut mouse_scroll;
	
	// I did not go into this intending to make this code good. Sorry
	
	while win.is_open() {
		if let Some((x, y)) = win.get_mouse_pos(minifb::MouseMode::Discard) {
			mouse = (x as Coord, y as Coord);
		}
		if let Some((x, y)) = win.get_scroll_wheel() {
			mouse_scroll = (x as Coord, y as Coord);
		} else {
			mouse_scroll = (0, 0);
		}
		
		if win.get_mouse_down(minifb::MouseButton::Left) {
			if !mouse_down && !mouse_click {
				mouse_click = true;
				mouse_down = false;
			} else {
				mouse_click = false;
				mouse_down = true;
			}
		} else {
			mouse_click = false;
			mouse_down = false;
		}
		
		let shift = win.is_key_down(Key::LeftShift) || win.is_key_down(Key::RightShift);
		if let Some(keys) = win.get_keys_pressed(minifb::KeyRepeat::Yes) {
			for k in keys {
				if let Some(ch) = key_to_char(k, shift) {
					if ch == '\x08' {
						specimen.pop();
					} else {
						specimen.push(ch);
					}
					redraw = true;
				}
			}
		}
		
		// Switch fonts
		if cur_font > 0 && win.is_key_pressed(Key::Up, minifb::KeyRepeat::Yes) {
			cur_font -= 1; redraw = true;
		}
		if cur_font < fonts.len() - 1 && win.is_key_pressed(Key::Down, minifb::KeyRepeat::Yes) {
			cur_font += 1; redraw = true;
		}
		let font = &fonts[cur_font];
		
		// Switch pages
		if win.is_key_pressed(Key::PageUp, minifb::KeyRepeat::Yes) {
			cur_page = cur_page.back(); redraw = true;
		}
		if win.is_key_pressed(Key::PageDown, minifb::KeyRepeat::Yes) {
			cur_page = cur_page.next(); redraw = true;
		}
		
		const GRID_CELLS: Vec2 = (8, 7);
		
		const CHRMAP_TL: Vec2 = (CENTER.0, 32);
		const CHRMAP_BR: Vec2 = (CORNER.0 - 1, CORNER.1 - 1);
		
		const GRID_CELLS_F: (f64, f64) = (GRID_CELLS.0 as f64, GRID_CELLS.1 as f64);
		const CHRMAP_TL_F: (f64, f64) = (CHRMAP_TL.0 as f64, CHRMAP_TL.1 as f64);
		const CHRMAP_BR_F: (f64, f64) = (CHRMAP_BR.0 as f64, CHRMAP_BR.1 as f64);
		
		// Update loop
		match cur_page {
			Page::Help => {},
			Page::Specimen => {
				if font_size > 0.51 && win.is_key_pressed(Key::NumPadMinus, minifb::KeyRepeat::Yes) {
					font_size -= 0.25; redraw = true;
				}
				if font_size < 6.99 && win.is_key_pressed(Key::NumPadPlus, minifb::KeyRepeat::Yes) {
					font_size += 0.25; redraw = true;
				}
			},
			Page::Map => {
				let b4_char = cur_char;
				
				if cur_char > 0 && win.is_key_pressed(Key::Left, minifb::KeyRepeat::Yes) {
					cur_char -= 1;
				}
				if win.is_key_pressed(Key::Right, minifb::KeyRepeat::Yes) {
					cur_char += 1;
				}
				
				let chrmap_camera_max = 0.max((font.1.len() as Coord - 1) / GRID_CELLS.0 + 1 - GRID_CELLS.1);
				
				if vec2_within_bounds(mouse, CHRMAP_TL, CHRMAP_BR) {
					if mouse_click {
						let mouse = invlerp_vec(CHRMAP_TL_F, CHRMAP_BR_F, (mouse.0 as f64, mouse.1 as f64));
						let mouse = ((mouse.0 * GRID_CELLS_F.0).floor() as Coord, (mouse.1 * GRID_CELLS_F.1).floor() as Coord);
						cur_char = (mouse.0 + (mouse.1 + chrmap_view) * GRID_CELLS.0) as usize;
						cur_char = 0.max(cur_char.min(font.1.len() - 1));
					}
					if mouse_scroll != (0, 0) {
						let dir = mouse_scroll.1.signum();
						chrmap_view -= dir;
						redraw = true;
					}
				}
				
				cur_char = 0.max(cur_char.min(font.1.len() - 1));
				
				if b4_char != cur_char {
					let cur_row = cur_char as Coord / GRID_CELLS.0;
					if chrmap_view >= cur_row {
						chrmap_view -= 1;
					}
					if chrmap_view + GRID_CELLS.1 - 1 <= cur_row {
						chrmap_view += 1;
					}
					
					redraw = true;
				}
				
				chrmap_view = 0.max(chrmap_view.min(chrmap_camera_max));
			},
		}
		
		if redraw {
			redraw = false;
			
			// Clear screen.
			buffer.fill(0x201d1a);
			
			let buf = &mut buffer;
			match cur_page {
				Page::Help => {
					const HELP_TEXT: &str = r"
Help

Hello! Welcome to my Font Viewer.
Use PgUp/PgDown to switch tabs.
Use ^/v to switch fonts.

Specimen:
Use keyboard to write example.
Use Numpad +/- to control size.

Map:
Use </>/Click to select.
Use scroll wheel to scroll.
					";
					draw_hershey_str(buf, ui_font, HELP_TEXT, (64, 24), 1.5, COOL_COLORS[0]);
				},
				Page::Specimen => {
					let tooltip = format!("{}\n#{}: {} (x{:.2})", cur_page.get_name(), cur_font, font.0, font_size);
					draw_hershey_str(buf, ui_font, &tooltip, (32, HEIGHT as Coord - 64), 1.0, COOL_COLORS[0]);
					
					let specimen = if specimen.is_empty() { "Type some text..." } else { &specimen };
					draw_hershey_str(buf, &font.1, specimen, (64, 96), font_size, COOL_COLORS[0]);
				},
				Page::Map => {
					let tooltip = format!("{}\n#{}: {}; {}", cur_page.get_name(), cur_font, font.0, cur_char);
					draw_hershey_str(buf, ui_font, &tooltip, (32, 40), 0.75, COOL_COLORS[0]);
					
					const CHR_SIZE: f64 = 8.0;
					
					let chr = &font.1[cur_char];
					let middle = (CENTER.0 / 2, CENTER.1);
					draw_hershey_char(buf, chr, middle, CHR_SIZE, COOL_COLORS[0]);
					
					for j in 0..GRID_CELLS.1 {
						for i in 0..GRID_CELLS.0 {
							let ci = (i + (j + chrmap_view) * GRID_CELLS.0) as usize;
							
							// hell
							let topleft = (i as f64 / GRID_CELLS_F.0, j as f64 / GRID_CELLS_F.1);
							let center = ((i as f64 + 0.5) / GRID_CELLS_F.0, (j as f64 + 0.5) / GRID_CELLS_F.1);
							let bottomright = ((i as f64 + 1.0) / GRID_CELLS_F.0, (j as f64 + 1.0) / GRID_CELLS_F.1);
							
							let tl = lerp_vec(CHRMAP_TL_F, CHRMAP_BR_F, topleft);
							let ce = lerp_vec(CHRMAP_TL_F, CHRMAP_BR_F, center);
							let br = lerp_vec(CHRMAP_TL_F, CHRMAP_BR_F, bottomright);
							
							let tl = (tl.0.floor() as Coord, tl.1.floor() as Coord);
							let br = (br.0.ceil() as Coord - 1, br.1.ceil() as Coord - 1);
							
							let cpos = (ce.0.round() as Coord, ce.1.round() as Coord);
							
							if let Some(chr) = font.1.get(ci) {
								let is_current = ci == cur_char;
								
								draw_rect(buf, tl, br, COOL_COLORS[1]);
								draw_hershey_char(buf, chr, cpos, if is_current { 1.25 } else { 1.0 }, COOL_COLORS[0]);
								
								if is_current {
									const INSET_AMT: Coord = 3;
									draw_rect(buf, (tl.0 + INSET_AMT, tl.1 + INSET_AMT), (br.0 - INSET_AMT, br.1 - INSET_AMT), COOL_COLORS[1]);
								}
							}
						}
					}
					
					let bottom = (middle.0, CORNER.1 - 1);
					let lh = ((chr.left_hand as f64 * CHR_SIZE) as Coord + bottom.0, bottom.1);
					let rh = ((chr.right_hand as f64 * CHR_SIZE) as Coord + bottom.0, bottom.1);
					draw_line(buf, lh, rh, COOL_COLORS[1]);
					draw_line(buf, (lh.0, lh.1 - (HEIGHT as Coord / 16)), lh, COOL_COLORS[1]);
					draw_line(buf, (rh.0, rh.1 - (HEIGHT as Coord / 16)), rh, COOL_COLORS[1]);
				},
			}
		}
		
		win.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
	}
	
	Ok(())
}

#[inline]
fn lerp(a: f64, b: f64, p: f64) -> f64 {
	((1.0 - p) * a) + (p * b)
}
#[inline]
fn invlerp(a: f64, b: f64, v: f64) -> f64 {
	(v - a) / (b - a)
}

fn lerp_vec(a: (f64, f64), b: (f64, f64), p: (f64, f64)) -> (f64, f64) {
	(lerp(a.0, b.0, p.0), lerp(a.1, b.1, p.1))
}
fn invlerp_vec(a: (f64, f64), b: (f64, f64), v: (f64, f64)) -> (f64, f64) {
	(invlerp(a.0, b.0, v.0), invlerp(a.1, b.1, v.1))
}

fn vec2_within(a: Vec2) -> bool {
	a.0 >= 0 && a.0 < CORNER.0 &&
	a.1 >= 0 && a.1 < CORNER.1
}
fn vec2_within_bounds(a: Vec2, tl: Vec2, br: Vec2) -> bool {
	a.0 >= tl.0 && a.0 < br.0 &&
	a.1 >= tl.1 && a.1 < br.1
}

#[inline]
fn vec2_to_index(a: Vec2) -> usize {
	(a.0 + a.1 * CORNER.0) as usize
}

#[inline]
fn v2i8_to_vec2(v: (i8, i8)) -> Vec2 {
	(v.0 as Coord, v.1 as Coord)
}

/// simply don't draw br > tl
fn draw_rect(buf: &mut Box<[u32]>, tl: Vec2, br: Vec2, c: u32) {
	if !vec2_within(tl) || !vec2_within(br) { return; }
	
	unsafe {
		for x in tl.0..=br.0 {
			*buf.get_unchecked_mut(vec2_to_index((x, tl.1))) = c;
			*buf.get_unchecked_mut(vec2_to_index((x, br.1))) = c;
		}
		for y in (tl.1..br.1).skip(1) {
			*buf.get_unchecked_mut(vec2_to_index((tl.0, y))) = c;
			*buf.get_unchecked_mut(vec2_to_index((br.0, y))) = c;
		}
	}
}

fn draw_line(buf: &mut Box<[u32]>, a: Vec2, b: Vec2, c: u32) {
	if !vec2_within(a) || !vec2_within(b) { return; }
	
	unsafe {
		for p in Line::new(a, b) {
			*buf.get_unchecked_mut(vec2_to_index(p)) = c;
		}
	}
}

fn draw_hershey_char(buf: &mut Box<[u32]>, chr: &HersheyChar, p: Vec2, s: f64, c: u32) {
	let mut pen_prev: Option<Vec2> = None;
	
	for v in chr.vertex_data.iter() {
		let mut v = v.map(v2i8_to_vec2);
		if let Some(v1) = v {
			let v1 = (
				p.0 + (v1.0 as f64 * s).round() as Coord,
				p.1 + (v1.1 as f64 * s).round() as Coord
			);
			v = Some(v1);
			if let Some(v2) = pen_prev {
				draw_line(buf, v1, v2, c);
			}
		}
		pen_prev = v;
	}
}

// does it show that this was hacked together?
// TODO: fix kerning
fn draw_hershey_str(buf: &mut Box<[u32]>, font: &[HersheyChar], st: &str, p: Vec2, s: f64, c: u32) {
	let mut ofs = (0, 0);
	
	for ch in st.bytes() {
		if ch == b'\n' {
			ofs = (0, ofs.1 + 32);
			continue;
		}
		
		let ch = ch.saturating_sub(b' ') as usize;
		let ch = font.get(ch);
		if let Some(ch) = ch {
			let w = (ch.right_hand - ch.left_hand) as Coord;
			let p = (
				p.0 + ((ofs.0 - ch.left_hand as Coord) as f64 * s) as Coord,
				p.1 + ((ofs.1 as f64) * s) as Coord
			);
			draw_hershey_char(buf, ch, p, s, c);
			ofs.0 += w;
		}
	}
}
