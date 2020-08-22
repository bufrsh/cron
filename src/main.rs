use std::fmt;
use std::io;
use std::thread;
use std::io::Write;
use std::io::Read;
use std::io::Error;
use std::io::ErrorKind;
use std::net::{TcpListener,TcpStream,Shutdown};
use std::time::Duration;

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
enum Token {
	Delim,
	Asterik,
	Comma,
	Num(char),
	Dash,
	Slash,
	Alpha(char),
	EOF,
}

impl Token {
	fn from_char(c: char) -> Option<Token> {
		match c {
			'*' => Some(Token::Asterik),
			',' => Some(Token::Comma),
			'0'..='9' => Some(Token::Num(c)),
			'-' => Some(Token::Dash),
			'/' => Some(Token::Slash),
			' '|'\t'|'\n'|'\r'|'\0' => Some(Token::Delim),
			'a'..='z'|'A'..='Z' => Some(Token::Alpha(c)),
			_ => None,
		}
	}

	fn is_legal_after(&self, t: &Token) -> bool {
		match self {
			Token::Asterik => {
				match t {
					Token::Delim|Token::Comma => true,
					_ => false,
				}
			},
			Token::Comma => {
				match t {
					Token::Asterik|Token::Num(_)|Token::Alpha(_) => true,
					_ => false,
				}
			},
			Token::Num(_) => {
				match t {
					Token::Delim|Token::Num(_)|Token::Slash|Token::Comma|Token::Dash => true,
					_ => false,
				}
			},
			Token::Dash => {
				match t {
					Token::Num(_)|Token::Alpha(_) => true,
					_ => false,
				}
			},
			Token::Slash => {
				match t {
					Token::Num(_)|Token::Asterik|Token::Alpha(_) => true,
					_ => false,
				}
			},
			Token::Delim|Token::EOF => {
				match t {
					Token::Delim|Token::Asterik|Token::Num(_)|Token::Alpha(_) => true,
					_ => false,
				}
			},
			Token::Alpha(_) => {
				match t {
					Token::Delim|Token::Alpha(_)|Token::Slash|Token::Comma|Token::Dash => true,
					_ => false,
				}
			}
		}
	}
}

struct TokenIter {
	v: Vec<char>,
	idx: usize,
	start: bool,
	prev: Token,
}

impl TokenIter {
	fn new(v: Vec<char>) -> Self {
		Self {v, idx: 0, start: true, prev: Token::Delim}
	}
}

impl Iterator for TokenIter {
	type Item = Token;

	fn next(&mut self) -> Option<Self::Item> {
		if self.start {
			self.start = false;
			//println!("{:?}",Token::Delim);
			return Some(Token::Delim);
		}
		if self.idx > self.v.len() {
			return None;
		}
		if self.idx == self.v.len() {
			self.idx += 1;
			if Token::EOF.is_legal_after(&self.prev) {
				//println!("{:?}",Token::EOF);
				return Some(Token::EOF);
			} else {
				println!("Unexpected end");
				return None;
			}
		}
		let tok = match Token::from_char(self.v[self.idx]) {
			Some(t) => t,
			None => {
				println!("Unknown token {:x?}",self.v[self.idx]);
				return None;
			},
		};
		if tok.is_legal_after(&self.prev) {
			self.prev = tok;
			self.idx += 1;
			//println!("{:?}",tok);
			return Some(tok);
		} else {
			println!("Illegal token {}",self.v[self.idx]);
			return None;
		}
	}
}

#[derive(Clone)]
#[derive(Copy)]
enum Pattern<T> {
	At(T),
	Every(usize),
	Range(T,T,usize),
}

impl<T: fmt::Display> fmt::Display for Pattern<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Pattern::At(t) => write!(f,"{:02}",t),
			_ => Ok(()),
		}
	}
}

struct Minute{}
struct Hour{}
struct DayOfMonth{}
struct Month{}
struct DayOfWeek{}

trait FieldDisp {
	fn typestr(&self) -> &str;
	fn atstr(&self) -> &str;
	fn atword(&self) -> &str;
	fn everystr(&self) -> &str;
}

impl FieldDisp for Minute {
	fn typestr(&self) -> &'static str {
		"minute"
	}

	fn atstr(&self) -> &'static str {
		"at minute"
	}

	fn atword(&self) -> &'static str {
		"at"
	}

	fn everystr(&self) -> &'static str {
		"at every minute"
	}
}

impl FieldDisp for Hour {
	fn typestr(&self) -> &'static str {
		"hour"
	}

	fn atstr(&self) -> &'static str {
		"past hour"
	}

	fn atword(&self) -> &'static str {
		"past"
	}

	fn everystr(&self) -> &'static str {
		"past every hour"
	}
}

impl FieldDisp for DayOfMonth {
	fn typestr(&self) -> &'static str {
		"day-of-month"
	}

	fn atstr(&self) -> &'static str {
		"on day-of-month"
	}

	fn atword(&self) -> &'static str {
		"on"
	}

	fn everystr(&self) -> &'static str {
		""
	}
}

impl FieldDisp for Month {
	fn typestr(&self) -> &'static str {
		"month"
	}

	fn atstr(&self) -> &'static str {
		"in"
	}

	fn atword(&self) -> &'static str {
		"in"
	}

	fn everystr(&self) -> &'static str {
		""
	}
}

impl FieldDisp for DayOfWeek {
	fn typestr(&self) -> &'static str {
		"day-of-week"
	}

	fn atstr(&self) -> &'static str {
		"on"
	}

	fn atword(&self) -> &'static str {
		"on"
	}

	fn everystr(&self) -> &'static str {
		""
	}
}

trait Convert<T> {
	fn from_num(&self, n: usize) -> Result<T,Error>;
	fn alpha_to_num(&self, s: &str) -> Result<usize,Error>;
}

impl Convert<usize> for Minute {
	fn from_num(&self, n: usize) -> Result<usize,Error> {
		match n {
			_ if n < 60 => Ok(n),
			_ => Err(Error::new(ErrorKind::InvalidInput,format!("Invalid MINUTE value {}",n))),
		}
	}

	fn alpha_to_num(&self, s: &str) -> Result<usize,Error> {
		Err(Error::new(ErrorKind::InvalidInput,format!("Invalid MINUTE value {}",s)))
	}
}

impl Convert<usize> for Hour {
	fn from_num(&self, n: usize) -> Result<usize,Error> {
		match n {
			_ if n < 24 => Ok(n),
			_ => Err(Error::new(ErrorKind::InvalidInput,format!("Invalid HOUR value {}",n))),
		}
	}

	fn alpha_to_num(&self, s: &str) -> Result<usize,Error> {
		Err(Error::new(ErrorKind::InvalidInput,format!("Invalid HOUR value {}",s)))
	}
}

impl Convert<usize> for DayOfMonth {
	fn from_num(&self, n: usize) -> Result<usize,Error> {
		match n {
			1..=23 => Ok(n),
			_ => Err(Error::new(ErrorKind::InvalidInput,format!("Invalid DAY-OF-MONTH value {}",n))),
		}
	}

	fn alpha_to_num(&self, s: &str) -> Result<usize,Error> {
		Err(Error::new(ErrorKind::InvalidInput,format!("Invalid DAY-OF-MONTH value {}",s)))
	}
}

impl Convert<&str> for Month {
	fn from_num(&self, n: usize) -> Result<&'static str,Error> {
		match n {
			1 => Ok("JAN"),
			2 => Ok("FEB"),
			3 => Ok("MAR"),
			4 => Ok("APR"),
			5 => Ok("MAY"),
			6 => Ok("JUN"),
			7 => Ok("JUL"),
			8 => Ok("AUG"),
			9 => Ok("SEP"),
			10 => Ok("OCT"),
			11 => Ok("NOV"),
			12 => Ok("DEC"),
			_ => Err(Error::new(ErrorKind::InvalidInput,format!("Invalid MONTH value {}",n))),
		}
	}

	fn alpha_to_num(&self, s: &str) -> Result<usize,Error> {
		let names = ["JAN","FEB","MAR","APR","MAY","JUN","JUL","AUG","SEP","OCT","NOV","DEC"];
		let idx = names.iter().position(|&name| name == s);
		match idx {
			Some(i) => Ok(i+1),
			None => Err(Error::new(ErrorKind::InvalidInput,format!("Invalid DAY-OF-WEEK value {}",s))),
		}
	}
}

impl Convert<&str> for DayOfWeek {
	fn from_num(&self, n: usize) -> Result<&'static str,Error> {
		match n {
			0 => Ok("SUN"),
			1 => Ok("MON"),
			2 => Ok("TUE"),
			3 => Ok("WED"),
			4 => Ok("THU"),
			5 => Ok("FRI"),
			6 => Ok("SAT"),
			_ => Err(Error::new(ErrorKind::InvalidInput,format!("Invalid DAY-OF-WEEK value {}",n))),
		}
	}

	fn alpha_to_num(&self, s: &str) -> Result<usize,Error> {
		let names = ["SUN","MON","TUE","WED","THU","FRI","SAT"];
		let idx = names.iter().position(|&name| name == s);
		match idx {
			Some(i) => Ok(i),
			None => Err(Error::new(ErrorKind::InvalidInput,format!("Invalid DAY-OF-WEEK value {}",s))),
		}
	}
}

struct Field<F,P> {
	typ: F,
	pats: Vec<Pattern<P>>,
}

impl<'a, F: Convert<P>, P> Field<F,P>
{
	fn new(t: F) -> Self {
		Self { typ: t, pats: Vec::new() }
	}

	fn get_num(s: &[Token]) -> Result<(&[Token],usize),Error> {
		let idx = s.iter()
			.position(|t| !matches!(t,Token::Num(_))).unwrap();
		let numstr: String = s[..idx].iter()
			.map(|t| match t {
				Token::Num(c) => c,
				_ => panic!(),
			})
			.collect();
		let num = match numstr.parse::<usize>() {
			Ok(n) => n,
			Err(_) => return Err(Error::new(ErrorKind::InvalidInput,"Error parsing a number")),
		};
		return Ok((&s[idx..],num));
	}

	fn get_val<'b>(&self, s: &'b [Token]) -> Result<(&'b [Token],usize),Error> {
		//see if a numerical value is given
		if let Ok((r,n)) = Field::<F,P>::get_num(s) {
			return Ok((r,n));
		};
		//try getting an alphabetical value
		let idx = s.iter()
			.position(|t| !matches!(t,Token::Alpha(_))).unwrap();
		let valstr: String = s[..idx].iter()
			.map(|t| match t {
				Token::Alpha(c) => c.to_ascii_uppercase(),
				_ => panic!(),
			})
			.collect();
		match self.typ.alpha_to_num(&valstr) {
			Ok(num) => Ok((&s[idx..],num)),
			Err(e) => Err(Error::new(ErrorKind::InvalidInput,e)),
		}
	}

	//caller should check if Delim is at idx, and call again if not
	fn from_toks(&mut self, s: &'a [Token]) -> Result<&'a [Token],Error> {
		if matches!(s[0],Token::Asterik) {
			match s[1] {
				Token::EOF|Token::Delim|Token::Comma => {
					self.pats.push(Pattern::Every(1));
					return Ok(&s[1..]);
				},
				Token::Slash => {
					let (ret,num) = Field::<F,P>::get_num(&s[2..])?;
					self.pats.push(Pattern::Every(num));
					return Ok(ret);
				}
				_ => return Err(Error::new(ErrorKind::InvalidInput,"Illegal char after *")),
			}
		}
		let (ret,num1) = self.get_val(s)?;
		match ret[0] {
			Token::EOF|Token::Delim|Token::Comma => {
				self.pats.push(Pattern::At(self.typ.from_num(num1)?));
				return Ok(ret);
			},
			Token::Dash => {
				let (ret2,num2) = self.get_val(&ret[1..])?;
				if num2 <= num1 {
					return Err(Error::new(ErrorKind::InvalidInput,format!("Range start ({}) cannot be bigger than or equal to end ({})",num1,num2)));
				}
				match ret2[0] {
					Token::EOF|Token::Delim|Token::Comma => {
						self.pats.push(Pattern::Range(self.typ.from_num(num1)?,self.typ.from_num(num2)?,1));
						return Ok(ret2);
					},
					Token::Slash => {
						let (rets,step) = Field::<F,P>::get_num(&ret2[1..])?;
						self.pats.push(Pattern::Range(self.typ.from_num(num1)?,self.typ.from_num(num2)?,step));
						return Ok(rets);
					},
					_ => return Err(Error::new(ErrorKind::InvalidInput,"Invalid char after range end number")),
				}
			},
			_ => return Err(Error::new(ErrorKind::InvalidInput,"Invalid char after number")),
		}
	}
}

trait Printer<T> {
	fn print(&self, out: &mut T) -> Result<(),Error>;
}

impl<P: fmt::Display, F: FieldDisp, T: io::Write> Printer<T> for Field<F,P> {
	fn print(&self, out: &mut T) -> Result<(),Error> {
		for pat in self.pats.iter() {
			match pat {
				Pattern::At(p) => write!(out,"{} {:02}\n",self.typ.atstr(),p)?,
				Pattern::Every(p) => {
					match p%20 {
						1 => {
							match self.typ.everystr() {
								"" => (),
								_ => write!(out,"{}\n",self.typ.everystr())?,
							}
						},
						2 => write!(out,"{} every {}nd {}\n",self.typ.atword(),p,self.typ.typestr())?,
						3 => write!(out,"{} every {}rd {}\n",self.typ.atword(),p,self.typ.typestr())?,
						_ => write!(out,"{} every {}th {}\n",self.typ.atword(),p,self.typ.typestr())?,
					}
				},
				Pattern::Range(fr,to,step) => {
					match step%20 {
						1 => write!(out,"{} every {} from {} to {}\n",self.typ.atword(),self.typ.typestr(),fr,to)?,
						2 => write!(out,"{} every {}nd {} from {} to {}\n",self.typ.atword(),step,self.typ.typestr(),fr,to)?,
						3 => write!(out,"{} every {}rd {} from {} to {}\n",self.typ.atword(),step,self.typ.typestr(),fr,to)?,
						_ => write!(out,"{} every {}th {} from {} to {}\n",self.typ.atword(),step,self.typ.typestr(),fr,to)?,
					}
				}
			}
		}
		Ok(())
	}
}

enum FieldType {
	Minute,
	Hour,
	DayOfMonth,
	Month,
	DayOfWeek,
	End,
}

fn respond(req: String, out: &mut TcpStream) -> Result<(),Error> {
	//handle @xxxx inputs by converting them to equivalent CRON expressions
	let mapstr = match req.lines().nth(0).unwrap() {
		"@yearly"|"@annually" => String::from("0 0 1 1 *"),
		"@monthly" => String::from("0 0 1 * *"),
		"@weekly" => String::from("0 0 * * 0"),
		"@daily" => String::from("0 0 * * *"),
		"@hourly" => String::from("0 * * * *"),
		"@reboot" => {
			//no need to translate this
			write!(out,"Run after reboot")?;
			return Ok(());
		}
		_ => req,
	};

	let tokiter = TokenIter::new(mapstr.chars().collect());
	if !matches!(tokiter.last().unwrap(),Token::EOF) {
		return Err(Error::new(ErrorKind::InvalidInput,"CRON syntax error"));
	}

	let mut min: Field<Minute,usize> = Field::new(Minute{});
	let mut hrs: Field<Hour,usize> = Field::new(Hour{});
	let mut dom: Field<DayOfMonth,usize> = Field::new(DayOfMonth{});
	let mut mon: Field<Month,&str> = Field::new(Month{});
	let mut dow: Field<DayOfWeek,&str> = Field::new(DayOfWeek{});

	let mut state = FieldType::Minute;

	let toks: Vec<Token> = TokenIter::new(mapstr.chars().collect()).collect();
	let mut ptr = &toks[..];
	'outer: loop {
		ptr = match ptr[0] {
			//condense multiple Delims into one
			Token::Delim => loop {
				ptr = match ptr[0] {
					Token::Delim => &ptr[1..],
					Token::EOF => {
						break 'outer;
					},
					_ => break ptr,
				}
			},
			Token::Comma => &ptr[1..],
			Token::EOF => {
				break;
			},
			_ => {
				write!(out,"Pattern doesn't start/end on correct token\n")?;
				return Err(Error::new(ErrorKind::InvalidInput,"Bad CRON"));
			}
		};
		ptr = match state {
			FieldType::Minute => min.from_toks(ptr)?,
			FieldType::Hour => hrs.from_toks(ptr)?,
			FieldType::DayOfMonth => dom.from_toks(ptr)?,
			FieldType::Month => mon.from_toks(ptr)?,
			FieldType::DayOfWeek => dow.from_toks(ptr)?,
			_ => return Err(Error::new(ErrorKind::InvalidInput,"Bad CRON")),
		};
		if matches!(ptr[0],Token::Delim|Token::EOF) {
			state = match state {
				FieldType::Minute => FieldType::Hour,
				FieldType::Hour => FieldType::DayOfMonth,
				FieldType::DayOfMonth => FieldType::Month,
				FieldType::Month => FieldType::DayOfWeek,
				FieldType::DayOfWeek => FieldType::End,
				_ => return Err(Error::new(ErrorKind::InvalidInput,"Bad CRON")),
			};
		}
	}

	if !matches!(state,FieldType::End) {
		return Err(Error::new(ErrorKind::InvalidInput,"Incomplete CRON"));
	}

	out.write(b"Run\n")?;

	if min.pats.len() == 1 && hrs.pats.len() == 1 && matches!(min.pats[0],Pattern::At(_)) && matches!(hrs.pats[0],Pattern::At(_)) {
		write!(out,"at {}:{}\n",hrs.pats[0],min.pats[0])?;
	} else {
		min.print(out)?;
		hrs.print(out)?;
	}

	let mut dom_mon_printed = false;
	if dom.pats.len() == 1 && mon.pats.len() == 1 && matches!(dom.pats[0],Pattern::At(_)) && matches!(mon.pats[0],Pattern::At(_)) {
		write!(out,"on {} {}\n",mon.pats[0],dom.pats[0])?;
		dom_mon_printed = true;
	}
	//https://pubs.opengroup.org/onlinepubs/9699919799/utilities/crontab.html
	let is_dom_defined = dom.pats.iter().find(|d| !matches!(d,Pattern::Every(1))).is_some();
	let is_mon_defined = mon.pats.iter().find(|d| !matches!(d,Pattern::Every(1))).is_some();
	let is_dom_mon_defined = is_dom_defined || is_mon_defined;
	let is_dow_defined = dow.pats.iter().find(|d| !matches!(d,Pattern::Every(1))).is_some();
	match(is_dom_mon_defined,is_dow_defined) {
		(true,true) => {
			if !dom_mon_printed {
				dom.print(out)?;
				mon.print(out)?;
			}
			out.write(b"and\n")?;
			dow.print(out)?;
		},
		(true,false) => {
			if !dom_mon_printed {
				dom.print(out)?;
				mon.print(out)?;
			}
		},
		(false,true) => {
			if !dom_mon_printed {
				dom.print(out)?;
			}
			dow.print(out)?;
		},
		(false,false) => {
			if !dom_mon_printed {
				mon.print(out)?;
			}
		},
	}

	Ok(())
}

fn main() {
	let srv = TcpListener::bind("0.0.0.0:6000").unwrap();
	println!("Listening");

	for stream in srv.incoming() {
		match stream {
			Ok(mut stream) => {
				thread::spawn(move || {
					let mut buf = [0u8; 64];
					let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
					match stream.read(&mut buf) {
						Ok(_) => {
							let req = String::from_utf8_lossy(&buf);
							println!("{}",req);
							match respond(req.to_string(),&mut stream) {
								Ok(_) => {
									let _ = write!(stream,"\n\n\u{1f426} \u{001b}[36;1m@bufrsh\u{001b}[0m ");
								},
								Err(e) => {
									let _ = write!(stream,"{}",e.to_string());
								},
							}
							let _ = stream.shutdown(Shutdown::Both);
						},
						Err(e) => println!("read ERR: {}",e),
					}
				});
			},
			Err(e) => {
				println!("connect ERR: {}",e);
			}
		}
	}
}

