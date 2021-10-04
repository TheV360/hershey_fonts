use std::num::ParseIntError;

/// This represents a single character in a Hershey font.
/// This won't actually reliably have *the actual codepoint it represents*
/// anywhere in its data, but it does have vertices!
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HersheyChar {
	/// Kinda... not used?
	/// 
	/// Supposed to be a unique number, but the fonts I've seen all put `12345`
	/// for every single character. I have no idea how you're supposed to
	/// derive what character's what with that scheme, lol.
	pub id: usize,
	
	/// Number of vertices in character.
	/// 
	/// ~~Surprisingly, it includes the left/right hand values
	/// in its total, despite me storing them separately.~~
	/// My API!!! I call the shots!!!
	pub vertex_num: usize,
	
	/// Probably some typography vocab I don't know.
	/// Seems to be a left bound?
	pub left_hand: i8,
	
	/// Probably some typography vocab I don't know.
	/// Seems to be a right bound?
	pub right_hand: i8,
	
	/// Here it is!
	pub vertex_data: Vec<Option<(i8, i8)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HersheyError {
	InvalidSpacing,
	InvalidAfterwards,
	InvalidId,
	TooShort,
	Parse(ParseIntError),
	MakeItStop,
}

impl HersheyChar {
	/// how do results work.
	pub fn new_from_str(s: &str) -> Result<Self, HersheyError> {
		assert!(s.len() >= 10, "doesn't seem to be long enough to be a Hershey character (at least 10 characters)");
		
		// col 0-4 = id number
		let first_space = s
			.find(|c: char| c.is_whitespace())
			.ok_or(HersheyError::InvalidSpacing)?;
		let first_alpha = s
			.find(|c: char| !(c.is_whitespace() || c.is_ascii_digit()))
			.ok_or(HersheyError::InvalidAfterwards)?;
		
		let id = &s[0..first_space];
		if id.len() > 8 { return Err(HersheyError::InvalidId); }
		
		let id = id.parse::<usize>()
			.map_err(HersheyError::Parse)?;
		
		// TODO: hmm does [1..] ever index out of bounds?
		let vertex_num = s[first_space..first_alpha][1..].trim();
		let vertex_num = str::parse::<usize>(vertex_num)
			.map_err(HersheyError::Parse)?;
		
		// It's my API!!! My rules!!!
		let vertex_num = vertex_num - 1;
		
		// "r"est of the "s"tring
		let r = &s[first_alpha..];
		
		let mut char_sludge = r[..2].chars();
		
		let left_hand = Self::parse_ascii_ofs(char_sludge.next()
			.ok_or(HersheyError::TooShort)?);
		let right_hand = Self::parse_ascii_ofs(char_sludge.next()
			.ok_or(HersheyError::TooShort)?);
		
		let r = &r[2..];
		
		let vertices: Vec<_> = r.chars().map(Self::parse_ascii_ofs).collect();
		let mut vertex_data = Vec::with_capacity(vertex_num);
		
		const PEN_UP: [i8; 2] = [-50, 0];
		// please don't look at this :(
		for pair in vertices.chunks(2) {
			if let [x, y] = *pair {
				vertex_data.push(
					if pair == PEN_UP { None }
					else { Some((x, y)) }
				);
			} else {
				unreachable!("fuck yo u");
			}
		}
		
		Ok(HersheyChar {
			id, vertex_num,
			left_hand, right_hand,
			vertex_data,
		})
	}
	
	/// Please give this only valid `char`s lol
	const fn parse_ascii_ofs(c: char) -> i8 {
		(c as i8) - (b'R' as i8) // ('R' is 82 in ASCII)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	/*
	#[derive(Debug)]
	enum DumpingError {
		Io(std::io::Error),
		Hershey(HersheyError),
	}
	impl From<std::io::Error> for DumpingError {
		fn from(e: std::io::Error) -> Self {
			DumpingError::Io(e)
		}
	}
	impl From<HersheyError> for DumpingError {
		fn from(e: HersheyError) -> Self {
			DumpingError::Hershey(e)
		}
	}
	
	#[test]
	fn dump_characters_lol() -> Result<(), DumpingError> {
		use std::fs::read_to_string;
		use std::fs::File;
		use std::io::Write;
		
		let jhf = read_to_string("../reference/futuram.jhf").expect(":(");
		let mut out = File::create("../reference/futuram.jhf.txt").expect("complaining");
		
		for line in jhf.trim().lines() {
			let c = HersheyChar::new_from_str(line)?;
			
			write!(out,
				"#{:5} ({:3} vtxs); ✋{:+3} 🤚{:+3} : ",
				c.id, c.vertex_num, c.left_hand, c.right_hand
			)?;
			for vtx in c.vertex_data {
				match vtx {
					Some((x, y)) => write!(out, "({:+3}, {:+3}) ", x, y)?,
					None => write!(out, "up ")?,
				}
			}
			writeln!(out, "end")?;
		}
		
		Ok(())
	}
	*/
	
	#[test]
	fn decode_a_space() -> Result<(), HersheyError> {
		// Space character
		let space = "12345  1JZ";
		let c = HersheyChar::new_from_str(space)?;
		
		assert_eq!(c.id, 12345);
		assert_eq!(c.vertex_num, 0);
		assert_eq!(c.left_hand, -8);
		assert_eq!(c.right_hand, 8);
		assert!(c.vertex_data.is_empty());
		
		Ok(())
	}
}
