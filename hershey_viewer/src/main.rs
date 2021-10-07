use std::fs::{File, read_dir, read_to_string};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use minifb::{Scale, Window, WindowOptions, Key};

use hershey_reader::*;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

mod bresenham;
use bresenham::*;

fn main() -> std::io::Result<()> {
	let mut fonts: Vec<(String, Vec<HersheyChar>)> = Vec::new();
	for entry in read_dir("hershey_viewer/fonts/")? {
		let entry = entry?;
		let path = entry.path();
		let mut success = false;
		
		if path.is_file() {
			if let Some(ext) = path.extension().and_then(OsStr::to_str) {
				if ext == "jhf" {
					let jhf = read_to_string(&path)?;
					let jhf = jhf.trim();
					
					let mut font = Vec::new();
					for line in jhf.lines() {
						match HersheyChar::new_from_str(line) {
							Ok(chr) => font.push(chr),
							Err(x) => println!("`{}`\nCouldn't parse, {:?}", line, x),
						}
					}
					
					let filename = path.file_name().unwrap().to_string_lossy().into_owned();
					fonts.push((filename, font));
					success = true;
				}
			}
			
			println!(
				"{} `{}`.",
				if success { "Loaded" } else { "Skipped" },
				path.to_string_lossy()
			);
		}
	}
	
	let mut buffer: Box<[u32]> = vec![0x201d1a; WIDTH * HEIGHT]
		.into_boxed_slice();
	
	let mut win = Window::new(
		"Hello", WIDTH, HEIGHT,
		WindowOptions {
			scale: Scale::X1,
			..Default::default()
		}
	).unwrap();
	win.limit_update_rate(Some(core::time::Duration::from_secs_f64(1.0/72.0)));
	
	let mut cur_font = 0;
	let mut cur_char = 0;
	let mut mouse = (0, 0);
	
	let mut specimen = "the quick brown fox jumped\nover the lazy dog\nTHE QUICK BROWN FOX JUMPED\nOVER THE LAZY DOG\n(0123456789)\n<HTML> ? @".to_string();
	
	fn key_to_char(k: Key) -> Option<char> {
		let kc = k as u8;
		match (k, kc) {
			(_,  0..= 9) => Some((b'0' +  kc      ) as char),
			(_, 10..=35) => Some((b'a' + (kc - 10)) as char),
			
			(Key::Space, _) => Some(' '),
			
			(Key::Backslash, _) => Some('\\'),
			(Key::Comma, _) => Some(','),
			(Key::Equal, _) => Some('='),
			(Key::LeftBracket, _) => Some('['),
			(Key::Minus, _) => Some('-'),
			(Key::Period, _) => Some('.'),
			(Key::RightBracket, _) => Some(']'),
			(Key::Semicolon, _) => Some(';'),
			(Key::Slash, _) => Some('/'),
			
			(Key::Backspace, _) => Some('\x08'),
			(Key::Delete, _) => Some('\x7F'),
			(Key::Enter, _) => Some('\n'),
			
			_ => None
		}
	}
	
	// I did not go into this intending to make this code good. Sorry
	
	while win.is_open() {
		if let Some((x, y)) = win.get_mouse_pos(minifb::MouseMode::Pass) {
			mouse = (x as isize, y as isize);
		}
		let mouse = (mouse.0 + 48, mouse.1);
		
		// Clear screen.
		buffer.fill(0x201d1a);
		
		let (a, b) =
		if win.get_mouse_down(minifb::MouseButton::Left) {
			((64, 96), mouse)
		} else {
			(mouse, (64, 96))
		};
		
		let shift = win.is_key_down(Key::LeftShift) || win.is_key_down(Key::RightShift);
		if let Some(keys) = win.get_keys_pressed(minifb::KeyRepeat::Yes) {
			for k in keys {
				if let Some(ch) = key_to_char(k) {
					if ch == '\x08' {
						specimen.pop();
					} else {
						specimen.push(if shift { ch.to_ascii_uppercase() } else { ch });
					}
				}
			}
		}
		
		// const COOL_COLORS: [u32; 8] = [0xFFFFFF, 0xFB4934, 0xFE8019, 0xFABD2F, 0xB8BB26, 0x8EC07C, 0x83A598, 0xD3869B];
		const COOL_COLORS: [u32; 8] = [0xFFFFFF, 0xCC241D, 0xD65D0E, 0xD79921, 0x98971A, 0x689D6A, 0x458588, 0xB16286];
		
		draw_hershey_char(&mut buffer, &fonts[cur_font].1[cur_char], a, 3, 0xFFEFEA);
		draw_hershey_str(&mut buffer, &fonts[5].1, &format!("{} (#{}) ch{}", fonts[cur_font].0, cur_font, cur_char), (a.0 + 128, a.1), 1, 0xFFEFEA);
		
		/*for (col_index, &color) in COOL_COLORS.iter().enumerate().rev() {
			let col_distance = col_index as isize;
			let b = (b.0 + col_distance, b.1 + col_distance);
			draw_hershey_str(
				&mut buffer, &fonts[cur_font].1,
				&specimen,
				b, 2,
				color
			);
		}
		draw_hershey_str(&mut buffer, &fonts[cur_font].1, &specimen, (b.0 + 1, b.1), 2, COOL_COLORS[0]);
		draw_hershey_str(&mut buffer, &fonts[cur_font].1, &specimen, (b.0, b.1 + 1), 2, COOL_COLORS[0]);*/
		draw_hershey_str(&mut buffer, &fonts[cur_font].1, &specimen, b, 2, COOL_COLORS[0]);
		
		if cur_font > 0 && win.is_key_pressed(Key::Up, minifb::KeyRepeat::Yes) { cur_font -= 1; }
		if cur_font < fonts.len() - 1 && win.is_key_pressed(Key::Down, minifb::KeyRepeat::Yes) {
			cur_font += 1;
		}
		
		if cur_char > 0 && win.is_key_pressed(Key::Left, minifb::KeyRepeat::Yes) { cur_char -= 1; }
		if win.is_key_pressed(Key::Right, minifb::KeyRepeat::Yes) {
			cur_char += 1;
		}
		cur_char = 0.max(cur_char.min(fonts[cur_font].1.len() - 1));
		
		win.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
	}
	
	Ok(())
}

fn vec2_within(a: Vec2) -> bool {
	a.0 >= 0 && a.0 < WIDTH as isize &&
	a.1 >= 0 && a.1 < HEIGHT as isize
}

#[inline]
fn vec2_to_index(a: Vec2) -> usize {
	(a.0 + a.1 * WIDTH as isize) as usize
}

#[inline]
fn v2i8_to_vec2(v: (i8, i8)) -> Vec2 {
	(v.0 as isize, v.1 as isize)
}

fn draw_line(buf: &mut Box<[u32]>, a: Vec2, b: Vec2, c: u32) {
	if !(vec2_within(a) && vec2_within(b)) { return; }
	
	for p in Line::new(a, b) { unsafe {
		*buf.get_unchecked_mut(vec2_to_index(p)) = c;
	} }
}

fn draw_hershey_char(buf: &mut Box<[u32]>, chr: &HersheyChar, p: Vec2, s: isize, c: u32) {
	let mut pen_prev: Option<Vec2> = None;
	
	for v in chr.vertex_data.iter() {
		if let Some(v) = *v {
			let v1 = v2i8_to_vec2(v);
			if let Some(v2) = pen_prev {
				let v1 = (v1.0 * s + p.0, v1.1 * s + p.1);
				let v2 = (v2.0 * s + p.0, v2.1 * s + p.1);
				draw_line(buf, v1, v2, c);
			}
		}
		pen_prev = v.map(v2i8_to_vec2);
	}
	
	let vofs = s * 4;
	let lh = (chr.left_hand as isize * s + p.0, p.1 + vofs);
	let rh = (chr.right_hand as isize * s + p.0, p.1 + vofs);
	draw_line(buf, lh, rh, 0x808080);
	draw_line(buf, (lh.0, lh.1 - s), (lh.0, lh.1 + s), 0x808080);
	draw_line(buf, (rh.0, rh.1 - s), (rh.0, rh.1 + s), 0x808080);
}

fn draw_hershey_str(buf: &mut Box<[u32]>, font: &[HersheyChar], st: &str, p: Vec2, s: isize, c: u32) {
	let mut ofs = (0, 0);
	for ch in st.bytes() {
		if ch == b'\n' {
			ofs = (0, ofs.1 + 32);
			continue;
		}
		
		let ch = (ch - b' ') as usize;
		let ch = font.get(ch);
		if let Some(ch) = ch {
			draw_hershey_char(buf, ch, (p.0 - (ch.left_hand as isize) + ofs.0 * s, p.1 + ofs.1 * s), s, c);
			ofs.0 += (ch.right_hand - ch.left_hand) as isize;
		}
	}
}
