//! # Addressing
//!
//! How the merge names a component, a property and a line, and how it finds
//! again in one calendar what it read in another.
//!
//! A component is matched across the three calendars by its `UID` and its
//! `RECURRENCE-ID`, the identity iCalendar itself uses (RFC 5545 3.8.4.7,
//! 3.8.4.4): an override of one instance is never confused with the series it
//! belongs to, however the two are ordered in the file.
//!
//! A component carrying no `UID` (a `VALARM`, a `STANDARD`, a `VTIMEZONE`
//! observance) is matched by its position among its same-named siblings.
//!
//! `BEGIN` and `END` are the component envelope rather than properties, and a
//! bare, envelope-less record holds them as lines like any other, so they are
//! skipped everywhere: no side is reported as adding or removing one, and none
//! is copied into a calendar that would then refuse to parse.

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    prop::IcalPropKind,
    tree::{
        cst::{IcalCst, IcalItem},
        leaf::IcalLeaf,
        line::IcalLine,
        merge::{IcalComponentPath, IcalComponentStep, IcalPropPath, diff::identity_in},
    },
};

/// One component of a calendar, with the path that addresses it.
pub(super) struct Node<'c, 'a> {
    pub(super) path: IcalComponentPath<'a>,
    pub(super) cst: &'c IcalCst<'a>,
}

impl<'a> IcalCst<'a> {
    /// Every component of the calendar, the root first, each with its path.
    pub(super) fn nodes(&self) -> Vec<Node<'_, 'a>> {
        let mut out = Vec::new();

        self.walk(IcalComponentPath::default(), &mut out);

        out
    }

    /// Collect this component and everything nested in it.
    fn walk<'c>(&'c self, path: IcalComponentPath<'a>, out: &mut Vec<Node<'c, 'a>>) {
        out.push(Node {
            path: path.clone(),
            cst: self,
        });

        let mut seen: Vec<(String, usize)> = Vec::new();

        for child in self.children() {
            let name = child.upper_name();
            let ordinal = match seen.iter_mut().find(|(held, _)| *held == name) {
                Some((_, count)) => {
                    *count += 1;
                    *count
                }
                None => {
                    seen.push((name.clone(), 0));
                    0
                }
            };

            let mut nested = path.clone();
            nested.0.push(IcalComponentStep {
                key: Cow::Owned(child.identity(ordinal)),
                name: Cow::Owned(name),
            });

            child.walk(nested, out);
        }
    }

    /// The components nested directly in this one.
    pub(super) fn children(&self) -> impl Iterator<Item = &IcalCst<'a>> {
        self.items.iter().filter_map(|item| match item {
            IcalItem::Component(child) => Some(&**child),
            _ => None,
        })
    }

    /// This component's name, uppercase.
    pub(super) fn upper_name(&self) -> String {
        self.begin
            .as_ref()
            .map(|begin| begin.raw_value_str().to_ascii_uppercase())
            .unwrap_or_default()
    }

    /// This component's identity among its same-named siblings: its `UID`,
    /// with the `RECURRENCE-ID` after a solidus when it overrides one
    /// instance, or its position when it carries no `UID`.
    pub(super) fn identity(&self, ordinal: usize) -> String {
        let Some(uid) = self.first_raw(IcalPropKind::Uid) else {
            return ordinal.to_string();
        };

        match self.first_raw(IcalPropKind::RecurrenceId) {
            Some(id) => format!("{uid}/{id}"),
            None => uid,
        }
    }

    /// The raw text of this component's first property of this kind.
    fn first_raw(&self, kind: IcalPropKind) -> Option<String> {
        self.prop_lines()
            .find(|line| line.name.get().eq_ignore_ascii_case(&kind))
            .map(|line| line.raw_value_str().into_owned())
    }

    /// This component's property lines, in source order, envelope excluded.
    pub(super) fn prop_lines(&self) -> impl Iterator<Item = &IcalLine<'a>> {
        self.items
            .iter()
            .filter_map(|item| match item {
                IcalItem::Prop(line) => Some(line),
                _ => None,
            })
            .filter(|line| !line.is_structural())
    }

    /// The component a path names.
    pub(super) fn at(&self, path: &IcalComponentPath<'a>) -> Option<&IcalCst<'a>> {
        let mut held = self;

        for step in &path.0 {
            let mut ordinal = 0;

            held = held.children().find(|child| {
                if child.upper_name() != step.name {
                    return false;
                }

                let matched = child.identity(ordinal) == step.key;
                ordinal += 1;
                matched
            })?;
        }

        Some(held)
    }

    /// The same, mutably.
    pub(super) fn at_mut(&mut self, path: &IcalComponentPath<'a>) -> Option<&mut IcalCst<'a>> {
        let mut held = self;

        for step in &path.0 {
            let mut ordinal = 0;

            held = held.items.iter_mut().find_map(|item| {
                let IcalItem::Component(child) = item else {
                    return None;
                };

                if child.upper_name() != step.name {
                    return None;
                }

                let matched = child.identity(ordinal) == step.key;
                ordinal += 1;
                matched.then_some(&mut **child)
            })?;
        }

        Some(held)
    }

    /// Where the component a path names sits among this one's items.
    pub(super) fn component_position(&self, at: &IcalComponentPath<'_>) -> Option<usize> {
        let step = at.0.last()?;
        let mut ordinal = 0;

        self.items.iter().position(|item| {
            let IcalItem::Component(child) = item else {
                return false;
            };

            if child.upper_name() != step.name {
                return false;
            }

            let held = child.identity(ordinal);
            ordinal += 1;
            held == step.key
        })
    }

    /// Which of this component's same-named lines a property path names: the
    /// one carrying its identity where it has one, and the position it was
    /// given otherwise.
    pub(super) fn line_ordinal(
        &self,
        at: &IcalPropPath<'_>,
        position: Option<usize>,
    ) -> Option<usize> {
        let Some(identity) = &at.identity else {
            return position;
        };

        self.prop_lines()
            .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
            .position(|line| line.value_key() == **identity)
    }

    /// The line a property path names inside this component.
    pub(super) fn line_at(
        &self,
        at: &IcalPropPath<'_>,
        position: Option<usize>,
    ) -> Option<&IcalLine<'a>> {
        let ordinal = self.line_ordinal(at, position)?;

        self.prop_lines()
            .filter(|line| line.name.get().eq_ignore_ascii_case(&at.name))
            .nth(ordinal)
    }

    /// The line of that name at that position inside this component, mutably.
    pub(super) fn nth_line_mut(&mut self, name: &str, at: usize) -> Option<&mut IcalLine<'a>> {
        self.items
            .iter_mut()
            .filter_map(|item| match item {
                IcalItem::Prop(line) => Some(line),
                _ => None,
            })
            .filter(|line| !line.is_structural() && line.name.get().eq_ignore_ascii_case(name))
            .nth(at)
    }

    /// Where the line of that name at that position sits among this
    /// component's items.
    pub(super) fn line_position(&self, name: &str, at: usize) -> Option<usize> {
        let mut ordinal = 0;

        self.items.iter().position(|item| {
            let IcalItem::Prop(line) = item else {
                return false;
            };

            if line.is_structural() || !line.name.get().eq_ignore_ascii_case(name) {
                return false;
            }

            let held = ordinal;
            ordinal += 1;
            held == at
        })
    }
}

impl<'a> IcalLine<'a> {
    /// Whether the line is a component envelope keyword rather than a
    /// property.
    pub(super) fn is_structural(&self) -> bool {
        let name = self.name.get();

        name.eq_ignore_ascii_case("BEGIN") || name.eq_ignore_ascii_case("END")
    }

    /// The whole raw value of the line, as written.
    ///
    /// Not its first value: a `CAL-ADDRESS` list is one value in the merge's
    /// eyes, and reading only up to the first comma would give two different
    /// lines one identity.
    pub(super) fn value_text(&self) -> String {
        let mut out = Vec::new();

        self.value.write_bytes(&mut out);

        String::from_utf8_lossy(&out).into_owned()
    }

    /// The same value, normalised into the key an identity is compared on.
    ///
    /// Matching normalises and writing is exact. A URI scheme is
    /// case-insensitive (RFC 3986 3.1) and so is a mail address host, so
    /// `MAILTO:Ada@Example.com` and `mailto:ada@example.com` name one person
    /// and have to meet. What goes back on the wire is the bytes the line
    /// arrived with.
    pub(super) fn value_key(&self) -> String {
        self.value_text().to_lowercase()
    }

    /// A copy of the line that is sure to end.
    ///
    /// A side may have been read from a truncated download, its last line
    /// carrying no line ending. Copied into the middle of a calendar it would
    /// swallow the line after it, `END:VCALENDAR` included, and the merge
    /// would emit bytes its own parser refuses.
    pub(super) fn terminated(&self) -> IcalLine<'a> {
        let mut held = self.clone();

        if held.eol.get().is_empty() {
            held.eol = IcalLeaf(Cow::Borrowed("\r\n"));
        }

        held
    }
}

impl<'a> IcalComponentPath<'a> {
    /// The path with its last step dropped: the component holding the one it
    /// names.
    pub(super) fn parent(&self) -> IcalComponentPath<'a> {
        let mut parent = self.clone();
        parent.0.pop();
        parent
    }

    /// Every proper ancestor path, nearest first.
    pub(super) fn ancestors(&self) -> impl Iterator<Item = IcalComponentPath<'a>> + '_ {
        (1..self.0.len()).map(|depth| IcalComponentPath(self.0[..depth].to_vec()))
    }
}

impl<'a> IcalPropPath<'a> {
    /// What tells a line from its component's other properties of that name:
    /// its identity where it has one, and its position either way.
    pub(super) fn of(
        component: &IcalComponentPath<'a>,
        lines: &[&IcalLine<'a>],
        at: usize,
    ) -> Self {
        let name = lines[at].name.get();
        let index = lines[..at]
            .iter()
            .filter(|held| held.name.get().eq_ignore_ascii_case(name))
            .count();

        IcalPropPath {
            component: component.clone(),
            name: Cow::Owned(name.to_owned()),
            index,
            identity: identity_in(lines, at),
        }
    }
}
