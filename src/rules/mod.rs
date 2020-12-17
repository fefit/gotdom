use crate::selector::rule::{self, add_rules, RuleItem};
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};
lazy_static! {
  static ref IS_RULES_INIT: AtomicBool = AtomicBool::new(false);
}
pub(crate) mod all;
pub(crate) mod attr;
pub(crate) mod class;
pub(crate) mod id;
pub(crate) mod name;
pub(crate) mod pseudo;
pub fn init() {
  if !IS_RULES_INIT.load(Ordering::SeqCst) {
    // init rule
    rule::init();
    // add rules
    let mut rules: Vec<RuleItem> = Vec::with_capacity(20);
    // keep the init order
    class::init(&mut rules);
    id::init(&mut rules);
    attr::init(&mut rules);
    pseudo::init(&mut rules);
    name::init(&mut rules);
    all::init(&mut rules);
    add_rules(rules);
    // set init true
    IS_RULES_INIT.store(true, Ordering::SeqCst);
  }
}