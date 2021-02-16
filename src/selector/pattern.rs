/*
*
* all: *
* id: #{identity}
* class: .{identity}
* attribute: [{identity}{rule##"(^|*~$)?=('")"##}]
*/
use crate::utils::{
	chars_to_int, divide_isize, is_char_available_in_key, to_static_str, RoundType,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fmt::Debug, usize};

pub type FromParamsFn =
	Box<dyn Fn(&str, &str) -> Result<Box<dyn Pattern>, String> + Send + 'static>;
lazy_static! {
	static ref REGEXS: Mutex<HashMap<&'static str, Arc<Regex>>> = Mutex::new(HashMap::new());
	static ref PATTERNS: Mutex<HashMap<&'static str, FromParamsFn>> = Mutex::new(HashMap::new());
}

fn no_implemented(name: &str) -> ! {
	panic!("No supported Pattern type '{}' found", name);
}

pub type MatchedData = HashMap<&'static str, &'static str>;
#[derive(Debug, Default, Clone)]
pub struct Matched {
	pub chars: Vec<char>,
	pub name: &'static str,
	pub data: MatchedData,
}

pub trait Pattern: Send + Sync + Debug {
	fn matched(&self, chars: &[char]) -> Option<Matched>;
	// check if nested pattern
	fn is_nested(&self) -> bool {
		false
	}
	// get a pattern trait object
	fn from_params(s: &str, _p: &str) -> Result<Box<dyn Pattern>, String>
	where
		Self: Sized + Send + 'static,
	{
		no_implemented(s);
	}
}

impl Pattern for char {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		let ch = chars[0];
		if *self == ch {
			return Some(Matched {
				chars: vec![ch],
				..Default::default()
			});
		}
		None
	}
}

impl Pattern for &[char] {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		let total = self.len();
		if total > chars.len() {
			return None;
		}
		let mut result: Vec<char> = Vec::with_capacity(total);
		for (index, &ch) in self.iter().enumerate() {
			let cur = chars
				.get(index)
				.expect("Pattern for slice char's length must great than target's chars.z");
			if ch == *cur {
				result.push(ch);
			} else {
				return None;
			}
		}
		Some(Matched {
			chars: result,
			..Default::default()
		})
	}
}

impl Pattern for Vec<char> {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		self.as_slice().matched(chars)
	}
}

/// Identity
#[derive(Debug, Default)]
pub struct Identity(bool);

impl Pattern for Identity {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		let mut result: Vec<char> = Vec::with_capacity(5);
		let first = chars[0];
		let name: &str = "identity";
		if !(first.is_ascii_alphabetic() || first == '_') {
			if self.0 {
				// optional
				return Some(Matched {
					name,
					..Default::default()
				});
			}
			return None;
		}
		for &c in chars {
			if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
				result.push(c);
			} else {
				break;
			}
		}
		Some(Matched {
			chars: result,
			name,
			..Default::default()
		})
	}
	// from_str
	fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
		if s == "?" {
			Ok(Box::new(Identity(true)))
		} else {
			check_params_return(&[p], || Box::new(Identity::default()))
		}
	}
}
/// AttrKey
#[derive(Debug, Default)]
pub struct AttrKey;

impl Pattern for AttrKey {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		let mut result = Vec::with_capacity(5);
		for ch in chars {
			if is_char_available_in_key(ch) {
				result.push(*ch);
			} else {
				break;
			}
		}
		if !result.is_empty() {
			return Some(Matched {
				chars: result,
				name: "attr_key",
				..Default::default()
			});
		}
		None
	}
	// from_params
	fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
		check_params_return(&[s, p], || Box::new(AttrKey::default()))
	}
}
/// Spaces
#[derive(Debug, Default)]
pub struct Spaces(usize);

impl Pattern for Spaces {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		let mut result: Vec<char> = Vec::with_capacity(2);
		for ch in chars {
			if ch.is_ascii_whitespace() {
				result.push(*ch);
			} else {
				break;
			}
		}
		if result.len() >= self.0 {
			return Some(Matched {
				chars: result,
				name: "spaces",
				..Default::default()
			});
		}
		None
	}
	fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
		let mut min_count = 0;
		if !p.is_empty() {
			return Err(format!("Spaces not support param '{}'", p));
		}
		if !s.trim().is_empty() {
			let rule: [Box<dyn Pattern>; 3] = [Box::new('('), Box::new(Index::default()), Box::new(')')];
			let chars: Vec<char> = s.chars().collect();
			let (result, _, _, match_all) = exec(&rule, &chars);
			if !match_all {
				return Err(format!("Wrong 'Spaces{}'", s));
			}
			min_count = chars_to_int(&result[1].chars).map_err(|e| e.to_string())?;
		}
		Ok(Box::new(Spaces(min_count)))
	}
}

/// Index
#[derive(Debug, Default)]
pub struct Index;

impl Pattern for Index {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		let first = chars[0];
		let mut result = Vec::with_capacity(2);
		let numbers = '0'..'9';
		if numbers.contains(&first) {
			result.push(first);
			if first != '0' {
				for ch in &chars[1..] {
					if numbers.contains(ch) {
						result.push(*ch);
					}
				}
			}
			return Some(Matched {
				chars: result,
				name: "index",
				..Default::default()
			});
		}
		None
	}
	fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
		check_params_return(&[s, p], || Box::new(Index::default()))
	}
}

/// `Nth`
/// 2n/+2n+1/2n-1/-2n+1/+0/-1/2
#[derive(Debug, Default)]
pub struct Nth;

impl Pattern for Nth {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		let rule: RegExp = RegExp {
			cache: true,
			context: r#"^(?:([-+])?([0-9]|[1-9]\d+)?n(?:\s*([+-])\s*([0-9]|[1-9]\d+))?|([-+])?([0-9]|[1-9]\d+))"#,
		};
		let mut data = HashMap::with_capacity(2);
		let mut matched_chars: Vec<char> = Vec::new();
		if let Some(v) = Pattern::matched(&rule, chars) {
			let rule_data = v.data;
			// when the group index 6,
			let only_index = rule_data.get("6").is_some();
			let index_keys = if only_index { ("6", "5") } else { ("4", "3") };
			// set index
			if let Some(index) = Nth::get_number(&rule_data, index_keys, None) {
				data.insert("index", index);
			}
			// also has `n`
			if !only_index {
				if let Some(n) = Nth::get_number(&rule_data, ("2", "1"), Some("1")) {
					data.insert("n", n);
				}
			}
			matched_chars = v.chars;
		} else {
			// maybe 'even' or 'odd'
			let even = vec!['e', 'v', 'e', 'n'];
			let odd = vec!['o', 'd', 'd'];
			if Pattern::matched(&even, chars).is_some() {
				data.insert("n", "2");
				data.insert("index", "0");
				matched_chars = even;
			} else if Pattern::matched(&odd, chars).is_some() {
				data.insert("n", "2");
				data.insert("index", "1");
				matched_chars = odd;
			}
		}
		if !data.is_empty() {
			return Some(Matched {
				name: "nth",
				data,
				chars: matched_chars,
			});
		}
		None
	}
	// from params to pattern
	fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
		check_params_return(&[s, p], || Box::new(Nth::default()))
	}
}

impl Nth {
	fn get_number(data: &MatchedData, keys: (&str, &str), def: Option<&str>) -> Option<&'static str> {
		const MINUS: &str = "-";
		if let Some(&idx) = data.get(keys.0).or_else(|| def.as_ref()) {
			let mut index = String::from(idx);
			if let Some(&op) = data.get(keys.1) {
				if op == MINUS {
					index = String::from(op) + &index;
				}
			}
			return Some(to_static_str(index));
		}
		None
	}
	// get indexs allowed
	pub fn get_allowed_indexs(n: Option<&str>, index: Option<&str>, total: usize) -> Vec<usize> {
		// has n
		if let Some(n) = n {
			let n = n.parse::<isize>().unwrap();
			let index = index
				.map(|index| index.parse::<isize>().unwrap())
				.unwrap_or(0);
			// n == 0
			if n == 0 {
				if index > 0 {
					let index = index as usize;
					if index <= total {
						return vec![index - 1];
					}
				}
				return vec![];
			}
			// n < 0 or n > 0
			let mut start_loop: isize;
			let end_loop: isize;
			if n < 0 {
				// -2n - 1/ -2n + 0
				if index <= 0 {
					return vec![];
				}
				// -2n + 1
				if index <= -n {
					let index = index as usize;
					if index <= total {
						return vec![index - 1];
					}
					return vec![];
				}
				start_loop = divide_isize(index - (total as isize), -n, RoundType::Ceil);
				end_loop = divide_isize(index - 1, -n, RoundType::Floor);
			} else {
				// n > 0
				start_loop = divide_isize(1 - index, n, RoundType::Ceil);
				end_loop = divide_isize((total as isize) - index, n, RoundType::Floor);
			}
			// set start_loop min 0
			if start_loop < 0 {
				start_loop = 0;
			}
			// when start_loop >= end_loop, no index is allowed
			if start_loop > end_loop {
				return vec![];
			}
			let start = start_loop as usize;
			let end = end_loop as usize;
			let mut allow_indexs = Vec::with_capacity((end - start + 1) as usize);
			for i in start..=end {
				let cur_index = (i as isize * n + index) as usize;
				if cur_index < 1 {
					continue;
				}
				// last index need -1 for real list index
				allow_indexs.push(cur_index - 1);
			}
			return allow_indexs;
		}
		// only index
		let index = index
			.expect("Nth must have 'index' value when 'n' is not setted.")
			.parse::<isize>()
			.expect("Nth's index is not a correct number");
		if index <= 0 || index > (total as isize) {
			return vec![];
		}
		return vec![(index - 1) as usize];
	}
}

/// RegExp
#[derive(Debug)]
pub struct RegExp<'a> {
	pub cache: bool,
	pub context: &'a str,
}

impl<'a> Pattern for RegExp<'a> {
	fn matched(&self, chars: &[char]) -> Option<Matched> {
		let Self { context, cache } = *self;
		let content = chars.iter().collect::<String>();
		let rule = RegExp::get_rule(context, cache);
		if let Some(caps) = rule.captures(to_static_str(content)) {
			let total_len = caps[0].len();
			let mut data = HashMap::with_capacity(caps.len() - 1);
			for (index, m) in caps.iter().skip(1).enumerate() {
				if let Some(m) = m {
					data.insert(to_static_str((index + 1).to_string()), m.as_str());
				}
			}
			let result = chars[..total_len].to_vec();
			return Some(Matched {
				chars: result,
				name: "regexp",
				data,
			});
		}
		None
	}
	fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
		let mut cache = true;
		if !s.is_empty() {
			if s == "!" {
				cache = false;
			} else {
				return Err("Wrong param of Pattern type 'regexp', just allow '!' to generate a regexp with 'cached' field falsely.".into());
			}
		}
		Ok(Box::new(RegExp {
			context: to_static_str(p.to_string()),
			cache,
		}))
	}
}

impl<'a> RegExp<'a> {
	pub fn get_rule(context: &str, cache: bool) -> Arc<Regex> {
		let wrong_regex = format!("Wrong regex context '{}'", context);
		let last_context = String::from("^") + context;
		let rule = if cache {
			let mut regexs = REGEXS.lock().unwrap();
			if let Some(rule) = regexs.get(&last_context[..]) {
				Arc::clone(rule)
			} else {
				let key = &to_static_str(last_context);
				let rule = Regex::new(key).expect(&wrong_regex);
				let value = Arc::new(rule);
				let result = Arc::clone(&value);
				regexs.insert(key, value);
				result
			}
		} else {
			let key = &last_context[..];
			Arc::new(Regex::new(key).expect(&wrong_regex))
		};
		rule
	}
}

/// Nested
#[derive(Debug, Default)]
pub struct NestedSelector;

impl Pattern for NestedSelector {
	fn matched(&self, _chars: &[char]) -> Option<Matched> {
		None
	}
	// from params to pattern
	fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
		check_params_return(&[s, p], || Box::new(NestedSelector::default()))
	}
	// set to be nested
	fn is_nested(&self) -> bool {
		true
	}
}

pub fn add_pattern(name: &'static str, from_handle: FromParamsFn) {
	let mut patterns = PATTERNS.lock().unwrap();
	if patterns.get(name).is_some() {
		panic!("The pattern '{}' is already exist.", name);
	} else {
		patterns.insert(name, from_handle);
	}
}

pub(crate) fn init() {
	// add lib supported patterns
	add_pattern("identity", Box::new(Identity::from_params));
	add_pattern("spaces", Box::new(Spaces::from_params));
	add_pattern("attr_key", Box::new(AttrKey::from_params));
	add_pattern("index", Box::new(Index::from_params));
	add_pattern("nth", Box::new(Nth::from_params));
	add_pattern("regexp", Box::new(RegExp::from_params));
	add_pattern("selector", Box::new(NestedSelector::from_params));
}

pub fn to_pattern(name: &str, s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
	let patterns = PATTERNS.lock().unwrap();
	if let Some(cb) = patterns.get(name) {
		return cb(s, p);
	}
	no_implemented(name);
}

pub fn exec(queues: &[Box<dyn Pattern>], chars: &[char]) -> (Vec<Matched>, usize, usize, bool) {
	let mut start_index = 0;
	let mut result: Vec<Matched> = Vec::with_capacity(queues.len());
	let mut matched_num: usize = 0;
	for item in queues {
		if let Some(matched) = item.matched(&chars[start_index..]) {
			start_index += matched.chars.len();
			matched_num += 1;
			result.push(matched);
		} else {
			break;
		}
	}
	(result, start_index, matched_num, start_index == chars.len())
}

pub fn check_params_return<F: Fn() -> Box<dyn Pattern>>(
	params: &[&str],
	cb: F,
) -> Result<Box<dyn Pattern>, String> {
	for &p in params {
		if !p.is_empty() {
			let all_params = params.iter().fold(String::from(""), |mut r, &s| {
				r.push_str(s);
				r
			});
			return Err(format!("Unrecognized params '{}'", all_params));
		}
	}
	Ok(cb())
}
