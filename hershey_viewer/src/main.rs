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
				
			} else {
				
			}
			println!(
				"{} `{}`.",
				if success { "Loaded" } else { "Skipped" },
				path.to_string_lossy()
			);
		}
	}
	
	let mut buffer: Box<[u32]> = vec![0x201d1au32; WIDTH * HEIGHT]
		.into_boxed_slice();
	
	let mut win = Window::new(
		"Hello", WIDTH, HEIGHT,
		WindowOptions {
			scale: Scale::X1,
			..Default::default()
		}
	).unwrap();
	
	let mut cur_font = 0;
	let mut cur_char = 0;
	let mut mouse = (0, 0);
	
	while win.is_open() {
		if let Some((x, y)) = win.get_mouse_pos(minifb::MouseMode::Pass) {
			mouse = (x as isize, y as isize);
		}
		let mouse = (mouse.0 + 48, mouse.1);
		
		// let a = (1, y % HEIGHT as isize);
		// let b = (WIDTH as isize - 1, HEIGHT as isize - 4);
		
		// draw_line(&mut buffer, a, b, 0x363430);
		// y += 1;
		
		buffer.fill(0);
		
		let (a, b) =
		if win.get_mouse_down(minifb::MouseButton::Left) {
			((64, 96), mouse)
		} else {
			(mouse, (64, 96))
		};
		
		draw_hershey_char(&mut buffer, &fonts[cur_font].1[cur_char], a, 3, 0xFFEFEA);
		draw_hershey_str(&mut buffer, &fonts[5].1, &format!("{} (#{}) ch{}", fonts[cur_font].0, cur_font, cur_char), (a.0 + 128, a.1), 1, 0xFFEFEA);
		draw_hershey_str(&mut buffer, &fonts[cur_font].1, "the quick brown fox jumped\nover the lazy dog\nTHE QUICK BROWN FOX JUMPED\nOVER THE LAZY DOG\n(0123456789)\n<HTML> ? @", b, 2, 0xFFEFEA);
		
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
		let i = vec2_to_index(p);
		*buf.get_unchecked_mut(i) = c;
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
