use super::*;

mod selector_groups;
mod selector_nth_attr;
mod selector_step_parser;

pub(crate) use selector_groups::{
    parse_selector_chain, parse_selector_groups, split_selector_groups,
};
pub(crate) use selector_nth_attr::{
    find_matching_paren, parse_nth_child_selector, parse_selector_attr_condition,
};
pub(crate) use selector_step_parser::parse_selector_step;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelectorAttrCondition {
    Exists { key: String },
    Eq { key: String, value: String },
    StartsWith { key: String, value: String },
    EndsWith { key: String, value: String },
    Contains { key: String, value: String },
    Includes { key: String, value: String },
    DashMatch { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelectorPseudoClass {
    Scope,
    FirstChild,
    LastChild,
    FirstOfType,
    LastOfType,
    OnlyChild,
    OnlyOfType,
    Checked,
    Indeterminate,
    Disabled,
    Enabled,
    Required,
    Optional,
    Readonly,
    Readwrite,
    Empty,
    Focus,
    FocusWithin,
    Active,
    NthOfType(NthChildSelector),
    NthLastOfType(NthChildSelector),
    Not(Vec<Vec<SelectorPart>>),
    Is(Vec<Vec<SelectorPart>>),
    Where(Vec<Vec<SelectorPart>>),
    Has(Vec<Vec<SelectorPart>>),
    NthChild(NthChildSelector),
    NthLastChild(NthChildSelector),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NthChildSelector {
    Exact(usize),
    Odd,
    Even,
    AnPlusB(i64, i64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SelectorStep {
    pub(crate) tag: Option<String>,
    pub(crate) universal: bool,
    pub(crate) id: Option<String>,
    pub(crate) classes: Vec<String>,
    pub(crate) attrs: Vec<SelectorAttrCondition>,
    pub(crate) pseudo_classes: Vec<SelectorPseudoClass>,
}

impl SelectorStep {
    pub(crate) fn id_only(&self) -> Option<&str> {
        if !self.universal
            && self.tag.is_none()
            && self.classes.is_empty()
            && self.attrs.is_empty()
            && self.pseudo_classes.is_empty()
        {
            self.id.as_deref()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectorCombinator {
    Descendant,
    Child,
    AdjacentSibling,
    GeneralSibling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectorPart {
    pub(crate) step: SelectorStep,
    // Relation to previous (left) selector part.
    pub(crate) combinator: Option<SelectorCombinator>,
}
