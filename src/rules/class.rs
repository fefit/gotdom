use crate::interface::{BoxDynElement, IAttrValue};
use crate::selector::rule::{Matcher, MatcherData};
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
use crate::utils::get_class_list;
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"class",
		".{identity}",
		1000,
		vec![("identity", 0)],
		Box::new(|data: MatcherData| {
			let class_name = Rule::param(&data, "identity").expect("The 'class' selector is not correct");
			Matcher {
				one_handle: Some(Box::new(move |ele: &BoxDynElement, _| -> bool {
					if let Some(IAttrValue::Value(names, _)) = ele.get_attribute("class") {
						let class_list = get_class_list(&names);
						return class_list.contains(&class_name);
					}
					false
				})),
				..Default::default()
			}
		}),
	);
	rules.push(rule.into());
}
