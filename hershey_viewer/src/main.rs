use std::fs::read_to_string;

fn main() -> std::io::Result<()> {
	println!("Hello, woeful world.");
	
	let jhf = read_to_string("reference/futuram.jhf")?;
	
	for l in jhf.lines() {
		
	}
	
	Ok(())
}
