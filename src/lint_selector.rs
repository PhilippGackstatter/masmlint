use std::collections::BTreeMap;

use miette::{Report, Result};

use crate::{
    EarlyLintPass,
    lints::{BareAssert, PushImmediate},
};

#[derive(Debug, Clone, Default)]
pub enum LintSelector {
    #[default]
    All,
    Select(Vec<String>),
    Exclude(Vec<String>),
}

impl LintSelector {
    pub fn select(self) -> Result<Vec<Box<dyn EarlyLintPass>>> {
        let mut lints = all_lints();
        match self {
            LintSelector::All => Ok(lints.into_values().collect()),
            LintSelector::Select(selected) => {
                let mut selected_lints = Vec::new();

                for selected_lint in selected {
                    let lint = lints.remove(selected_lint.as_str()).ok_or_else(|| {
                        Report::msg(format!("failed to select unknown lint `{selected_lint}`"))
                    })?;
                    selected_lints.push(lint);
                }

                Ok(selected_lints)
            },
            LintSelector::Exclude(excluded) => {
                for excluded_lint in excluded {
                    lints.remove(excluded_lint.as_str()).ok_or_else(|| {
                        Report::msg(format!("failed to exclude unknown lint `{excluded_lint}`"))
                    })?;
                }

                Ok(lints.into_values().collect())
            },
        }
    }
}

fn all_lints() -> BTreeMap<&'static str, Box<dyn EarlyLintPass>> {
    BTreeMap::from_iter([
        (BareAssert::NAME, bare_assert()),
        (PushImmediate::NAME, push_immediate()),
    ])
}

fn bare_assert() -> Box<dyn EarlyLintPass> {
    Box::new(BareAssert)
}

fn push_immediate() -> Box<dyn EarlyLintPass> {
    Box::new(PushImmediate::new())
}
