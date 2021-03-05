use super::{
	BoxDynNode, BoxDynText, Elements, IAttrValue, INodeTrait, INodeType, InsertPosition, Texts,
};
use crate::error::Error as IError;
use std::error::Error;
use std::ops::Range;

pub type BoxDynElement<'a> = Box<dyn IElementTrait + 'a>;
pub type MaybeElement<'a> = Option<BoxDynElement<'a>>;
pub trait IElementTrait: INodeTrait {
	fn is(&self, ele: &BoxDynElement) -> bool {
		if let Some(uuid) = self.uuid() {
			if let Some(o_uuid) = ele.uuid() {
				return uuid == o_uuid;
			}
		}
		false
	}
	// root element
	fn root<'b>(&self) -> BoxDynElement<'b> {
		let mut root = self.parent();
		loop {
			if root.is_some() {
				let parent = root.as_ref().unwrap().parent();
				if let Some(parent) = &parent {
					root = Some(parent.cloned());
				} else {
					break;
				}
			} else {
				break;
			}
		}
		root.unwrap_or_else(|| self.cloned())
	}
	// cloned
	fn cloned<'b>(&self) -> BoxDynElement<'b> {
		let ele = self.clone_node();
		ele.typed().into_element().unwrap()
	}
	// next sibling
	fn next_element_sibling<'b>(&self) -> MaybeElement<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			let total = parent.child_nodes_length();
			// find the next
			for cur_index in index + 1..total {
				let ele = parent
					.child_nodes_item(cur_index)
					.expect("Child nodes item index must less than total");
				if matches!(ele.node_type(), INodeType::Element) {
					return Some(
						ele
							.typed()
							.into_element()
							.expect("Call `typed` for element ele."),
					);
				}
			}
		}
		None
	}
	// next siblings
	fn next_element_siblings<'b>(&self) -> Elements<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			let total = parent.child_nodes_length();
			let start_index = index + 1;
			// find the next
			let mut result: Elements = Elements::with_capacity(total - start_index);
			for cur_index in start_index..total {
				let ele = parent
					.child_nodes_item(cur_index)
					.expect("Child nodes item index must less than total");
				if matches!(ele.node_type(), INodeType::Element) {
					result.push(
						ele
							.typed()
							.into_element()
							.expect("Call `typed` for element ele."),
					);
				}
			}
			return result;
		}
		Elements::new()
	}
	// previous sibling
	fn previous_element_sibling<'b>(&self) -> MaybeElement<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			if index > 0 {
				// find the prev
				for cur_index in (0..index).rev() {
					let ele = parent
						.child_nodes_item(cur_index)
						.expect("Child nodes item index must less than total");
					if matches!(ele.node_type(), INodeType::Element) {
						return Some(
							ele
								.typed()
								.into_element()
								.expect("Call `typed` for element ele."),
						);
					}
				}
			}
		}
		None
	}
	// previous siblings
	fn previous_element_siblings<'b>(&self) -> Elements<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			if index > 0 {
				// find the prev
				let mut result: Elements = Elements::with_capacity(index);
				for cur_index in 0..index {
					let ele = parent
						.child_nodes_item(cur_index)
						.expect("Child nodes item index must less than total");
					if matches!(ele.node_type(), INodeType::Element) {
						result.push(
							ele
								.typed()
								.into_element()
								.expect("Call `typed` for element ele."),
						);
					}
				}
				return result;
			}
		}
		Elements::new()
	}
	// siblings
	fn siblings<'b>(&self) -> Elements<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			if index == 0 {
				return self.next_element_siblings();
			}
			let total = parent.child_nodes_length();
			if index == total - 1 {
				return self.previous_element_siblings();
			}
			let mut result: Elements = Elements::with_capacity(total - 1);
			fn loop_handle(range: &Range<usize>, parent: &BoxDynElement, result: &mut Elements) {
				for cur_index in range.start..range.end {
					let ele = parent
						.child_nodes_item(cur_index)
						.expect("Child nodes item index must less than total");
					if matches!(ele.node_type(), INodeType::Element) {
						result.push(
							ele
								.typed()
								.into_element()
								.expect("Call `typed` for element ele."),
						);
					}
				}
			}
			loop_handle(&(0..index), parent, &mut result);
			loop_handle(&(index + 1..total), parent, &mut result);
			return result;
		}
		Elements::new()
	}
	// tag name
	fn tag_name(&self) -> &str;
	// childs
	fn child_nodes_length(&self) -> usize;
	fn child_nodes_item<'b>(&self, index: usize) -> Option<BoxDynNode<'b>>;
	fn child_nodes<'b>(&self) -> Vec<BoxDynNode<'b>> {
		let total = self.child_nodes_length();
		let mut result = Vec::with_capacity(total);
		for index in 0..total {
			result.push(
				self
					.child_nodes_item(index)
					.expect("child nodes index must less than total."),
			);
		}
		result
	}
	fn children<'b>(&self) -> Elements<'b> {
		let child_nodes = self.child_nodes();
		let mut result = Elements::with_capacity(child_nodes.len());
		for ele in child_nodes.iter() {
			if let INodeType::Element = ele.node_type() {
				let ele = ele.clone_node();
				result.push(ele.typed().into_element().unwrap());
			}
		}
		result
	}
	// get all childrens
	fn childrens<'b>(&self) -> Elements<'b> {
		let childs = self.children();
		let count = childs.length();
		if count > 0 {
			let mut result = Elements::with_capacity(5);
			let all_nodes = result.get_mut_ref();
			for c in childs.get_ref() {
				all_nodes.push(c.cloned());
				all_nodes.extend(c.childrens());
			}
			return result;
		}
		Elements::new()
	}
	// attribute
	fn get_attribute(&self, name: &str) -> Option<IAttrValue>;
	fn set_attribute(&mut self, name: &str, value: Option<&str>);
	fn remove_attribute(&mut self, name: &str);
	fn has_attribute(&self, name: &str) -> bool {
		self.get_attribute(name).is_some()
	}
	// html
	fn html(&self) -> &str {
		self.inner_html()
	}
	fn inner_html(&self) -> &str;
	fn outer_html(&self) -> &str;

	// append child, insert before, remove child
	fn insert_adjacent(&mut self, position: &InsertPosition, ele: &BoxDynElement);
	fn remove_child(&mut self, ele: BoxDynElement);
	// texts
	fn texts<'b>(&self, _limit_depth: u32) -> Option<Texts<'b>> {
		None
	}
	// special for content tag, 'style','script','title','textarea'
	#[allow(clippy::boxed_local)]
	fn into_text<'b>(self: Box<Self>) -> Result<BoxDynText<'b>, Box<dyn Error>> {
		Err(Box::new(IError::InvalidTraitMethodCall {
			method: "into_text".into(),
			message: "The into_text method is not implemented.".into(),
		}))
	}
}
