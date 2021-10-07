use core::num::ParseIntError;

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
/// I don't know how to make error `enum`s that don't suck / aren't redundant.
pub enum HersheyError {
	InvalidSpacing,
	InvalidAfterwards,
	InvalidId,
	TooShort,
	Parse(ParseIntError),
	
	/// make it stop
	MakeItStop,
}

impl HersheyChar {
	/// how do results work.
	pub fn new_from_str(s: &str) -> Result<Self, HersheyError> {
		// oh dear god what did i do
		
		// col 0-4 = id number
		let id = s[0..5]
			.trim_start();
		let id = id.find(char::is_whitespace)
			.map(|i| &id[0..i])
			.unwrap_or(id)
			.parse::<usize>()
			.map_err(HersheyError::Parse)?;
		
		// dbg!(id);
		
		let s = s.trim(); // id is not gone, it's just invalid now
		
		let next_space = s
			.find(char::is_whitespace)
			.ok_or(HersheyError::InvalidSpacing)?
			.min(5);
		let first_alpha = s
			.find(|c: char| !(c.is_whitespace() || c.is_ascii_digit()))
			.ok_or(HersheyError::InvalidAfterwards)?;
		
		// println!("_: {}, a: {}", next_space, first_alpha);
		
		// col 5-7 = num of vertices
		let vertex_num = s[next_space..first_alpha].trim();
		let vertex_num = str::parse::<usize>(vertex_num)
			.map_err(HersheyError::Parse)?;
		
		// It's my API!!! My rules!!!
		let vertex_num = vertex_num.saturating_sub(1);
		
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
		for pair in vertices.chunks_exact(2) {
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
		
		let jhf = read_to_string("../reference/futuram.jhf")?;
		let mut out = File::create("../reference/futuram.jhf.txt")?;
		
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
		const CHR: &str = "12345  1JZ";
		
		let c = HersheyChar::new_from_str(CHR);
		let c = c?;
		
		assert_eq!(c.id, 12345);
		assert_eq!(c.vertex_num, 0);
		assert_eq!(c.left_hand, -8);
		assert_eq!(c.right_hand, 8);
		assert!(c.vertex_data.is_empty());
		
		Ok(())
	}
	
	#[test]
	fn wackier_space() -> Result<(), HersheyError> {
		// Incorrect/nonstandard/whatever-you-wanna-call-it
		
		// Space-obsessed space character
		const CHR: &str = "3    1 JZ";
		
		let c = HersheyChar::new_from_str(CHR);
		let c = c?;
		
		assert_eq!(c.id, 3);
		assert_eq!(c.vertex_num, 0);
		assert_eq!(c.left_hand, -8);
		assert_eq!(c.right_hand, 8);
		assert!(c.vertex_data.is_empty());
		
		Ok(())
	}
	
	#[test]
	fn something_else() -> Result<(), HersheyError> {
		const CHR: &str = " 2715 58I\\LKLJMHNGQFTFWGXHYJYLXNWOUPRQ RLKMKMJNHQGTGWHXJXLWNUORP RMIPG RUGXI RXMTP RRPRTSTSP RRXQYQZR[S[TZTYSXRX RRYRZSZSYRY";
		
		let c = HersheyChar::new_from_str(CHR);
		let c = c?;
		
		assert_eq!(c.id, 2715);
		assert_eq!(c.vertex_num, 58 - 1); // i forgot about me setting my own rules
		
		Ok(())
	}
	
	#[test]
	fn the_gosh_darned_8() -> Result<(), HersheyError> {
		const CHR: &str = "12345104H]SFPGOHNJNMOOQPTPWOYNZLZIYGWFSF RUFPG RPHOJONPO ROORP RSPWO RXNYLYIXG RYGUF RSFQHPJPNQP RTPVOWNXLXHWF RQPMQKSJUJXKZN[R[VZWYXWXTWRVQTP RRPMQ RNQLSKUKXLZ RKZP[VZ RVYWWWTVR RVQSP RQPOQMSLULXMZN[ RR[TZUYVWVSUQTP";
		
		let c = HersheyChar::new_from_str(CHR);
		let c = c?;
		
		assert_eq!(c.id, 12345);
		assert_eq!(c.vertex_num, 104 - 1);
		
		Ok(())
	}
}
