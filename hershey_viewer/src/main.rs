use std::default;
use std::fs::{File, read_dir, read_to_string};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use minifb::{Scale, Window, WindowOptions, Key};
use nanorand::{Rng, WyRand};

use hershey_reader::*;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;

mod bresenham;
use bresenham::*;

fn main() -> std::io::Result<()> {
	let look_in = &std::env::args().nth(1)
		.unwrap_or_else(||"hershey_viewer/fonts/".to_string());
	let mut fonts: Vec<(String, Vec<HersheyChar>)> = Vec::new();
	
	for entry in read_dir(look_in)? {
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
							Err(x) => panic!("`{}`\nCouldn't parse, {:?}", line, x),
						}
					}
					
					let filename = path.file_name()
						.map(|x| x.to_string_lossy().into_owned())
						.unwrap_or_else(||"oops".to_string());
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
	if fonts.is_empty() {
		println!("i need some fonts to view.\nsupply a path to some .jhf fonts as my first arg?");
		return Ok(()); // sloppy i know
	}
	
	#[derive(Debug, Clone, Copy, Default)]
	struct BoundBox {
		left: f64, right: f64,
		top: f64, bottom: f64,
	}
	impl BoundBox {
		fn add_vec(self, v: (f64, f64)) -> Self {
			BoundBox {
				left:   self.left  .min(v.0),
				right:  self.right .max(v.0),
				top:    self.top   .min(v.1),
				bottom: self.bottom.max(v.1),
			}
		}
		fn add(self, o: Self) -> Self {
			BoundBox {
				left:   self.left  .min(o.left  ),
				right:  self.right .max(o.right ),
				top:    self.top   .min(o.top   ),
				bottom: self.bottom.max(o.bottom),
			}
		}
		fn div_scalar(self, s: f64) -> Self {
			BoundBox {
				left:   self.left   / s,
				right:  self.right  / s,
				top:    self.top    / s,
				bottom: self.bottom / s,
			}
		}
		fn total(self, o: Self) -> Self {
			BoundBox {
				left:   self.left   + o.left  ,
				right:  self.right  + o.right ,
				top:    self.top    + o.top   ,
				bottom: self.bottom + o.bottom,
			}
		}
	}
	
	/*
	let mut bb_totaltotal = BoundBox::default();
	
	for (fname, font) in fonts.iter() {
		let mut bb_runtotal = BoundBox::default();
		let mut bb = BoundBox::default();
		
		for chr in font.iter() {
			let bb_chr = chr.vertex_data.iter()
				.flatten()
				.map(|(x, y)| (*x as f64, *y as f64))
				.fold(BoundBox::default(), BoundBox::add_vec);
			
			bb = bb.add(bb_chr);
			bb_runtotal = bb_runtotal.total(bb_chr);
		}
		
		bb_runtotal = bb_runtotal.div_scalar(font.len() as f64);
		bb_totaltotal = bb_totaltotal.add(bb);
		
		dbg!(fname, bb, bb_runtotal);
	}
	dbg!(bb_totaltotal);
	*/
	
	let mut rng = WyRand::new();
	
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
			
			(_, Key::Apostrophe, _) => Some('\''),
			(_, Key::Backquote, _) => Some('`'),
			(_, Key::Backslash, _) => Some('\\'),
			(_, Key::Comma, _) => Some(','),
			(_, Key::Equal, _) => Some('='),
			(_, Key::LeftBracket, _) => Some('['),
			(_, Key::Minus, _) => Some('-'),
			(_, Key::Period, _) => Some('.'),
			(_, Key::RightBracket, _) => Some(']'),
			(_, Key::Semicolon, _) => Some(';'),
			(_, Key::Slash, _) => Some('/'),
			
			(_, Key::Backspace, _) => Some('\x08'),
			(_, Key::Delete, _) => Some('\x7F'),
			(_, Key::Enter, _) => Some('\n'),
			
			_ => None
		}
	}
	
	let ui_font = &fonts.get(5.min(fonts.len()));
	
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
				if let Some(ch) = key_to_char(k, shift) {
					if ch == '\x08' {
						specimen.pop();
					} else {
						specimen.push(ch);
					}
				}
			}
		}
		
		// const COOL_COLORS: [u32; 8] = [0xFFFFFF, 0xFB4934, 0xFE8019, 0xFABD2F, 0xB8BB26, 0x8EC07C, 0x83A598, 0xD3869B];
		const COOL_COLORS: [u32; 8] = [0xFFFFFF, 0xCC241D, 0xD65D0E, 0xD79921, 0x98971A, 0x689D6A, 0x458588, 0xB16286];
		
		draw_hershey_char(&mut buffer, &mut rng, &fonts[cur_font].1[cur_char], a, 3, 0xFFEFEA);
		draw_hershey_str(&mut buffer, &mut rng, &ui_font.unwrap().1, &format!("{} (#{}) ch{}", fonts[cur_font].0, cur_font, cur_char), (a.0 + 128, a.1), 1, 0xFFEFEA);
		
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
		draw_hershey_str(&mut buffer, &mut rng, &fonts[cur_font].1, &specimen, b, 2, COOL_COLORS[0]);
		
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

fn draw_hershey_char(buf: &mut Box<[u32]>, rng: &mut WyRand, chr: &HersheyChar, p: Vec2, s: isize, c: u32) {
	let mut pen_prev: Option<Vec2> = None;
	
	for mut v in chr.vertex_data.iter().map(|i|i.map(v2i8_to_vec2)) {
		if let Some(v1) = v {
			// let vr = (rng.generate_range(-1isize..=1), rng.generate_range(-1isize..=1));
			let vr = (0, 0);
			let v1 = (v1.0 * s + p.0 + vr.0, v1.1 * s + p.1 + vr.1);
			v = Some(v1);
			if let Some(v2) = pen_prev {
				draw_line(buf, v1, v2, c);
			}
		}
		// pen_prev = v.map(v2i8_to_vec2);
		pen_prev = v;
	}
	
	/*let vofs = s * 4;
	let lh = (chr.left_hand as isize * s + p.0, p.1 + vofs);
	let rh = (chr.right_hand as isize * s + p.0, p.1 + vofs);
	draw_line(buf, lh, rh, 0x808080);
	draw_line(buf, (lh.0, lh.1 - s), (lh.0, lh.1 + s), 0x808080);
	draw_line(buf, (rh.0, rh.1 - s), (rh.0, rh.1 + s), 0x808080);*/
}

fn draw_hershey_str(buf: &mut Box<[u32]>, rng: &mut WyRand, font: &[HersheyChar], st: &str, p: Vec2, s: isize, c: u32) {
	let mut ofs = (0, 0);
	for ch in st.bytes() {
		if ch == b'\n' {
			ofs = (0, ofs.1 + 32);
			continue;
		}
		
		let ch = (ch - b' ') as usize;
		let ch = font.get(ch);
		if let Some(ch) = ch {
			let w = (ch.right_hand - ch.left_hand) as isize;
			draw_hershey_char(buf, rng, ch, (p.0 - ch.left_hand as isize + ofs.0 * s, p.1 + ofs.1 * s), s, c);
			ofs.0 += w;
		}
	}
}
