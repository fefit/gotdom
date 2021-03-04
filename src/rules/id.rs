use crate::selector::rule::{Matcher, MatcherData, Rule, RuleItem};
use crate::{constants::USE_CACHE_DATAKEY, interface::Elements};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = RuleItem {
		name: "id",
		context: "#{identity}",
		rule: Rule {
			priority: 10000,
			in_cache: true,
			fields: vec![("identity", 0), USE_CACHE_DATAKEY],
			handle: Box::new(|data: MatcherData| {
				let id = Rule::param(&data, "identity").expect("The 'id' selector is not correct");
				Matcher {
					all_handle: Some(Box::new(move |eles: &Elements, use_cache: Option<bool>| {
						let use_cache = use_cache.is_some();
						let mut result = Elements::with_capacity(1);
						if !eles.is_empty() {
							let first_ele = eles
								.get_ref()
								.get(0)
								.expect("The elements must have at least one element.");
							if let Some(doc) = &first_ele.owner_document() {
								if let Some(id_element) = &doc.get_element_by_id(id) {
									if use_cache {
										// just add, will checked if the element contains the id element
										result.push(id_element.cloned());
									} else {
										// filter methods, will filtered in elements
										for ele in eles.get_ref() {
											if ele.is(id_element) {
												result.push(ele.cloned());
												break;
											}
										}
									}
								}
							}
						}
						result
					})),
					..Default::default()
				}
			}),
			queues: Vec::new(),
		},
	};
	rules.push(rule);
}
