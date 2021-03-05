use super::{BoxDynElement, BoxDynNode, Elements};
use crate::utils::to_static_str;
use std::error::Error;
use std::rc::Rc;

pub type MaybeDoc = Option<Box<dyn IDocumentTrait>>;
pub type IErrorHandle = Box<dyn Fn(Box<dyn Error>)>;
pub trait IDocumentTrait {
	fn get_element_by_id<'b>(&self, id: &str) -> Option<BoxDynElement<'b>>;
	fn source_code(&self) -> &'static str;
	// get root node
	fn get_root_node<'b>(&self) -> BoxDynNode<'b>;
	// document element, html tag
	fn document_element<'b>(&self) -> Option<BoxDynElement<'b>> {
		if let Some(root) = &self.get_root_node().root_element() {
			let root = Elements::with_node(root);
			return root.find("html").get(0).map(|ele| ele.cloned());
		}
		None
	}
	// title
	fn title(&self) -> Option<&'static str> {
		if let Some(root) = &self.get_root_node().root_element() {
			let root = Elements::with_node(root);
			let title = root.find("head").eq(0).find("title");
			if !title.is_empty() {
				return Some(to_static_str(String::from(title.text())));
			}
		}
		None
	}
	// head
	fn head<'b>(&self) -> Option<BoxDynElement<'b>> {
		if let Some(root) = &self.get_root_node().root_element() {
			let root = Elements::with_node(root);
			return root.find("head").get(0).map(|ele| ele.cloned());
		}
		None
	}
	// head
	fn body<'b>(&self) -> Option<BoxDynElement<'b>> {
		if let Some(root) = &self.get_root_node().root_element() {
			let root = Elements::with_node(root);
			return root.find("body").get(0).map(|ele| ele.cloned());
		}
		None
	}
	// onerror
	fn onerror(&self) -> Option<Rc<IErrorHandle>> {
		None
	}
	// trigger error
	fn trigger_error(&self, error: Box<dyn Error>) {
		if let Some(handle) = &self.onerror() {
			handle(error);
		}
	}
}
